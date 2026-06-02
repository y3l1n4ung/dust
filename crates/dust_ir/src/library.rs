use dust_text::{FileId, TextRange};

use crate::{ClassIr, QueryCallIr, enum_type::EnumIr};

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
    /// The package root that owns this library.
    pub package_root: String,
    /// The Dart package name that owns this library.
    pub package_name: String,
    /// The original source file path.
    pub source_path: String,
    /// The target generated file path.
    pub output_path: String,
    /// Imported library URIs preserved from the source library.
    pub imports: Vec<String>,
    /// The span covering the source library.
    pub span: SpanIr,
    /// The lowered classes in this library.
    pub classes: Vec<ClassIr>,
    /// The lowered enums in this library.
    pub enums: Vec<EnumIr>,
    /// The lowered DB query helper calls in this library.
    pub query_calls: Vec<QueryCallIr>,
}
