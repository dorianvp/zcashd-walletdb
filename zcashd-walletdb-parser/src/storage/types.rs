use std::{borrow::Cow, fmt::Debug, io};

#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    Little,
    Big,
}

/// A page number in the BDB storage format.
/// This represents the `pgno` field of a page header.
pub type PageNumber = u32;

pub type DbIndex = u16;

pub type LogSequenceNumber = u64;

/// The size of a BDB page.
pub type PageSize = u32;

pub type ByteVec = Vec<u8>;

pub type ByteSlice<'a> = Cow<'a, [u8]>;

/// Low-level source of pages.
pub trait PageSource: Debug + Send + Sync {
    /// Read a single page by page number. Returns the raw bytes.
    fn read_page(&self, page_no: PageNumber) -> io::Result<ByteVec>;

    /// Total number of pages. The `Option` is to allow for streaming.
    fn page_count(&self) -> Option<u64>;

    /// Get path or source identifier (for provenance / logging).
    fn source_id(&self) -> String;
}

/// Represents the format of the BDB storage.
#[derive(Debug, Clone)]
pub struct FormatProfile {
    pub page_size: PageSize,
    pub endianness: Endianness,
    pub btree_root: PageNumber,
    pub berkeley_db_version: Option<String>,
}
