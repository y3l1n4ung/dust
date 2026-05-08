part of 'session_view_model.dart';

class SessionViewModelScope extends StatefulWidget {
  const SessionViewModelScope({
    super.key,
    required this.create,
    required this.child,
  });

  final SessionViewModel Function(BuildContext context) create;
  final Widget child;

  static SessionViewModel of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<_SessionViewModelInherited>();
    if (scope == null) {
      throw StateError('No SessionViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  @override
  State<SessionViewModelScope> createState() => _SessionViewModelScopeState();
}

class _SessionViewModelScopeState extends State<SessionViewModelScope> {
  SessionViewModel? _viewModel;

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
    return _SessionViewModelInherited(
      viewModel: _viewModel!,
      child: widget.child,
    );
  }
}

class _SessionViewModelInherited extends InheritedNotifier<SessionViewModel> {
  const _SessionViewModelInherited({
    required this.viewModel,
    required super.child,
  }) : super(notifier: viewModel);

  final SessionViewModel viewModel;
}

extension SessionViewModelBuildContext on BuildContext {
  SessionViewModel get sessionViewModel => SessionViewModelScope.of(this);
}
