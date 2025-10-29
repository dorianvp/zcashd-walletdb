use crate::util::{Endian, u16e, u32e};

/// Leaf entry kinds in BDB 4.x/5.x.
/// 1 = inline bytes; 3 = overflow reference; high bit is "deleted".
#[derive(Debug)]
pub enum LeafItem<'a> {
    /// Key/value bytes lives inline in this page.
    KeyData(&'a [u8]),
    /// Key/value lives on an overflow chain.
    Overflow { first_pg: u32, total_len: u32 },
}

#[derive(Debug)]
pub struct ParsedLeafEntry<'a> {
    pub(crate) deleted: bool,
    pub(crate) item: LeafItem<'a>,
}

/// Parse a single **leaf** item at absolute `off`.
/// Layout:
///   - Inline:   len:u16, kind:u8(=1 or 0x81 if deleted), data[len]
///   - Overflow: pad:u16, kind:u8(=3 or 0x83 if deleted), pad:u8,
///               first_pg:u32, total_len:u32
pub fn parse_leaf_entry<'a>(
    page: &'a [u8],
    off: usize,
    e: Endian,
) -> anyhow::Result<ParsedLeafEntry<'a>> {
    use anyhow::{bail, ensure};
    ensure!(off + 3 <= page.len(), "leaf entry header OOB");
    let len = u16e(e, &page[off..off + 2]) as usize;
    let kind_raw = page[off + 2];
    let deleted = (kind_raw & 0x80) != 0;
    let kind = kind_raw & 0x7F;

    match kind {
        1 => {
            let start = off + 3;
            let end = start + len;
            ensure!(end <= page.len(), "KeyData OOB");
            Ok(ParsedLeafEntry {
                deleted,
                item: LeafItem::KeyData(&page[start..end]),
            })
        }
        3 => {
            let start = off + 4; // skip pad
            ensure!(start + 8 <= page.len(), "Overflow OOB");
            let first_pg = u32e(e, &page[start..start + 4]);
            let total_len = u32e(e, &page[start + 4..start + 8]);
            Ok(ParsedLeafEntry {
                deleted,
                item: LeafItem::Overflow {
                    first_pg,
                    total_len,
                },
            })
        }
        k => bail!("unknown leaf item kind {k} at off={off}"),
    }
}
