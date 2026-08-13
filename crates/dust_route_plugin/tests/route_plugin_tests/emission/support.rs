use std::{fs, path::PathBuf};

use dust_ir::DartFileIr;
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};
use dust_route_plugin::register_plugin;

pub(crate) fn generate_route_output(library: &DartFileIr) -> PluginContribution {
    register_plugin()
        .generate(
            library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution")
}

pub(crate) fn generate_route_output_with_plan(
    library: &DartFileIr,
    plan: &SymbolPlan,
) -> PluginContribution {
    register_plugin()
        .generate(
            library,
            &dust_plugin_api::PluginContext { symbol_plan: plan },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution")
}

pub(crate) fn assert_route_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    if std::env::var_os("DUST_UPDATE_ROUTE_SNAPSHOTS").is_some() {
        fs::write(&path, actual).unwrap();
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing route snapshot `{}`: {error}", path.display()));
    assert_eq!(actual, expected, "route snapshot `{name}` changed");
}

pub(crate) fn route_outputs_snapshot(contribution: &PluginContribution) -> String {
    assert!(contribution.primary_source.is_none());
    assert!(contribution.suppress_primary_output);
    let mut outputs = contribution
        .auxiliary_outputs
        .iter()
        .map(|output| {
            let path = output.output_path.to_string_lossy();
            format!("// file: {path}\n{}", output.source)
        })
        .collect::<Vec<_>>();
    outputs.sort();
    outputs.join("\n")
}

pub(crate) fn diagnostic_messages(diagnostics: &[dust_diagnostics::Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/route_plugin_tests/snapshots")
        .join(name)
}
