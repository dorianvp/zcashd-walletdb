use std::fmt::Debug;

/// High-level kind inferred from the raw key bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    // TODO: Here would go the registered records
}

/// Classifies raw keys into RecordKind with optional parsed key metadata.
pub trait RecordClassifier {
    /// Inspect raw key bytes and return kind and optional structured key info.
    fn classify(&self, key: &[u8]) -> (RecordKind, Option<String>);
}

/// Decoder result type for domain objects. Keep domain types opaque to parser module.
pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug)]
pub struct DecodeError {
    pub message: String,
}

/// Decoder trait for converting raw value bytes into a typed domain object.
pub trait RecordDecoder: Send + Sync {
    type Item: Send + Sync + Debug;

    /// Decode bytes into a typed domain object.
    fn decode(&self, raw_value: &[u8]) -> DecodeResult<Self::Item>;

    /// Human-readable name for the decoder.
    fn name(&self) -> &'static str;
}
