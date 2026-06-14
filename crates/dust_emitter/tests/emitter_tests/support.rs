use dust_diagnostics::Diagnostic;
use dust_emitter::emit_library;
use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{
    DustPlugin, GENERATED_HEADER, PluginContribution, PluginRegistry, SymbolPlan,
};
use dust_text::{FileId, TextRange};

pub(crate) fn generated_output(body: &str) -> String {
    format!("{GENERATED_HEADER}\n{body}")
}

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
        configs: Vec::new(),
    }
}

pub(crate) fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        kind,
        has_default: false,
        default_value_source: None,
    }
}

pub(crate) fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(30, 40),
        params,
    }
}

pub(crate) fn class(name: &str, fields: Vec<FieldIr>, constructors: Vec<ConstructorIr>) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
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
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/user.dart".to_owned(),
        output_path,
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 120),
        classes: vec![class(
            "User",
            Vec::new(),
            vec![constructor(None, Vec::new())],
        )],
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
