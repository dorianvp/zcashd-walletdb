use core::fmt;

use crate::{
    page::PageType,
    util::{Endian, detect_endian, hex, u32e},
};

#[derive(Debug)]
pub struct BtreeMeta {
    pub endian: Endian,
    // Common 12-byte page header
    pub lsn_file: u32,   // 0..=3
    pub lsn_offset: u32, // 4..=7
    pub pgno: u32,       // 8..=11 (should be 0 on meta page)

    // Meta body (starts at byte 12 of page 0)
    pub magic: u32,    // 12..=15 (Btree magic 0x0005_3162)
    pub version: u32,  // 16..=19
    pub pagesize: u32, // 20..=23

    pub encrypt_alg: u8,  // 24
    pub p_type: PageType, // 25 (9 = meta, 3 = internal, 5 = leaf)
    pub metaflags: u8,    // 26
    pub _unused1: u8,     // 27

    pub free: u32,         // 28..=31  (freelist head page)
    pub last_pgno: u32,    // 32..=35  (highest allocated page)
    pub _unused3: u32,     // 36..=39
    pub key_count: u32,    // 40..=43  (cached stats; often 0)
    pub record_count: u32, // 44..=47  (cached stats; often 0)
    pub flags: u32,        // 48..=51  (btree meta flags)
    pub uid: [u8; 20],     // 52..=71  (file ID)

    pub _unused_after_uid: u32, // 72..=75
    pub minkey: u32,            // 76..=79  (DB->set_bt_minkey)
    pub re_len: u32,            // 80..=83  (RECNO fixed record length; 0 for Btree)
    pub re_pad: u32,            // 84..=87  (RECNO pad byte, commonly 0x20)
    pub root: u32,              // 88..=91  (root page number)

    // Tail (encryption-era fields; present even if unused)
    pub crypto_magic: u32, // 460..=463
    pub iv: [u8; 16],      // 476..=491
    pub chksum: [u8; 20],  // 496..=515
}

impl fmt::Display for BtreeMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BtreeMeta {{\n")?;
        writeln!(f, "  endianness   : {:?}", self.endian)?;
        writeln!(f, "  pagesize     : {}", self.pagesize)?;
        writeln!(f, "  page0.pgno   : {}", self.pgno)?;
        writeln!(
            f,
            "  lsn          : [{}][{}]",
            self.lsn_file, self.lsn_offset
        )?;
        writeln!(f, "  magic        : 0x{:08x}", self.magic)?;
        writeln!(f, "  version      : {}", self.version)?;
        writeln!(f, "  type         : {}", self.p_type)?; // 9=meta, 3=internal, 5=leaf
        writeln!(f, "  metaflags    : 0x{:x}", self.metaflags)?;
        writeln!(f, "  free         : {}", self.free)?;
        writeln!(f, "  last_pgno    : {}", self.last_pgno)?;
        writeln!(f, "  key_count    : {}", self.key_count)?;
        writeln!(f, "  record_count : {}", self.record_count)?;
        writeln!(f, "  flags        : 0x{:08x}", self.flags)?;
        writeln!(f, "  uid          : {}", hex(&self.uid))?;
        writeln!(f, "  minkey       : {}", self.minkey)?;
        writeln!(f, "  re_len       : {}", self.re_len)?;
        writeln!(f, "  re_pad       : 0x{:x}", self.re_pad)?;
        writeln!(f, "  root         : {}", self.root)?;
        if self.crypto_magic != 0 {
            writeln!(f, "  crypto_magic : 0x{:08x}", self.crypto_magic)?;
            writeln!(f, "  iv           : {}", hex(&self.iv))?;
            writeln!(f, "  chksum       : {}", hex(&self.chksum))?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub fn parse_btree_meta_page0(page: &[u8]) -> anyhow::Result<BtreeMeta> {
    use anyhow::{Context, bail};
    if page.len() < 512 {
        bail!("page buffer too small (<512)");
    }

    let endian = detect_endian(page).context("not a Btree meta page: magic not found at 12..16")?;
    let pagesize = u32e(endian, &page[20..24]);

    // Basic sanity
    if pagesize == 0 || pagesize as usize % 512 != 0 {
        bail!("implausible pagesize {pagesize}");
    }

    // Common header
    let lsn_file = u32e(endian, &page[0..4]);
    let lsn_offset = u32e(endian, &page[4..8]);
    let pgno = u32e(endian, &page[8..12]);

    // Meta body
    let magic = u32e(endian, &page[12..16]);
    let version = u32e(endian, &page[16..20]);

    let encrypt_alg = page[24];
    let p_type = PageType::from(page[25]);
    let metaflags = page[26];
    let _unused1 = page[27];

    let free = u32e(endian, &page[28..32]);
    let last_pgno = u32e(endian, &page[32..36]);
    let _unused3 = u32e(endian, &page[36..40]);
    let key_count = u32e(endian, &page[40..44]);
    let record_count = u32e(endian, &page[44..48]);
    let flags = u32e(endian, &page[48..52]);

    let mut uid = [0u8; 20];
    uid.copy_from_slice(&page[52..72]);

    let _unused_after_uid = u32e(endian, &page[72..76]);
    let minkey = u32e(endian, &page[76..80]);
    let re_len = u32e(endian, &page[80..84]);
    let re_pad = u32e(endian, &page[84..88]);
    let root = u32e(endian, &page[88..92]);

    // Tail (only if page is large enough)
    let crypto_magic = if page.len() >= 464 {
        u32e(endian, &page[460..464])
    } else {
        0
    };
    let mut iv = [0u8; 16];
    if page.len() >= 492 {
        iv.copy_from_slice(&page[476..492]);
    }
    let mut chksum = [0u8; 20];
    if page.len() >= 516 {
        chksum.copy_from_slice(&page[496..516]);
    }

    Ok(BtreeMeta {
        endian,
        lsn_file,
        lsn_offset,
        pgno,
        magic,
        version,
        pagesize,
        encrypt_alg,
        p_type,
        metaflags,
        _unused1,
        free,
        last_pgno,
        _unused3,
        key_count,
        record_count,
        flags,
        uid,
        _unused_after_uid,
        minkey,
        re_len,
        re_pad,
        root,
        crypto_magic,
        iv,
        chksum,
    })
}
