#![allow(dead_code)]

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ClassKindIr, LibraryIr, SpanIr, SymbolId};
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn sample_library() -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 80),
            fields: Vec::new(),
            constructors: Vec::new(),
            traits: Vec::new(),
            serde: None,
        }],
        enums: Vec::new(),
    }
}

pub(crate) struct FakePlugin {
    pub(crate) name: &'static str,
    pub(crate) traits: Vec<SymbolId>,
    pub(crate) configs: Vec<SymbolId>,
    pub(crate) requested: Vec<&'static str>,
}

impl DustPlugin for FakePlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn claimed_traits(&self) -> Vec<SymbolId> {
        self.traits.clone()
    }

    fn claimed_configs(&self) -> Vec<SymbolId> {
        self.configs.clone()
    }

    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        self.requested
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        PluginContribution::default()
    }
}
