use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, LibraryIr,
    ParamKind, SpanIr, SymbolId, TypeIr,
};
use dust_parser_dart::{
    ParsedAnnotation, ParsedClassKind, ParsedClassSurface, ParsedLibrarySurface,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn config(name: &str, args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new(format!("dust_flutter::{name}")),
        args.map(str::to_owned),
        span(1, 2),
    )
}

pub(crate) fn route_page_class(
    name: &str,
    route_args: &str,
    params: Vec<ConstructorParamIr>,
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some("StatelessWidget".to_owned()),
        span: span(10, 90),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(12, 18),
            params,
        }],
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![config("AppRoute", Some(route_args))],
        serde: None,
    }
}

pub(crate) fn guard_class(name: &str, params: Vec<ConstructorParamIr>) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 90),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(12, 18),
            params,
        }],
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    }
}

pub(crate) fn named_constructor_guard_class(name: &str) -> ClassIr {
    let mut guard = guard_class(name, Vec::new());
    guard.constructors[0].name = Some("create".to_owned());
    guard
}

pub(crate) fn router_class(args: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "TestRouter".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some("$TestRouter".to_owned()),
        span: span(10, 90),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![config("AppRouter", Some(args))],
        serde: None,
    }
}

pub(crate) fn constructor_param(name: &str, ty: TypeIr) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        kind: ParamKind::Named,
        has_default: false,
        default_value_source: None,
    }
}

pub(crate) fn defaulted_param(name: &str, ty: TypeIr) -> ConstructorParamIr {
    ConstructorParamIr {
        has_default: true,
        default_value_source: Some("1".to_owned()),
        ..constructor_param(name, ty)
    }
}

pub(crate) fn library_with_classes(mut classes: Vec<ClassIr>) -> LibraryIr {
    if classes.iter().any(|class| {
        class
            .configs
            .iter()
            .any(|config| config.symbol.0.ends_with("::AppRouter"))
    }) && !classes.iter().any(|class| class.name == "NotFoundPage")
    {
        classes.push(route_page_class(
            "NotFoundPage",
            "('/404', name: 'notFound', guards: [])",
            vec![string_default_param("path", "''")],
        ));
    }

    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "route_test".to_owned(),
        source_path: "lib/route.dart".to_owned(),
        output_path: "lib/route.g.dart".to_owned(),
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

fn string_default_param(name: &str, default_value: &str) -> ConstructorParamIr {
    ConstructorParamIr {
        has_default: true,
        default_value_source: Some(default_value.to_owned()),
        ..constructor_param(name, TypeIr::string())
    }
}

pub(crate) fn parsed_library_with_annotations(
    class_name: &str,
    annotations: Vec<ParsedAnnotation>,
) -> ParsedLibrarySurface {
    ParsedLibrarySurface {
        span: TextRange::new(0_u32, 100_u32),
        directives: Vec::new(),
        classes: vec![ParsedClassSurface {
            kind: ParsedClassKind::Class,
            name: class_name.to_owned(),
            is_abstract: false,
            is_interface: false,
            superclass_name: None,
            annotations,
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            span: TextRange::new(10_u32, 90_u32),
        }],
        enums: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    }
}

pub(crate) fn parsed_annotation(name: &str, args: &str) -> ParsedAnnotation {
    parsed_qualified_annotation(None, name, args)
}

pub(crate) fn parsed_prefixed_annotation(prefix: &str, name: &str, args: &str) -> ParsedAnnotation {
    parsed_qualified_annotation(Some(prefix), name, args)
}

fn parsed_qualified_annotation(prefix: Option<&str>, name: &str, args: &str) -> ParsedAnnotation {
    ParsedAnnotation {
        name: name.to_owned(),
        prefix: prefix.map(str::to_owned),
        qualified_name: prefix
            .map(|prefix| format!("{prefix}.{name}"))
            .unwrap_or_else(|| name.to_owned()),
        arguments_source: Some(args.to_owned()),
        parsed_arguments: None,
        span: TextRange::new(1_u32, 2_u32),
    }
}
