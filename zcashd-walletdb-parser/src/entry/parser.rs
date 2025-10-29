use anyhow::{Result, ensure};

use crate::{
    constants::BTDATAOFF,
    entry::constants::{Field, OverflowRef},
    leaf::{LeafItem, ParsedLeafEntry, parse_leaf_entry},
    page::PageType,
    util::{Endian, PageHeader, page_slice, parse_page_header, u16e, u32e},
};

/// Read absolute byte offsets from the slot array [BTDATAOFF .. lower).
#[inline]
fn slot_abs_offsets<'a>(page: &'a [u8], e: Endian, lower: u16) -> impl Iterator<Item = usize> + 'a {
    let lower = lower as usize;
    (BTDATAOFF..lower)
        .step_by(2)
        .map(move |i| u16e(e, &page[i..i + 2]) as usize)
}

/// Walk an OVERFLOW chain and materialize `total_len` bytes.
/// Each page contributes `page[BTDATAOFF..]`; follow `hdr.next`.
fn read_overflow_chain(
    all: &[u8],
    ps: usize,
    e: Endian,
    r: OverflowRef,
) -> anyhow::Result<Vec<u8>> {
    use anyhow::ensure;
    let mut out = Vec::with_capacity(r.total_len as usize);
    let mut pg = r.first_page;
    let mut rem = r.total_len as usize;

    while rem > 0 {
        let page = page_slice(all, ps, pg);
        let hdr = parse_page_header(page, e)?;
        ensure!(
            matches!(hdr.ptype, PageType::Overflow),
            "expected overflow page"
        );
        let payload = &page[BTDATAOFF..];
        let take = rem.min(payload.len());
        out.extend_from_slice(&payload[..take]);
        rem -= take;
        if rem == 0 {
            break;
        }
        ensure!(
            hdr.next != 0,
            "overflow chain ended early (need {rem} more bytes)"
        );
        pg = hdr.next;
    }
    Ok(out)
}
/// Extract (key,value) pairs from a **leaf** page.
/// Pairs are formed by taking the next **non-deleted** entry as value
/// for the previous **non-deleted** entry as key.
pub fn leaf_pairs_on_page(
    all: &[u8],
    ps: usize,
    e: Endian,
    page: &[u8],
    hdr: &PageHeader,
) -> anyhow::Result<Vec<(Vec<u8>, Vec<u8>)>> {
    use anyhow::ensure;
    ensure!(matches!(hdr.ptype, PageType::Leaf), "not a leaf page");

    // Build absolute offsets from the slot array.
    let offs: Vec<usize> = slot_abs_offsets(page, e, hdr.entries).collect();

    for &off in &offs {
        // entry should live in packed region near the end of the page
        if off < hdr.hf_offset as usize || off + 3 > page.len() {
            // Log suspicious offsets instead of crashing
            eprintln!(
                "skip bad slot off={off} upper={} len={}",
                hdr.hf_offset,
                page.len()
            );
            continue;
        }
    }

    let mut out = Vec::new();
    let mut pend: Option<ParsedLeafEntry> = None;

    for off in offs {
        let entry = parse_leaf_entry(page, off, e)?;
        if entry.deleted {
            continue;
        }

        match pend.take() {
            None => {
                // treat as key, wait for next non-deleted for value
                pend = Some(entry);
            }
            Some(k) => {
                // materialize key
                let key = match k.item {
                    LeafItem::KeyData(s) => s.to_vec(),
                    LeafItem::Overflow {
                        first_pg,
                        total_len,
                    } => read_overflow_chain(
                        all,
                        ps,
                        e,
                        OverflowRef {
                            first_page: first_pg,
                            total_len,
                        },
                    )?,
                };
                // materialize value
                let val = match entry.item {
                    LeafItem::KeyData(s) => s.to_vec(),
                    LeafItem::Overflow {
                        first_pg,
                        total_len,
                    } => read_overflow_chain(
                        all,
                        ps,
                        e,
                        OverflowRef {
                            first_page: first_pg,
                            total_len,
                        },
                    )?,
                };
                out.push((key, val));
            }
        }
    }
    // If pend is Some(..) here, the page ended with an unpaired key (tombstoned value, etc.). Skip it.
    Ok(out)
}

