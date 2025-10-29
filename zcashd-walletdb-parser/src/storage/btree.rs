use std::io;

use crate::storage::{
    page::{EntryDescriptor, ValueSupplier},
    types::{ByteSlice, ByteVec, PageNumber},
};

/// Logical node representation.
pub(crate) enum Node<'a> {
    Internal {
        keys: Vec<ByteSlice<'a>>,
        children: Vec<PageNumber>,
    },
    Leaf {
        entries: Vec<EntryDescriptor>,
    },
}

/// The BTreeWalker knows how to walk the on-disk tree.
/// It is the only component that understands separator keys, child pointers, and root lookup.
pub(crate) trait BTreeWalker {
    /// In-order traversal yielding descriptors + a supplier that can materialize each value.
    /// The supplier must capture whatever is necessary (page buffer + source) to materialize lazily.
    fn walk_in_order<'s>(
        &'s self,
    ) -> Box<dyn Iterator<Item = (ByteVec, Box<dyn ValueSupplier>)> + 's>;

    /// Convenience: collect into a map eagerly (used in tests / simple clients).
    fn collect_map(&self) -> io::Result<std::collections::HashMap<ByteVec, ByteVec>>;
}
