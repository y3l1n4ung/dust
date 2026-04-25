use std::sync::Arc;

use crate::{FileId, LineCol, LineIndex, TextRange, TextSize};

/// Owns one source file's text and its precomputed line index.
#[derive(Debug, Clone)]
pub struct SourceText {
    file_id: FileId,
    text: Arc<str>,
    line_index: LineIndex,
}

impl SourceText {
    /// Creates a new source container.
    pub fn new(file_id: FileId, text: impl Into<Arc<str>>) -> Self {
        let text = text.into();
        let line_index = LineIndex::new(&text);

        Self {
            file_id,
            text,
            line_index,
        }
    }

    /// Returns the owning file identifier.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Returns the full source as a string slice.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Returns the total byte length.
    pub fn len(&self) -> TextSize {
        TextSize::from(self.text.len())
    }

    /// Returns `true` if the source is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the range covering the entire source.
    pub fn full_range(&self) -> TextRange {
        TextRange::new(TextSize::new(0), self.len())
    }

    /// Returns the line index for this source.
    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    /// Returns a UTF-8 slice for the given byte range.
    pub fn slice(&self, range: TextRange) -> Option<&str> {
        self.text
            .get(range.start().to_usize()..range.end().to_usize())
    }

    /// Converts a byte offset into a line/column pair.
    pub fn line_col(&self, offset: impl Into<TextSize>) -> Option<LineCol> {
        self.line_index.line_col(offset)
    }
}
