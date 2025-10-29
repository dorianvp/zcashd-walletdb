use crate::storage::types::{ByteVec, PageNumber};

/// Provenance metadata for debugging or salvaging.
#[derive(Debug, Clone)]
pub struct Provenance {
    pub source_id: String,
    pub page_no: PageNumber,
    pub slot_index: u16,
}

/// Map entry stored in-memory. Value may be owned or materialized lazily.
#[derive(Debug)]
pub struct MapEntry {
    pub key: ByteVec,
    pub value: ByteVec,
    pub meta: Option<Provenance>,
}

/// Primary in-memory container interface. Implementation may be backed by HashMap or BTreeMap.
pub trait InMemoryMap {
    /// Insert an owned pair. Returns previous value if present.
    fn insert(
        &mut self,
        key: ByteVec,
        value: ByteVec,
        provenance: Option<Provenance>,
    ) -> Option<MapEntry>;

    /// Get a borrowed reference to a value.
    fn get(&self, key: &[u8]) -> Option<&ByteVec>;

    /// Iterate owned entries (for decoding to domain objects).
    fn iter(&self) -> Box<dyn Iterator<Item = (&ByteVec, &MapEntry)> + '_>;

    /// Number of entries.
    fn len(&self) -> usize;
}
