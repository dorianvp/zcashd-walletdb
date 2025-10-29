use crate::{
    constants::BTDATAOFF,
    util::{Endian, u16e},
};

/// Reference to bytes stored off-page (overflow).
/// Start at `first_page` and read `total_len` bytes across a chain of P_OVERFLOW pages.
#[derive(Debug, Copy, Clone)]
pub struct OverflowRef {
    pub first_page: u32,
    pub total_len: u32,
}

/// A BLEAF field (key or value), either inline on the leaf page or off-page (overflow).
#[derive(Debug)]
pub enum Field<'a> {
    Inline(&'a [u8]),
    Overflow(OverflowRef),
}

/// Read u16 offsets from the slot array (28..lower).
pub fn iter_slots<'a>(page: &'a [u8], e: Endian, lower: u16) -> impl Iterator<Item = u16> + 'a {
    let lower = lower as usize;
    (BTDATAOFF..lower)
        .step_by(2)
        .map(move |i| u16e(e, &page[i..i + 2]))
}
