use dust_ir::{ClassIr, ClassKindIr, LibraryIr, SpanIr};
use dust_text::{FileId, TextRange};

pub(crate) fn sample_library() -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            is_interface: false,
            superclass_name: None,
            span: span(10, 80),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
            serde: None,
        }],
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

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}
