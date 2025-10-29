use std::{fmt::Debug, io};

use crate::storage::types::{ByteSlice, ByteVec, DbIndex, LogSequenceNumber, PageNumber};

/// The type of a BDB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    Meta,
    BtreeInternal,
    BtreeLeaf,
    Overflow,
    Unknown(u8),
}

/// The header of a BDB page.
#[derive(Debug, Clone)]
pub struct PageHeader {
    /// 00-07: Log sequence number (LSN)
    pub lsn: LogSequenceNumber,

    /// 12-15: Prev in page chain (overflow chain or sequential)
    pub prev_pgno: PageNumber,

    /// 16-19: Next in page chain
    pub next_pgno: PageNumber,

    /// 20-21: Number of slot entries (the slot array length)
    pub entries: DbIndex,

    /// 22-23: High free offset (hf_offset) — start of packed data region.
    /// This is the authoritative "upper" boundary for data stored in this page.
    pub hf_offset: DbIndex,

    /// 24: level (1 = leaf)
    pub level: u8,

    /// 25: on-disk page type value
    pub page_type: u8,

    /// Optional flags, checksum, or extended metadata.
    /// Not all builds enable checksums or the same flags.
    pub flags: Option<u32>,
    pub checksum: Option<u32>,
}

impl PageHeader {
    /// Derived: number of slots as usize
    pub fn num_slots(&self) -> usize {
        todo!()
    }

    /// Derived: the lower boundary (end of header + slot array) in bytes,
    /// computed from header size and `entries`.
    pub fn lower_bound(&self, header_size: usize, slot_entry_size: usize) -> usize {
        todo!()
    }

    /// Derived: the upper boundary in bytes (hf_offset as usize).
    pub fn upper_bound(&self) -> usize {
        todo!()
    }
}

/// A BDB page.
#[derive(Debug)]
pub struct Page {
    pub header: PageHeader,
    pub raw: ByteVec,
}

/// Describes a slot/entry inside a page without materializing the value.
#[derive(Debug, Clone)]
pub struct EntryDescriptor {
    pub slot_index: u16,
    pub key_len: usize,
    pub value_len: usize,
    pub flags: u8,
    pub key_range: (usize, usize),
    pub value_range: (usize, usize),
}

/// Encapsulates the ability to materialize a blob for an entry.
/// Implementations may capture references into a page buffer and a PageSource for overflow follow-ups.
pub trait ValueSupplier: Send + Sync + Debug {
    /// Materialize the full value bytes. Follows overflow references.
    fn materialize(&self) -> io::Result<ByteVec>;

    /// Attempt a zero-copy borrow if the value is fully contained in-memory.
    /// Returns None if borrowing is not possible or value requires concatenation.
    fn try_borrow<'a>(&'a self) -> Option<ByteSlice<'a>>;
}
