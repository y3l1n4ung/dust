use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, FieldIr, LibraryIr, SymbolId,
    TraitApplicationIr, TypeIr,
};

use crate::support::span;

pub(super) fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/model.dart".to_owned(),
        output_path: "lib/model.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(0, 100),
        classes,
        enums: Vec::new(),
        query_calls: Vec::new(),
    }
}

pub(super) fn class(name: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 80),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(40, 60),
            params: Vec::new(),
        }],
        methods: Vec::new(),
        traits: vec![TraitApplicationIr {
            symbol: SymbolId::new("dust_dart::Validate"),
            span: span(5, 9),
        }],
        configs: Vec::new(),
        serde: None,
    }
}

pub(super) fn field(name: &str, ty: TypeIr, configs: Vec<ConfigApplicationIr>) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        has_default: false,
        serde: None,
        configs,
    }
}

pub(super) fn validate(arguments: &str) -> ConfigApplicationIr {
    ConfigApplicationIr {
        symbol: SymbolId::new("dust_dart::Validate"),
        arguments_source: Some(arguments.to_owned()),
        span: span(20, 30),
    }
}
