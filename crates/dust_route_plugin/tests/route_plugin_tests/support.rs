use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, DartFileIr,
    NormalizedConfigIr, ParamKind, RouteConfigIr, RouterConfigIr, SpanIr, SymbolId, TypeIr,
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
    let mut route_config = config("AppRoute", Some(route_args));
    route_config.normalized = typed_route_config(&route_config).map(NormalizedConfigIr::Route);
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
        configs: vec![route_config],
        serde: None,
    }
}

fn typed_route_config(config: &ConfigApplicationIr) -> Option<RouteConfigIr> {
    Some(RouteConfigIr {
        path: config.positional_string(0)?,
        name: config.named_string("name"),
        result_type: config.named_type("result"),
        shell: config.named_type("shell"),
        branch: config.named_string("branch"),
        guards: config.named_type_list("guards").unwrap_or_default(),
        guards_configured: config.has_named_argument("guards"),
        transition: config.named_expression_source("transition"),
        fullscreen_dialog: config.named_bool("fullscreenDialog").unwrap_or(false),
        maintain_state: config.named_bool("maintainState").unwrap_or(true),
    })
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

pub(crate) fn shell_class(name: &str, params: Vec<ConstructorParamIr>) -> ClassIr {
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
        configs: Vec::new(),
        serde: None,
    }
}

pub(crate) fn positional_param(name: &str, ty: TypeIr) -> ConstructorParamIr {
    ConstructorParamIr {
        kind: ParamKind::Positional,
        ..constructor_param(name, ty)
    }
}

pub(crate) fn named_constructor_guard_class(name: &str) -> ClassIr {
    let mut guard = guard_class(name, Vec::new());
    guard.constructors[0].name = Some("create".to_owned());
    guard
}

pub(crate) fn router_class(args: &str) -> ClassIr {
    let mut router_config = config("AppRouter", Some(args));
    router_config.normalized = Some(NormalizedConfigIr::Router(RouterConfigIr {
        initial: router_config.named_string("initial"),
        not_found: router_config.named_string("notFound"),
    }));
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
        configs: vec![router_config],
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
    defaulted_param_source(name, ty, "1")
}

pub(crate) fn defaulted_param_source(
    name: &str,
    ty: TypeIr,
    default_value: &str,
) -> ConstructorParamIr {
    ConstructorParamIr {
        has_default: true,
        default_value_source: Some(default_value.to_owned()),
        ..constructor_param(name, ty)
    }
}

pub(crate) fn library_with_classes(mut classes: Vec<ClassIr>) -> DartFileIr {
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

    DartFileIr {
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
