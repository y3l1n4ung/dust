//! Micro-benchmark for generated state plugin emission throughput.

use std::{sync::Arc, time::Instant};

use dust_ir::{ClassIr, ClassKindIr, ConfigApplicationIr, LibraryIr, SpanIr, SymbolId};
use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;
use dust_text::{FileId, TextRange};

/// Builds a placeholder source span for synthetic benchmark IR nodes.
fn span() -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
}

/// Builds a synthetic view model class configured for state emission.
fn view_model(name: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some(format!("${name}")),
        span: span(),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![ConfigApplicationIr::new(
            SymbolId::new("dust_flutter::ViewModel"),
            Some("(state: BenchState, args: ViewModelArgs)".to_owned()),
            span(),
        )],
        serde: None,
    }
}

fn main() {
    const VIEW_MODELS: usize = 250;
    const FIELDS: usize = 40;
    const ITERS: usize = 50;

    let fields = (0..FIELDS)
        .map(|index| format!(r#"{{"name":"field{index}","type_source":"int"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        format!(r#"{{"class_name":"BenchState","fields":[{fields}]}}"#),
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(builder.build()));
    let library = LibraryIr {
        package_root: ".".to_owned(),
        package_name: "bench".to_owned(),
        source_path: "lib/bench.dart".to_owned(),
        output_path: "lib/bench.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(),
        classes: (0..VIEW_MODELS)
            .map(|index| view_model(&format!("Bench{index}ViewModel")))
            .collect(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums: Vec::new(),
        query_calls: Vec::new(),
    };
    let plugin = register_plugin();
    let started = Instant::now();
    let mut bytes = 0usize;
    for _ in 0..ITERS {
        let contribution = plugin.emit(&library, &plan);
        bytes += contribution
            .support_types
            .iter()
            .map(String::len)
            .sum::<usize>();
    }
    let elapsed = started.elapsed();
    println!(
        "state_emit view_models={VIEW_MODELS} fields={FIELDS} iterations={ITERS} bytes={bytes} elapsed_ms={}",
        elapsed.as_millis()
    );
}
