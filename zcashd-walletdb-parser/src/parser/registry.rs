use crate::parser::record::{RecordDecoder, RecordKind};

/// Registry that maps RecordKind -> decoder instance. Tokes ownership of decoders.
pub struct DecoderRegistry {
    // implementation detail: maps RecordKind to boxed decoders
}

impl DecoderRegistry {
    /// Register a decoder for a kind.
    pub fn register<D: RecordDecoder + 'static>(&mut self, kind: RecordKind, decoder: D) {
        todo!()
    }

    /// Lookup decoder for a kind.
    pub fn get(&self, kind: RecordKind) -> Option<&dyn RecordDecoder<Item = dyn std::any::Any>> {
        todo!()
    }
}