/// Parse one BLEAF entry at `off` into key/data fields (either inline slices or BigRef).
fn parse_bleaf_fields<'a>(page: &'a [u8], off: usize, e: Endian) -> Result<(Field<'a>, Field<'a>)> {
    ensure!(off + 9 <= page.len(), "BLEAF header out of bounds");
    let ksize = u32e(e, &page[off..off + 4]) as usize;
    let dsize = u32e(e, &page[off + 4..off + 8]) as usize;
    let flags = page[off + 8];
    let mut p = off + 9;

    // key
    let key = if (flags & 0x01) == 0 {
        ensure!(p + ksize <= page.len(), "inline key OOB");
        let s = &page[p..p + ksize];
        p += ksize;
        Field::Inline(s)
    } else {
        ensure!(p + 8 <= page.len(), "big-key ref OOB");
        let first_page = u32e(e, &page[p..p + 4]);
        let total_len = u32e(e, &page[p + 4..p + 8]);
        p += 8;
        Field::Overflow(OverflowRef {
            first_page,
            total_len,
        })
    };

    // data
    let data = if (flags & 0x02) == 0 {
        ensure!(p + dsize <= page.len(), "inline data OOB");
        let s = &page[p..p + dsize];
        Field::Inline(s)
    } else {
        ensure!(p + 8 <= page.len(), "big-data ref OOB");
        let first_page = u32e(e, &page[p..p + 4]);
        let total_len = u32e(e, &page[p + 4..p + 8]);
        Field::Overflow(OverflowRef {
            first_page,
            total_len,
        })
    };

    Ok((key, data))
}

/// Follow an overflow chain and materialize `total_len` bytes.
/// Each overflow page’s payload is `page[BTDATAOFF..]`. Use header.next to chain.
pub fn read_overflow(all: &[u8], ps: usize, e: Endian, br: OverflowRef) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(br.total_len as usize);
    let mut pg = br.first_page;
    let mut rem = br.total_len as usize;

    while rem > 0 {
        let page = page_slice(all, ps, pg);
        let hdr = parse_page_header(page, e)?;
        ensure!(
            matches!(hdr.ptype, PageType::Overflow),
            "expected overflow page, got {:?}",
            hdr.ptype
        );

        let payload = &page[BTDATAOFF..];
        let take = rem.min(payload.len());
        out.extend_from_slice(&payload[..take]);
        rem -= take;

        if rem == 0 {
            break;
        }
        ensure!(
            hdr.next != 0,
            "overflow chain ended early (need {rem} more bytes)"
        );
        pg = hdr.next;
    }
    Ok(out)
}

/// Read one BLEAF item fully into owned Vecs (follows overflow if needed).
pub fn read_leaf_item(
    all: &[u8],
    ps: usize,
    e: Endian,
    page: &[u8],
    off: usize,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (kf, df) = parse_bleaf_fields(page, off, e)?;
    let key = match kf {
        Field::Inline(s) => s.to_vec(),
        Field::Overflow(r) => read_overflow(all, ps, e, r)?,
    };
    let val = match df {
        Field::Inline(s) => s.to_vec(),
        Field::Overflow(r) => read_overflow(all, ps, e, r)?,
    };
    Ok((key, val))
}

/// Convenience wrapper: extract pairs from a leaf page by page number.
pub fn extract_leaf_pairs(
    all: &[u8],
    ps: usize,
    e: Endian,
    leaf_pgno: u32,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let page = page_slice(all, ps, leaf_pgno);
    let hdr = parse_page_header(page, e)?;
    ensure!(matches!(hdr.ptype, PageType::Leaf), "not a leaf page");
    leaf_pairs_on_page(all, ps, e, page, &hdr)
}

pub fn read_compact_size(s: &[u8]) -> Option<(u64, usize)> {
    if s.is_empty() {
        return None;
    }
    let b0 = s[0];
    match b0 {
        0x00..=0xfc => Some((b0 as u64, 1)),
        0xfd => {
            if s.len() >= 3 {
                Some((u16::from_le_bytes([s[1], s[2]]) as u64, 3))
            } else {
                None
            }
        }
        0xfe => {
            if s.len() >= 5 {
                Some((u32::from_le_bytes([s[1], s[2], s[3], s[4]]) as u64, 5))
            } else {
                None
            }
        }
        0xff => {
            if s.len() >= 9 {
                Some((
                    u64::from_le_bytes([s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[8]]),
                    9,
                ))
            } else {
                None
            }
        }
    }
}

pub fn split_walletdb_key(key: &[u8]) -> Option<(&str, &[u8])> {
    let (len, n) = read_compact_size(key)?;
    let len = len as usize;
    if key.len() < n + len {
        return None;
    }
    let tag_bytes = &key[n..n + len];
    let tag = core::str::from_utf8(tag_bytes).ok()?;
    Some((tag, &key[n + len..]))
}
