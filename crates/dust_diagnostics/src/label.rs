use dust_text::{FileId, TextRange};

/// A source-attached message describing where a diagnostic applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLabel {
    /// The source file this label refers to.
    pub file_id: FileId,
    /// The byte range this label highlights.
    pub range: TextRange,
    /// A short explanation for the highlighted region.
    pub message: String,
}

impl SourceLabel {
    /// Creates a new source label.
    pub fn new(file_id: FileId, range: TextRange, message: impl Into<String>) -> Self {
        Self {
            file_id,
            range,
            message: message.into(),
        }
    }
}
