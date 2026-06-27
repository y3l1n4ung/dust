#![cfg(test)]

use super::lower_library;
use dust_ir::{ClassKindIr, ConfigApplicationIr, SpanIr, SymbolId, TypeIr};
use dust_parser_dart::{
    ParameterKind, ParsedExtensionSurface, ParsedExtensionTypeSurface, ParsedFieldSurface,
    ParsedFunctionSurface, ParsedMethodParamSurface, ParsedMethodSurface, ParsedMixinSurface,
    ParsedTopLevelVariableSurface, ParsedTypeSurface, ParsedTypedefSurface,
};
use dust_resolver::{ResolvedClass, ResolvedLibrary, ResolvedMethod, ResolvedMethodParam};
use dust_text::{FileId, TextRange};

fn range(start: u32, end: u32) -> TextRange {
    TextRange::new(start, end)
}

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(77), range(start, end))
}

fn parsed_type(source: &str, start: u32) -> ParsedTypeSurface {
    ParsedTypeSurface::parse(source, range(start, start + source.len() as u32)).unwrap()
}

fn config(name: &str) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new(format!("dust_dart::{name}")),
        None,
        span(1, 2),
    )
}

#[test]
fn lowers_parser_top_level_declaration_surfaces_into_dart_file_ir() {
    let user_type = parsed_type("User", 10);
    let string_type = parsed_type("String", 20);
    let list_string_type = parsed_type("List<String>", 30);
    let map_type = parsed_type("Map<String, User>", 40);

    let library = ResolvedLibrary {
        source_path: "lib/declarations.dart".to_owned(),
        output_path: "lib/declarations.g.dart".to_owned(),
        span: span(0, 400),
        directives: Vec::new(),
        part_uri: None,
        classes: Vec::new(),
        enums: Vec::new(),
        mixins: vec![ParsedMixinSurface {
            name: "Auditable".to_owned(),
            annotations: Vec::new(),
            fields: vec![ParsedFieldSurface {
                name: "auditId".to_owned(),
                annotations: Vec::new(),
                type_source: Some("String".to_owned()),
                parsed_type: Some(string_type.clone()),
                has_default: false,
                span: range(50, 65),
            }],
            span: range(45, 80),
        }],
        extensions: vec![ParsedExtensionSurface {
            name: Some("UserFormatting".to_owned()),
            on_type_source: Some("User".to_owned()),
            parsed_on_type: Some(user_type.clone()),
            annotations: Vec::new(),
            span: range(90, 140),
        }],
        extension_types: vec![ParsedExtensionTypeSurface {
            name: "UserId".to_owned(),
            representation_name: "value".to_owned(),
            representation_type_source: Some("String".to_owned()),
            parsed_representation_type: Some(string_type.clone()),
            annotations: Vec::new(),
            span: range(150, 190),
        }],
        functions: vec![ParsedFunctionSurface {
            name: "greet".to_owned(),
            return_type_source: Some("String".to_owned()),
            parsed_return_type: Some(string_type.clone()),
            params: vec![ParsedMethodParamSurface {
                name: "user".to_owned(),
                annotations: Vec::new(),
                type_source: Some("User".to_owned()),
                parsed_type: Some(user_type.clone()),
                kind: ParameterKind::Positional,
                is_required: false,
                has_default: false,
                default_value_source: None,
                span: range(205, 214),
            }],
            annotations: Vec::new(),
            span: range(195, 230),
        }],
        variables: vec![ParsedTopLevelVariableSurface {
            name: "names".to_owned(),
            type_source: Some("List<String>".to_owned()),
            parsed_type: Some(list_string_type.clone()),
            initializer_source: Some("const ['a']".to_owned()),
            initializer_span: Some(range(260, 271)),
            annotations: Vec::new(),
            span: range(240, 272),
        }],
        typedefs: vec![ParsedTypedefSurface {
            name: "UserMap".to_owned(),
            aliased_type_source: Some("Map<String, User>".to_owned()),
            parsed_aliased_type: Some(map_type),
            annotations: Vec::new(),
            span: range(280, 320),
        }],
        query_calls: Vec::new(),
    };

    let outcome = lower_library(&library);

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let file = outcome.value;
    assert_eq!(file.mixins[0].name.source, "Auditable");
    assert_eq!(file.mixins[0].fields[0].name, "auditId");
    assert_eq!(file.mixins[0].fields[0].ty, TypeIr::string());
    assert_eq!(
        file.extensions[0].name.as_ref().unwrap().source,
        "UserFormatting"
    );
    assert_eq!(file.extensions[0].on_type, TypeIr::named("User"));
    assert_eq!(file.extension_types[0].name.source, "UserId");
    assert_eq!(file.extension_types[0].representation.name, "value");
    assert_eq!(file.extension_types[0].representation.ty, TypeIr::string());
    assert_eq!(file.functions[0].name.source, "greet");
    assert_eq!(file.functions[0].return_type, TypeIr::string());
    assert_eq!(file.functions[0].params[0].ty, TypeIr::named("User"));
    assert_eq!(file.variables[0].name.source, "names");
    assert_eq!(
        file.variables[0]
            .initializer
            .as_ref()
            .map(|initializer| initializer.source.as_str()),
        Some("const ['a']")
    );
    assert_eq!(
        file.variables[0].ty,
        TypeIr::generic("List", vec![TypeIr::string()])
    );
    assert_eq!(file.typedefs[0].name.source, "UserMap");
    assert_eq!(
        file.typedefs[0].aliased_type,
        TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::named("User")])
    );
}

#[test]
fn lowers_explicit_required_method_parameter_state() {
    let param = ParsedMethodParamSurface {
        name: "traceId".to_owned(),
        annotations: Vec::new(),
        type_source: Some("String?".to_owned()),
        parsed_type: Some(parsed_type("String?", 20)),
        kind: ParameterKind::Named,
        is_required: true,
        has_default: false,
        default_value_source: None,
        span: range(20, 35),
    };
    let method = ParsedMethodSurface {
        name: "search".to_owned(),
        is_static: false,
        is_external: false,
        annotations: Vec::new(),
        return_type_source: Some("void".to_owned()),
        parsed_return_type: Some(parsed_type("void", 10)),
        has_body: false,
        body_source: None,
        params: vec![param.clone()],
        span: range(10, 50),
    };
    let library = ResolvedLibrary {
        source_path: "lib/api.dart".to_owned(),
        output_path: "lib/api.g.dart".to_owned(),
        span: span(0, 100),
        directives: Vec::new(),
        part_uri: None,
        classes: vec![ResolvedClass {
            kind: ClassKindIr::Class,
            name: "Api".to_owned(),
            is_abstract: true,
            is_interface: true,
            superclass_name: None,
            span: span(0, 100),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: vec![ResolvedMethod {
                surface: method,
                span: span(10, 50),
                traits: Vec::new(),
                configs: Vec::new(),
                params: vec![ResolvedMethodParam {
                    surface: param,
                    span: span(20, 35),
                    traits: Vec::new(),
                    configs: Vec::new(),
                }],
            }],
            traits: Vec::new(),
            configs: vec![config("HttpClient")],
        }],
        enums: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    };

    let outcome = lower_library(&library);

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    assert!(outcome.value.classes[0].methods[0].params[0].is_required);
}
