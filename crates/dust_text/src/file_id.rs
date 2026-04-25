/// Identifies a source file inside one Dust pipeline run.
///
/// `FileId` is intentionally small and copyable so higher layers can
/// attach spans and diagnostics without carrying full path strings
/// through every data structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FileId(u32);

impl FileId {
    /// Creates a new file identifier from a raw numeric value.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the underlying numeric identifier.
    pub const fn raw(self) -> u32 {
        self.0
    }
}
