use dust_text::{FileId, TextRange};

use crate::{
    AnnotationIr, ClassIr, ExportIr, ExtensionIr, ExtensionTypeIr, FunctionIr, ImportIr,
    LibraryDeclIr, MixinIr, PartIr, PartOfIr, QueryCallIr, TopLevelVariableIr, TypedefIr,
    enum_type::EnumIr,
};

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

/// One canonical Dart file IR ready for plugin validation and emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DartFileIr {
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
    /// The optional Dart `library` directive.
    pub library: Option<LibraryDeclIr>,
    /// Metadata annotations attached to the library directive or file.
    pub library_annotations: Vec<AnnotationIr>,
    /// Full import directives preserved from the source file.
    pub import_directives: Vec<ImportIr>,
    /// Full export directives preserved from the source file.
    pub export_directives: Vec<ExportIr>,
    /// Full part directives preserved from the source file.
    pub part_directives: Vec<PartIr>,
    /// The optional `part of` directive when this file is a part file.
    pub part_of: Option<PartOfIr>,
    /// The span covering the source library.
    pub span: SpanIr,
    /// The lowered classes in this library.
    pub classes: Vec<ClassIr>,
    /// The lowered mixins in this library.
    pub mixins: Vec<MixinIr>,
    /// The lowered extensions in this library.
    pub extensions: Vec<ExtensionIr>,
    /// The lowered extension types in this library.
    pub extension_types: Vec<ExtensionTypeIr>,
    /// The lowered top-level functions in this library.
    pub functions: Vec<FunctionIr>,
    /// The lowered top-level variables in this library.
    pub variables: Vec<TopLevelVariableIr>,
    /// The lowered typedefs in this library.
    pub typedefs: Vec<TypedefIr>,
    /// The lowered enums in this library.
    pub enums: Vec<EnumIr>,
    /// The lowered DB query helper calls in this library.
    pub query_calls: Vec<QueryCallIr>,
}

impl DartFileIr {
    /// Creates an empty Dart file IR with all additive parser surfaces initialized.
    pub fn empty(
        package_root: impl Into<String>,
        package_name: impl Into<String>,
        source_path: impl Into<String>,
        output_path: impl Into<String>,
        span: SpanIr,
    ) -> Self {
        Self {
            package_root: package_root.into(),
            package_name: package_name.into(),
            source_path: source_path.into(),
            output_path: output_path.into(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span,
            classes: Vec::new(),
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }
}

/// Compatibility alias while the migration from `LibraryIr` to `DartFileIr` is in progress.
pub type LibraryIr = DartFileIr;
