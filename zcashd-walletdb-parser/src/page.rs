use core::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageType {
    Meta,
    Internal,
    Leaf,
    Overflow,
    Other(u8),
}

impl From<u8> for PageType {
    fn from(t: u8) -> Self {
        match t {
            9 => PageType::Meta,
            3 => PageType::Internal,
            5 => PageType::Leaf,
            4 => PageType::Overflow,
            x => PageType::Other(x),
        }
    }
}

impl PageType {
    /// Return the canonical Berkeley DB type code.
    pub const fn code(self) -> u8 {
        match self {
            Self::Meta => 0x09,
            Self::Internal => 0x03,
            Self::Leaf => 0x02,
            Self::Overflow => 0x04,
            Self::Other(x) => x,
        }
    }

    /// Human-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Internal => "internal",
            Self::Leaf => "leaf",
            Self::Overflow => "overflow",
            Self::Other(_) => "other",
        }
    }

    /// Construct from the page flags field (type is low 5 bits).
    pub const fn from_flags(flags: u32) -> Self {
        match (flags & 0x1f) as u8 {
            0x09 => Self::Meta,
            0x03 => Self::Internal,
            0x02 => Self::Leaf,
            0x04 => Self::Overflow,
            x => Self::Other(x),
        }
    }
}

impl fmt::Display for PageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
