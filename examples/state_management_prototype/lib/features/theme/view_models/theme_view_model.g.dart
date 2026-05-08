part of 'theme_view_model.dart';

class ThemeViewModelScope extends StatefulWidget {
  const ThemeViewModelScope({
    super.key,
    required this.create,
    required this.child,
  });

  final ThemeViewModel Function(BuildContext context) create;
  final Widget child;

  static ThemeViewModel of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<_ThemeViewModelInherited>();
    if (scope == null) {
      throw StateError('No ThemeViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  @override
  State<ThemeViewModelScope> createState() => _ThemeViewModelScopeState();
}

class _ThemeViewModelScopeState extends State<ThemeViewModelScope> {
  ThemeViewModel? _viewModel;

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
    return _ThemeViewModelInherited(
      viewModel: _viewModel!,
      child: widget.child,
    );
  }
}

class _ThemeViewModelInherited extends InheritedNotifier<ThemeViewModel> {
  const _ThemeViewModelInherited({
    required this.viewModel,
    required super.child,
  }) : super(notifier: viewModel);

  final ThemeViewModel viewModel;
}

extension ThemeViewModelBuildContext on BuildContext {
  ThemeViewModel get themeViewModel => ThemeViewModelScope.of(this);
}
