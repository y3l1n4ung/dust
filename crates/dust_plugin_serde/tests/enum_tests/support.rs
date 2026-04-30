use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, EnumVariantIr, FieldIr,
    LibraryIr, ParamKind, SerdeClassConfigIr, SerdeRenameRuleIr, SpanIr, SymbolId,
    TraitApplicationIr, TypeIr,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

pub(crate) fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

pub(crate) fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
    }
}

pub(crate) fn enum_variant(name: &str) -> EnumVariantIr {
    EnumVariantIr {
        name: name.to_owned(),
        span: span(10, 20),
    }
}

pub(crate) fn enum_ir(name: &str, variants: Vec<EnumVariantIr>, traits: &[&str]) -> EnumIr {
    EnumIr {
        name: name.to_owned(),
        span: span(0, 100),
        variants,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

pub(crate) fn renamed_enum(
    name: &str,
    variants: Vec<EnumVariantIr>,
    rule: SerdeRenameRuleIr,
) -> EnumIr {
    let mut value = enum_ir(
        name,
        variants,
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );
    value.serde = Some(SerdeClassConfigIr {
        rename_all: Some(rule),
        ..Default::default()
    });
    value
}

pub(crate) fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind,
        has_default: false,
    }
}

pub(crate) fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        span: span(25, 60),
        params,
    }
}

pub(crate) fn class(
    name: &str,
    fields: Vec<FieldIr>,
    constructors: Vec<ConstructorIr>,
    traits: &[&str],
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

pub(crate) fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        span: span(0, 200),
        classes,
        enums,
    }
}
