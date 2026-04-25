use std::ops::{Add, Sub};

/// A byte offset into UTF-8 source text.
///
/// Dust uses byte offsets because parser backends and generated spans are
/// naturally byte-based. Higher layers can convert these offsets into
/// line/column pairs through [`crate::LineIndex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TextSize(u32);

impl TextSize {
    /// Creates a new byte offset.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw `u32` offset.
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Returns the offset as `usize`.
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for TextSize {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<TextSize> for u32 {
    fn from(value: TextSize) -> Self {
        value.0
    }
}

impl From<usize> for TextSize {
    fn from(value: usize) -> Self {
        let raw = u32::try_from(value).expect("text size must fit into u32");
        Self(raw)
    }
}

impl Add for TextSize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_add(rhs.0).expect("text size overflow"))
    }
}

impl Sub for TextSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_sub(rhs.0).expect("text size underflow"))
    }
}

/// A half-open byte range inside UTF-8 source text.
///
/// The range includes `start` and excludes `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    /// Creates a new half-open range.
    ///
    /// Panics if `start > end`.
    pub fn new(start: impl Into<TextSize>, end: impl Into<TextSize>) -> Self {
        let start = start.into();
        let end = end.into();
        assert!(start <= end, "text range start must be <= end");
        Self { start, end }
    }

    /// Creates a new range from `start` and `len`.
    pub fn at(start: impl Into<TextSize>, len: impl Into<TextSize>) -> Self {
        let start = start.into();
        let len = len.into();
        Self::new(start, start + len)
    }

    /// Creates an empty range at one offset.
    pub fn empty(offset: impl Into<TextSize>) -> Self {
        let offset = offset.into();
        Self::new(offset, offset)
    }

    /// Returns the inclusive start offset.
    pub const fn start(self) -> TextSize {
        self.start
    }

    /// Returns the exclusive end offset.
    pub const fn end(self) -> TextSize {
        self.end
    }

    /// Returns the length in bytes.
    pub fn len(self) -> TextSize {
        self.end - self.start
    }

    /// Returns `true` if the range is empty.
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns `true` if the offset lies inside the range.
    pub fn contains(self, offset: impl Into<TextSize>) -> bool {
        let offset = offset.into();
        self.start <= offset && offset < self.end
    }

    /// Returns `true` if two ranges overlap.
    pub fn intersects(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Returns the smallest range that covers both ranges.
    pub fn cover(self, other: Self) -> Self {
        let start = if self.start <= other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end >= other.end {
            self.end
        } else {
            other.end
        };
        Self { start, end }
    }
}
