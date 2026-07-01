use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::support::{make_workspace, write_file};

#[test]
fn build_writes_state_output_for_view_model_library() {
    let workspace = make_workspace();
    write_state_workspace(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let output = workspace.path().join("lib/task_board_view_model.g.dart");
    let source = fs::read_to_string(&output).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(output.exists());
    assert!(source.contains("abstract class $TaskBoardViewModel"));
    assert!(source.contains("extends ViewModelBase<TaskBoardState, TaskBoardArgs>"));
    assert!(!source.contains("_TaskBoardViewModelAspect"));
    assert!(source.contains("TaskBoardState get value"));
    assert!(!source.contains("select<R>"));
    assert!(!source.contains("int get count"));
    assert!(!source.contains("String? get message"));
}

fn write_state_workspace(root: &std::path::Path) {
    write_file(
        &root.join("lib/task_board_view_model.dart"),
        "import 'package:dust_flutter/state.dart';\n\
         part 'task_board_view_model.g.dart';\n\
         final class PrototypeRepository { const PrototypeRepository(); }\n\
         final class TaskBoardState {\n\
           const TaskBoardState({this.count = 0, this.message});\n\
           final int count;\n\
           final String? message;\n\
         }\n\
         final class TaskBoardArgs extends ViewModelArgs {\n\
           const TaskBoardArgs({required this.repository, super.observer});\n\
           final PrototypeRepository repository;\n\
         }\n\
         @ViewModel(state: TaskBoardState, args: TaskBoardArgs)\n\
         final class TaskBoardViewModel extends $TaskBoardViewModel {\n\
           TaskBoardViewModel(super.args);\n\
         }\n",
    );
}
