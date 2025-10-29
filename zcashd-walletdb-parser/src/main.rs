use std::{env, fs, path::PathBuf, process};

use anyhow::Result;
use zcashd_walletdb_parser::{
    entry::parser::leaf_pairs_on_page,
    headers::parse_btree_meta_page0,
    page::PageType,
    util::{page_slice, parse_page_header},
};

fn main() -> Result<()> {
    let mut args = env::args_os();
    let prog = args.next().unwrap_or_default(); // program name

    // Expect exactly one positional argument: the wallet.dat path (or "-" for stdin)
    let path: PathBuf = match args.next() {
        Some(p) => p.into(),
        None => {
            eprintln!("usage: {} <wallet.dat | ->", prog.to_string_lossy());
            process::exit(2);
        }
    };

    // Optional: reject extra args
    if args.next().is_some() {
        eprintln!(
            "error: too many arguments\nusage: {} <wallet.dat | ->",
            prog.to_string_lossy()
        );
        process::exit(2);
    }

    let bytes = fs::read(path)?;

    // Grab page 0 using the largest plausible default (we’ll trim by pagesize after parsing)
    if bytes.len() < 512 {
        anyhow::bail!("file < 512 bytes");
    }
    // Parse directly from start of file (page 0):
    let meta = parse_btree_meta_page0(&bytes[..std::cmp::min(bytes.len(), 4096)])?;
    println!("{}", meta);

    let ps = meta.pagesize as usize;
    let endian = meta.endian;

    // Basic sanity
    let npages = bytes.len() / ps;
    assert_eq!(meta.pgno, 0, "page 0 should be pgno=0");
    assert!(
        npages >= (meta.last_pgno as usize + 1),
        "file shorter than last_pgno"
    );
    assert!(
        meta.root != 0 && meta.root <= meta.last_pgno,
        "root out of range"
    );

    // // Walk headers for all pages (skip meta 0)
    // for pg in 1..=meta.last_pgno {
    //     let page = page_slice(&bytes, ps, pg);
    //     let hdr = parse_page_header(page, endian)?;
    //     println!(
    //         "page {:>3}: type={} (code {:02x}) slots={} lower={} upper={} prev={} next={} flags=0x{:08x}",
    //         pg,
    //         hdr.ptype.as_str(),
    //         hdr.ptype.code(),
    //         hdr.nslots,
    //         hdr.lower,
    //         hdr.upper,
    //         hdr.prev,
    //         hdr.next,
    //         hdr.flags
    //     );
    // }

    let mut total = 0usize;
    for pg in 1..=meta.last_pgno {
        let page = page_slice(&bytes, ps, pg);
        let hdr = parse_page_header(page, endian)?;
        if matches!(hdr.ptype, PageType::Leaf) {
            let pairs = leaf_pairs_on_page(&bytes, ps, endian, page, &hdr)?;
            total += pairs.len();
            for (i, (k, v)) in pairs.iter().take(3).enumerate() {
                println!(
                    "page {pg} item {i}: key_len={} val_len={}",
                    k.len(),
                    v.len()
                );
            }
        }
    }
    println!("total kv pairs (incl. overflow) = {total}");

    Ok(())
}
