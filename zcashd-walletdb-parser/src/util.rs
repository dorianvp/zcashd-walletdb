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
    pub lsn_file: u32,   // 0..=3
    pub lsn_off: u32,    // 4..=7
    pub pgno: u32,       // 8..=11
    pub prev: u32,       // 12..=15
    pub next: u32,       // 16..=19
    pub entries: u16,    // 20..=21  (#slots)
    pub hf_offset: u16,  // 22..=23  (start of data region)
    pub level: u8,       // 24
    pub ptype: PageType, // 25
}

pub fn parse_page_header(page: &[u8], e: Endian) -> anyhow::Result<PageHeader> {
    use anyhow::bail;
    if page.len() < 26 {
        bail!("short page");
    }
    Ok(PageHeader {
        lsn_file: u32e(e, &page[0..4]),
        lsn_off: u32e(e, &page[4..8]),
        pgno: u32e(e, &page[8..12]),
        prev: u32e(e, &page[12..16]),
        next: u32e(e, &page[16..20]),
        entries: u16e(e, &page[20..22]),
        hf_offset: u16e(e, &page[22..24]),
        level: page[24],
        ptype: PageType::from(page[25]),
    })
}

pub fn page_slice<'a>(all: &'a [u8], ps: usize, pgno: u32) -> &'a [u8] {
    let i = pgno as usize;
    &all[i * ps..(i + 1) * ps]
}
