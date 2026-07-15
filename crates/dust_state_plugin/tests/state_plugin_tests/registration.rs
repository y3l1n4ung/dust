use dust_plugin_api::{DustPlugin, PluginContext, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use super::support::{args_class, library_with_classes, state_class, view_model_class};

#[test]
fn plugin_claims_view_model_config() {
    let plugin = register_plugin();

    assert_eq!(plugin.plugin_name(), "ViewModel");
    assert_eq!(plugin.claimed_configs(), vec!["dust_flutter::ViewModel"]);
    assert_eq!(plugin.supported_annotations(), vec!["ViewModel"]);
}

#[test]
fn collects_workspace_facts_from_canonical_ir() {
    let plugin = register_plugin();
    let library = library_with_classes(vec![
        state_class(),
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]);
    let mut analysis = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis_ir(&library, &mut analysis);
    let snapshot = analysis.snapshot();

    assert!(
        snapshot
            .string_set("dust_state.states.v1")
            .expect("state facts")
            .iter()
            .any(|value| value.contains(r#""class_name":"TaskBoardArgs""#))
    );
    assert_eq!(
        snapshot.string_set("dust_state.view_models.v1").unwrap(),
        &[r#"{"class_name":"TaskBoardViewModel","state_type":"TaskBoardState","args_type":"TaskBoardArgs","initial_source":null,"mode":"sync","generated_base_class":"$TaskBoardViewModel","import_uri":"lib/task_board_view_model.dart"}"#.to_owned()]
    );
}

#[test]
fn plugin_contract_returns_empty_output_for_unannotated_library() {
    let plugin = dust_state_plugin::StatePlugin::new();
    let library = library_with_classes(Vec::new());

    assert!(plugin.validate(&library).is_empty());
    assert_eq!(
        plugin
            .generate(
                &library,
                &dust_plugin_api::PluginContext {
                    symbol_plan: &dust_plugin_api::SymbolPlan::default()
                }
            )
            .into_iter()
            .next()
            .expect("plugin must generate one contribution"),
        dust_plugin_api::PluginContribution::default()
    );
}

#[test]
fn plugin_generate_returns_state_generated_unit() {
    let plugin = dust_state_plugin::StatePlugin::new();
    let library = library_with_classes(vec![
        state_class(),
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]);
    let plan = SymbolPlan::default();

    let units = plugin.generate(&library, &PluginContext { symbol_plan: &plan });

    assert_eq!(units.len(), 1);
    assert_eq!(
        units[0],
        plugin
            .generate(
                &library,
                &dust_plugin_api::PluginContext { symbol_plan: &plan }
            )
            .into_iter()
            .next()
            .expect("plugin must generate one contribution")
    );
    assert!(units[0].support_types[0].contains("abstract class $TaskBoardViewModel"));
}
