use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, FieldIr,
    LibraryIr, MethodIr, MethodParamIr, ParamKind, SpanIr, SymbolId, TypeIr,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn config(name: &str, args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr {
        symbol: SymbolId::new(format!("dust_http_client_annotation::{name}")),
        arguments_source: args.map(str::to_owned),
        span: span(1, 2),
    }
}

pub(crate) fn param(name: &str, ty: TypeIr, configs: Vec<ConfigApplicationIr>) -> MethodParamIr {
    MethodParamIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        kind: ParamKind::Positional,
        has_default: false,
        default_value_source: None,
        traits: Vec::new(),
        configs,
    }
}

pub(crate) fn named_param(
    name: &str,
    ty: TypeIr,
    configs: Vec<ConfigApplicationIr>,
) -> MethodParamIr {
    MethodParamIr {
        kind: ParamKind::Named,
        ..param(name, ty, configs)
    }
}

pub(crate) fn method(
    name: &str,
    return_type: TypeIr,
    configs: Vec<ConfigApplicationIr>,
    params: Vec<MethodParamIr>,
) -> MethodIr {
    MethodIr {
        name: name.to_owned(),
        is_static: false,
        is_external: false,
        return_type,
        has_body: false,
        params,
        span: span(40, 70),
        traits: Vec::new(),
        configs,
        body_source: None,
    }
}

pub(crate) fn http_client_class(
    class_configs: Vec<ConfigApplicationIr>,
    methods: Vec<MethodIr>,
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "Api".to_owned(),
        is_abstract: true,
        is_interface: true,
        superclass_name: None,
        span: span(10, 90),
        fields: Vec::new(),
        constructors: vec![factory_constructor()],
        methods,
        traits: Vec::new(),
        configs: class_configs,
        serde: None,
    }
}

pub(crate) fn library_for(class: ClassIr) -> LibraryIr {
    library_with_classes(vec![class])
}

pub(crate) fn library_for_with_imports(class: ClassIr, imports: Vec<&str>) -> LibraryIr {
    library_with_classes_and_imports(vec![class], imports)
}

pub(crate) fn library_with_classes(classes: Vec<ClassIr>) -> LibraryIr {
    library_with_classes_and_imports(classes, Vec::new())
}

pub(crate) fn library_with_classes_and_imports(
    classes: Vec<ClassIr>,
    imports: Vec<&str>,
) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/api.dart".to_owned(),
        output_path: "lib/api.g.dart".to_owned(),
        imports: imports.into_iter().map(str::to_owned).collect(),
        span: span(0, 100),
        classes,
        enums: Vec::new(),
        query_calls: Vec::new(),
    }
}

pub(crate) fn future_of(inner: TypeIr) -> TypeIr {
    TypeIr::generic("Future", vec![inner])
}

fn factory_constructor() -> ConstructorIr {
    ConstructorIr {
        name: None,
        is_factory: true,
        redirected_target_source: Some("_$Api".to_owned()),
        redirected_target_name: Some("_$Api".to_owned()),
        span: span(12, 18),
        params: vec![
            ConstructorParamIr {
                name: "dio".to_owned(),
                ty: TypeIr::named("Dio"),
                span: span(13, 14),
                kind: ParamKind::Positional,
                has_default: false,
                default_value_source: None,
            },
            ConstructorParamIr {
                name: "baseUrl".to_owned(),
                ty: TypeIr::string().nullable(),
                span: span(14, 15),
                kind: ParamKind::Named,
                has_default: false,
                default_value_source: None,
            },
        ],
    }
}

pub(crate) fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(15, 16),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    }
}

pub(crate) fn serde_model_class(name: &str, fields: Vec<FieldIr>) -> ClassIr {
    let constructor_params = fields
        .iter()
        .map(|field| ConstructorParamIr {
            name: field.name.clone(),
            ty: field.ty.clone(),
            span: span(16, 17),
            kind: ParamKind::Named,
            has_default: field.has_default,
            default_value_source: None,
        })
        .collect();

    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 90),
        fields,
        constructors: vec![
            ConstructorIr {
                name: None,
                is_factory: false,
                redirected_target_source: None,
                redirected_target_name: None,
                span: span(12, 18),
                params: constructor_params,
            },
            ConstructorIr {
                name: Some("fromJson".to_owned()),
                is_factory: true,
                redirected_target_source: None,
                redirected_target_name: None,
                span: span(18, 24),
                params: vec![ConstructorParamIr {
                    name: "json".to_owned(),
                    ty: TypeIr::map_of(TypeIr::string(), TypeIr::object().nullable()),
                    span: span(18, 19),
                    kind: ParamKind::Positional,
                    has_default: false,
                    default_value_source: None,
                }],
            },
        ],
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    }
}
