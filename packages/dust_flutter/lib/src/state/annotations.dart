// coverage:ignore-file

/// Selects the generated ViewModel base shape.
enum ViewModelMode {
  /// Synchronous state managed directly by the ViewModel.
  sync,

  /// Async loaded data wrapped in generated async lifecycle state.
  async,
}

/// Marks a class as a Dust view model generation target.
///
/// Example:
/// ```dart
/// @ViewModel(state: TaskBoardState, args: TaskBoardArgs)
/// final class TaskBoardViewModel extends $TaskBoardViewModel {
///   TaskBoardViewModel(super.args);
/// }
///
/// @ViewModel(
///   state: ShellTab,
///   args: ShellViewModelArgs,
///   initial: ShellTab.dashboard,
/// )
/// final class ShellViewModel extends $ShellViewModel {
///   ShellViewModel(super.args);
/// }
/// ```
class ViewModel {
  /// Creates metadata consumed by the Dust state generator.
  const ViewModel({
    required this.state,
    this.args,
    this.initial,
    this.mode = ViewModelMode.sync,
  });

  /// Immutable state type managed by the generated view model base.
  final Type state;

  /// Optional dependency bundle type. It must extend `ViewModelArgs`.
  final Type? args;

  /// Optional initial state expression for enums, imported states, or states
  /// without a default `const State()` constructor.
  final Object? initial;

  /// Generated ViewModel mode.
  final ViewModelMode mode;
}
