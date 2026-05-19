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
    ConfigApplicationIr {
        symbol: SymbolId::new(format!("dust_router::{name}")),
        arguments_source: args.map(str::to_owned),
        span: span(1, 2),
    }
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
        configs: vec![config("Route", Some(route_args))],
        serde: None,
    }
}

pub(crate) fn router_class(args: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "AppRouter".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some("$AppRouter".to_owned()),
        span: span(10, 90),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![config("Router", Some(args))],
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

pub(crate) fn library_with_classes(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "route_test".to_owned(),
        source_path: "lib/route.dart".to_owned(),
        output_path: "lib/route.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(0, 100),
        classes,
        enums: Vec::new(),
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
    }
}

pub(crate) fn parsed_annotation(name: &str, args: &str) -> ParsedAnnotation {
    ParsedAnnotation {
        name: name.to_owned(),
        arguments_source: Some(args.to_owned()),
        span: TextRange::new(1_u32, 2_u32),
    }
}
