use dust_diagnostics::Diagnostic;
use dust_emitter::emit_library;
use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, PluginContribution, PluginRegistry, SymbolPlan};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
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

pub(crate) fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        kind,
        has_default: false,
    }
}

pub(crate) fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        span: span(30, 40),
        params,
    }
}

pub(crate) fn class(name: &str, fields: Vec<FieldIr>, constructors: Vec<ConstructorIr>) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: Vec::new(),
        serde: None,
    }
}

pub(crate) fn trait_app(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

pub(crate) fn sample_library(output_path: String) -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path,
        span: span(0, 120),
        classes: vec![class(
            "User",
            Vec::new(),
            vec![constructor(None, Vec::new())],
        )],
        enums: Vec::new(),
    }
}

pub(crate) struct FakePlugin {
    pub(crate) name: &'static str,
    pub(crate) requested: Vec<&'static str>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) contribution: PluginContribution,
}

impl DustPlugin for FakePlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        self.requested
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        self.contribution.clone()
    }
}

pub(crate) fn emit_with_registry(
    library: &LibraryIr,
    registry: &PluginRegistry,
    previous: Option<&str>,
) -> dust_emitter::EmitResult {
    emit_library(library, registry, previous)
}
