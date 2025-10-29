/// Start of the per-page header/slot area on BDB 4.x/5.x pages.
/// The slot array lives in [BTDATAOFF .. lower), and item payloads are in [upper .. page.len()).
pub const BTDATAOFF: usize = 28;
