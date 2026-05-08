part of 'shell_view_model.dart';

class ShellViewModelScope extends StatefulWidget {
  const ShellViewModelScope({
    super.key,
    required this.create,
    required this.child,
  });

  final ShellViewModel Function(BuildContext context) create;
  final Widget child;

  static ShellViewModel of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<_ShellViewModelInherited>();
    if (scope == null) {
      throw StateError('No ShellViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  @override
  State<ShellViewModelScope> createState() => _ShellViewModelScopeState();
}

class _ShellViewModelScopeState extends State<ShellViewModelScope> {
  ShellViewModel? _viewModel;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _viewModel ??= widget.create(context);
  }

  @override
  void dispose() {
    _viewModel?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _ShellViewModelInherited(
      viewModel: _viewModel!,
      child: widget.child,
    );
  }
}

class _ShellViewModelInherited extends InheritedNotifier<ShellViewModel> {
  const _ShellViewModelInherited({
    required this.viewModel,
    required super.child,
  }) : super(notifier: viewModel);

  final ShellViewModel viewModel;
}

extension ShellViewModelBuildContext on BuildContext {
  ShellViewModel get shellViewModel => ShellViewModelScope.of(this);
}
