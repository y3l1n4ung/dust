use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, EnumIr, EnumVariantIr, FieldIr, LibraryIr, SpanIr,
    SymbolId, TypeIr,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn config(args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new("dust_flutter::ViewModel"),
        args.map(str::to_owned),
        span(1, 2),
    )
}

pub(crate) fn view_model_class(name: &str, args: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some(format!("${name}")),
        span: span(10, 90),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![config(Some(args))],
        serde: None,
    }
}

pub(crate) fn args_class() -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "TaskBoardArgs".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some("ViewModelArgs".to_owned()),
        span: span(10, 90),
        fields: vec![FieldIr {
            name: "repository".to_owned(),
            ty: TypeIr::named("PrototypeRepository"),
            span: span(20, 30),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        }],
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    }
}

pub(crate) fn state_class() -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "TaskBoardState".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 90),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    }
}

pub(crate) fn library_with_classes(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "state_test".to_owned(),
        source_path: "lib/task_board_view_model.dart".to_owned(),
        output_path: "lib/task_board_view_model.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 100),
        classes,
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

pub(crate) fn enum_type(name: &str, variants: &[&str]) -> EnumIr {
    EnumIr {
        name: name.to_owned(),
        span: span(10, 90),
        variants: variants
            .iter()
            .map(|variant| EnumVariantIr {
                name: (*variant).to_owned(),
                serde: None,
                span: span(20, 30),
            })
            .collect(),
        traits: Vec::new(),
        serde: None,
    }
}

pub(crate) fn library_with_classes_and_enums(
    classes: Vec<ClassIr>,
    enums: Vec<EnumIr>,
) -> LibraryIr {
    LibraryIr {
        enums,
        ..library_with_classes(classes)
    }
}
