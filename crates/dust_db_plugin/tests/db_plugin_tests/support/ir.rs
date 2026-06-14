use dust_ir::{
    ConfigApplicationIr, ConstructorParamIr, FieldIr, MethodParamIr, ParamKind, SpanIr, SymbolId,
    TraitApplicationIr, TypeIr,
};
use dust_text::{FileId, TextRange};

pub(crate) fn span() -> SpanIr {
    SpanIr::new(FileId::new(7), TextRange::new(0_u32, 1_u32))
}

pub(crate) fn config(symbol: &str, args: &str) -> ConfigApplicationIr {
    ConfigApplicationIr::new(SymbolId::new(symbol), Some(args.to_owned()), span())
}

pub(crate) fn trait_app(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(),
    }
}

pub(crate) fn field(name: &str, ty: TypeIr, configs: Vec<ConfigApplicationIr>) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(),
        has_default: false,
        serde: None,
        configs,
    }
}

pub(crate) fn named_param(name: &str, ty: TypeIr, has_default: bool) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(),
        kind: ParamKind::Named,
        has_default,
        default_value_source: None,
    }
}

pub(crate) fn method_param(name: &str, ty: TypeIr) -> MethodParamIr {
    MethodParamIr {
        name: name.to_owned(),
        ty,
        span: span(),
        kind: ParamKind::Positional,
        has_default: false,
        default_value_source: None,
        traits: Vec::new(),
        configs: Vec::new(),
    }
}

pub(crate) fn named_method_param(name: &str, ty: TypeIr, has_default: bool) -> MethodParamIr {
    MethodParamIr {
        name: name.to_owned(),
        ty,
        span: span(),
        kind: ParamKind::Named,
        has_default,
        default_value_source: has_default.then(|| "0".to_owned()),
        traits: Vec::new(),
        configs: Vec::new(),
    }
}

pub(crate) fn result_type(ok: TypeIr) -> TypeIr {
    TypeIr::generic(
        "Future",
        vec![TypeIr::generic(
            "Result",
            vec![ok, TypeIr::named("SqlxError")],
        )],
    )
}
