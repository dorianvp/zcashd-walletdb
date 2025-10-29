use std::io;

use crate::storage::{
    entry::{InMemoryMap, Provenance},
    page::ValueSupplier,
    types::{ByteVec, FormatProfile},
};

/// Modes controlling how aggressively we read a possibly-dirty DB image.
#[derive(Debug, Clone, Copy)]
pub enum SalvageMode {
    Conservative,
    BestEffort,
}

/// Exposes high-level operations that drive the parsing pipeline.
pub trait DbImageReader {
    /// Probe meta page and produce a format profile.
    fn probe(&self) -> io::Result<FormatProfile>;

    /// Produce an iterator of raw (key, value_supplier, provenance) from the file.
    /// This is the canonical entry point used by clients that want raw key->value pairs.
    fn entries<'s>(
        &'s self,
        salvage: SalvageMode,
    ) -> Box<dyn Iterator<Item = (ByteVec, Box<dyn ValueSupplier>, Provenance)> + 's>;

    /// Build an in-memory map eagerly using the entries iterator.
    fn build_map(&self, salvage: SalvageMode) -> io::Result<Box<dyn InMemoryMap>>;
}
