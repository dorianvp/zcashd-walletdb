use crate::page::PageType;

#[derive(Copy, Clone, Debug)]
pub enum Endian {
    Le,
    Be,
}
#[inline]
pub fn u16e(e: Endian, b: &[u8]) -> u16 {
    match e {
        Endian::Le => u16::from_le_bytes([b[0], b[1]]),
        Endian::Be => u16::from_be_bytes([b[0], b[1]]),
    }
}
#[inline]
pub fn u32e(e: Endian, b: &[u8]) -> u32 {
    match e {
        Endian::Le => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        Endian::Be => u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
    }
}
#[inline]
pub fn u64e(e: Endian, b: &[u8]) -> u64 {
    match e {
        Endian::Le => u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        Endian::Be => u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
    }
}

pub fn detect_endian(buf: &[u8]) -> Option<Endian> {
    const BTREE_MAGIC: u32 = 0x0005_3162; // stored at 12..16 in native endianness
    if buf.len() < 16 {
        return None;
    }
    if u32e(Endian::Le, &buf[12..16]) == BTREE_MAGIC {
        Some(Endian::Le)
    } else if u32e(Endian::Be, &buf[12..16]) == BTREE_MAGIC {
        Some(Endian::Be)
    } else {
        None
    }
}

pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[derive(Debug)]
pub struct PageHeader {
    pub lsn_file: u32, // 0..=3
    pub lsn_off: u32,  // 4..=7
    pub pgno: u32,     // 8..=11
    pub prev: u32,     // 12..=15
    pub next: u32,     // 16..=19
    pub flags: u32,    // 20..=23  (type = flags & 0x1f)
    pub lower: u16,    // 24..=25  (end of slot array)
    pub upper: u16,    // 26..=27  (start of data region)
    pub nslots: u16,   // computed: (lower - 28)/2
    pub ptype: PageType,
}

pub fn parse_page_header(page: &[u8], endian: Endian) -> anyhow::Result<PageHeader> {
    use anyhow::{bail, ensure};
    if page.len() < 28 {
        bail!("short page");
    }

    let lsn_file = u32e(endian, &page[0..4]);
    let lsn_off = u32e(endian, &page[4..8]);
    let pgno = u32e(endian, &page[8..12]);
    let prev = u32e(endian, &page[12..16]);
    let next = u32e(endian, &page[16..20]);
    let flags = u32e(endian, &page[20..24]);
    let lower = u16e(endian, &page[24..26]);
    let upper = u16e(endian, &page[26..28]);

    let ptype = PageType::from_flags(flags);
    const BTDATAOFF: usize = 28;

    // Only enforce and compute slots for leaf/internal pages
    let nslots = if matches!(ptype, PageType::Leaf | PageType::Internal) {
        ensure!(lower as usize >= BTDATAOFF, "lower too small");
        ensure!(lower <= upper, "lower > upper");
        ensure!(upper as usize <= page.len(), "upper beyond page");
        ((lower as usize - BTDATAOFF) / 2) as u16
    } else {
        0
    };

    Ok(PageHeader {
        lsn_file,
        lsn_off,
        pgno,
        prev,
        next,
        flags,
        lower,
        upper,
        nslots,
        ptype,
    })
}

fn scan_headers(bytes: &[u8], ps: usize, endian: Endian) -> anyhow::Result<()> {
    let npages = bytes.len() / ps;
    for i in 1..npages {
        let page = &bytes[i * ps..(i + 1) * ps];
        let hdr = parse_page_header(page, endian)?;
        println!(
            "page {:>3}: type={:?} slots={} lower={} upper={} prev={} next={} flags=0x{:08x}",
            i, hdr.ptype, hdr.nslots, hdr.lower, hdr.upper, hdr.prev, hdr.next, hdr.flags
        );
    }
    Ok(())
}

pub fn page_slice<'a>(all: &'a [u8], ps: usize, pgno: u32) -> &'a [u8] {
    let i = pgno as usize;
    &all[i * ps..(i + 1) * ps]
}
