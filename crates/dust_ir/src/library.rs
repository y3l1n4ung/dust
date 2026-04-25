use dust_text::{FileId, TextRange};

use crate::ClassIr;

/// A file-backed source span stored in the semantic IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanIr {
    /// The source file identifier.
    pub file_id: FileId,
    /// The byte range inside that file.
    pub range: TextRange,
}

impl SpanIr {
    /// Creates a new semantic span.
    pub const fn new(file_id: FileId, range: TextRange) -> Self {
        Self { file_id, range }
    }
}

/// One lowered Dart library ready for plugin validation and emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryIr {
    /// The original source file path.
    pub source_path: String,
    /// The target generated file path.
    pub output_path: String,
    /// The span covering the source library.
    pub span: SpanIr,
    /// The lowered classes in this library.
    pub classes: Vec<ClassIr>,
}
