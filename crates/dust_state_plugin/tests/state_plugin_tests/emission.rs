use std::{
    fs,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use super::support::{args_class, library_with_classes, state_class, view_model_class};

#[test]
fn emits_generated_base_with_args_getters() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    assert_eq!(contribution.support_types.len(), 1);
    let source = &contribution.support_types[0];
    assert!(source.contains("abstract class $TaskBoardViewModel"));
    assert!(source.contains("extends ViewModelBase<TaskBoardState, TaskBoardArgs>"));
    assert!(source.contains(
        "$TaskBoardViewModel(super.args) : super(initialState: const TaskBoardState());"
    ));
    assert!(source.contains("PrototypeRepository get repository => args.repository;"));
    assert!(source.contains("class TaskBoardViewModelScope extends StatefulWidget"));
    assert!(source.contains("debugName: 'TaskBoardViewModelScope'"));
    assert!(source.contains("debugName: 'TaskBoardViewModelScope.value'"));
    assert!(source.contains("class TaskBoardViewModelListener extends StatefulWidget"));
    assert_eq!(
        extract_extension(source, "extension TaskBoardViewModelBuildContext"),
        r#"extension TaskBoardViewModelBuildContext on BuildContext {
  TaskBoardViewModel get taskBoardViewModel => TaskBoardViewModelScope.of(this);

  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this, TaskBoardViewModelScope.read(this));
  }

  TaskBoardViewModel readTaskBoardViewModel() => TaskBoardViewModelScope.read(this);
}"#
    );
}

#[test]
fn emits_explicit_initial_expression() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            args_class(),
            view_model_class(
                "ShellViewModel",
                "(state: ShellTab, args: TaskBoardArgs, initial: ShellTab.dashboard)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    let source = &contribution.support_types[0];
    assert!(
        source.contains("$ShellViewModel(super.args) : super(initialState: ShellTab.dashboard);")
    );
}

#[test]
fn emits_state_fields_from_workspace_analysis() {
    let plugin = register_plugin();
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"TaskBoardState","fields":[{"name":"count","type_source":"int"},{"name":"message","type_source":"String?"}]}"#,
    );
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"TaskBoardArgs","fields":[{"name":"repository","type_source":"PrototypeRepository"},{"name":"observer","type_source":"StateObserver?"}]}"#,
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(builder.build()));

    let contribution = plugin.emit(
        &library_with_classes(vec![
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &plan,
    );

    let source = &contribution.support_types[0];
    assert!(source.contains("enum _TaskBoardViewModelAspect { count, message }"));
    assert!(source.contains("int get count => state.count;"));
    assert!(source.contains("String? get message => state.message;"));
    assert!(source.contains("PrototypeRepository get repository => args.repository;"));
    assert!(!source.contains("StateObserver? get observer"));
    assert_eq!(
        extract_class(source, "class _$TaskBoardViewModelProxy"),
        r#"class _$TaskBoardViewModelProxy {
  _$TaskBoardViewModelProxy(this._context, this._vm);

  final BuildContext _context;
  final TaskBoardViewModel _vm;

  TaskBoardState get value {
    TaskBoardViewModelScope.of(_context);
    return _vm.value;
  }

  int get count {
    TaskBoardViewModelScope.of(_context, aspect: _TaskBoardViewModelAspect.count);
    return _vm.state.count;
  }

  String? get message {
    TaskBoardViewModelScope.of(_context, aspect: _TaskBoardViewModelAspect.message);
    return _vm.state.message;
  }
}"#
    );
    assert_eq!(
        extract_class(source, "class _TaskBoardViewModelInherited"),
        r#"class _TaskBoardViewModelInherited extends InheritedModel<Object> {
  const _TaskBoardViewModelInherited({required this.viewModel, required this.state, required super.child});

  final TaskBoardViewModel viewModel;
  final TaskBoardState state;

  @override
  bool updateShouldNotify(_TaskBoardViewModelInherited oldWidget) => state != oldWidget.state;

  @override
  bool updateShouldNotifyDependent(_TaskBoardViewModelInherited oldWidget, Set<Object> dependencies) {
    for (final aspect in dependencies) {
      switch (aspect) {
        case _TaskBoardViewModelAspect.count:
          if (state.count != oldWidget.state.count) {
            return true;
          }
          break;
        case _TaskBoardViewModelAspect.message:
          if (state.message != oldWidget.state.message) {
            return true;
          }
          break;
        default:
          break;
      }
    }
    return false;
  }
}"#
    );
}

#[test]
fn emits_state_fields_from_imported_unannotated_state_file() {
    let root = temp_root("imported_state_fields");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/products_state.dart"),
        "enum ProductsStatus { initial, success }\n\
         class ProductsState {\n\
           final List<Product> products;\n\
           final ProductsStatus status;\n\
           final String? errorMessage;\n\
           const ProductsState({this.products = const [], required this.status, this.errorMessage});\n\
         }\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let mut library = library_with_classes(vec![
        args_class(),
        view_model_class(
            "ProductsViewModel",
            "(state: ProductsState, args: TaskBoardArgs)",
        ),
    ]);
    library.package_root = root.display().to_string();
    library.source_path = "lib/view_models/products_view_model.dart".to_owned();
    library
        .imports
        .push("../models/products_state.dart".to_owned());

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let source = &contribution.support_types[0];

    assert!(source.contains("enum _ProductsViewModelAspect { products, status, errorMessage }"));
    assert!(source.contains("List<Object?> get products"));
    assert!(source.contains("ProductsStatus get status"));
    assert!(source.contains("String? get errorMessage"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn emits_value_only_proxy_for_fieldless_state() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    let source = &contribution.support_types[0];
    assert!(!source.contains("enum _TaskBoardViewModelAspect"));
    assert!(source.contains("TaskBoardState get value"));
    assert_eq!(
        extract_extension(source, "extension TaskBoardViewModelBuildContext"),
        r#"extension TaskBoardViewModelBuildContext on BuildContext {
  TaskBoardViewModel get taskBoardViewModel => TaskBoardViewModelScope.of(this);

  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this, TaskBoardViewModelScope.read(this));
  }

  TaskBoardViewModel readTaskBoardViewModel() => TaskBoardViewModelScope.read(this);
}"#
    );
}

#[test]
fn emits_single_output_per_annotated_view_model() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
            view_model_class(
                "SecondaryViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    assert_eq!(contribution.support_types.len(), 2);
    assert!(contribution.support_types[0].contains("$TaskBoardViewModel"));
    assert!(contribution.support_types[1].contains("$SecondaryViewModel"));
}

fn extract_extension<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    &source[start..]
}

fn extract_class<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    let mut depth = 0_i32;
    let mut saw_body = false;
    for (offset, ch) in source[start..].char_indices() {
        match ch {
            '{' => {
                depth += 1;
                saw_body = true;
            }
            '}' if saw_body => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..start + offset + ch.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("class body did not close: {marker}");
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dust_state_plugin_{name}_{stamp}"))
}
