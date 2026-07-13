//! Integration tests for resolver-normalized view model configuration.

use dust_ir::{NormalizedConfigIr, StateModeIr};
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, resolve_library};
use dust_text::{FileId, SourceText};

#[test]
fn normalizes_view_model_from_parser_owned_values() {
    let source = SourceText::new(
        FileId::new(7),
        r#"
part 'task_board.g.dart';

@ViewModel(
  state: TaskBoardState,
  args: TaskBoardArgs,
  initial: const TaskBoardState.empty(),
  mode: ViewModelMode.async,
)
class TaskBoardViewModel extends $TaskBoardViewModel {}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("ViewModel", "dust_flutter::ViewModel");

    let resolved = resolve_library(
        FileId::new(7),
        "lib/task_board.dart",
        "lib/task_board.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let Some(NormalizedConfigIr::State(state)) =
        resolved.library.classes[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized ViewModel configuration");
    };
    assert_eq!(state.state_type, "TaskBoardState");
    assert_eq!(state.args_type.as_deref(), Some("TaskBoardArgs"));
    assert_eq!(
        state.initial_source.as_deref(),
        Some("const TaskBoardState.empty()")
    );
    assert_eq!(state.mode_source.as_deref(), Some("ViewModelMode.async"));
    assert_eq!(state.mode, StateModeIr::Async);
}

#[test]
fn leaves_view_model_untyped_when_state_is_missing() {
    let source = SourceText::new(
        FileId::new(8),
        "part 'task_board.g.dart'; @ViewModel(args: TaskBoardArgs) class TaskBoardViewModel extends $TaskBoardViewModel {}",
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("ViewModel", "dust_flutter::ViewModel");

    let resolved = resolve_library(
        FileId::new(8),
        "lib/task_board.dart",
        "lib/task_board.g.dart",
        &parsed.library,
        &catalog,
    );
    assert!(resolved.library.classes[0].configs[0].normalized.is_none());
}
