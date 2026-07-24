use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, DartFileIr, EnumIr, EnumVariantIr, FieldIr,
    MethodIr, NormalizedConfigIr, SpanIr, StateConfigIr, StateModeIr, SymbolId, TraitApplicationIr,
    TypeIr,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn config(args: Option<&str>) -> ConfigApplicationIr {
    let mut config = ConfigApplicationIr::new(
        SymbolId::new("dust_flutter::ViewModel"),
        args.map(str::to_owned),
        span(1, 2),
    );
    if let Some(state_type) = config
        .named_type("state")
        .or_else(|| config.positional_type(0))
    {
        let mode_source = config.named_expression_source("mode");
        config.normalized = Some(NormalizedConfigIr::State(StateConfigIr {
            state_type,
            args_type: config.named_type("args"),
            initial_source: config.named_expression_source("initial"),
            mode_source: mode_source.clone(),
            mode: match mode_source.as_deref() {
                Some(source) if source.ends_with(".async") || source == "async" => {
                    StateModeIr::Async
                }
                _ => StateModeIr::Sync,
            },
        }));
    }
    config
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

pub(crate) fn state_class_with_eq() -> ClassIr {
    let mut state = state_class();
    state.traits.push(TraitApplicationIr {
        symbol: SymbolId::new("dust_dart::Eq"),
        span: span(20, 30),
    });
    state
}

pub(crate) fn operator_eq_method() -> MethodIr {
    MethodIr {
        name: "==".to_owned(),
        is_static: false,
        is_external: false,
        return_type: TypeIr::named("bool"),
        has_body: true,
        body_source: Some("=> identical(this, other);".to_owned()),
        params: Vec::new(),
        span: span(20, 40),
        traits: Vec::new(),
        configs: Vec::new(),
    }
}

pub(crate) fn library_with_classes(classes: Vec<ClassIr>) -> DartFileIr {
    DartFileIr {
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
) -> DartFileIr {
    DartFileIr {
        enums,
        ..library_with_classes(classes)
    }
}
