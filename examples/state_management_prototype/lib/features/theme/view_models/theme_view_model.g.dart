part of 'theme_view_model.dart';

abstract class $ThemeViewModel
    extends ViewModelBase<ThemeMode, ThemeViewModelArgs> {
  $ThemeViewModel(super.args) : super(initialState: ThemeMode.dark);
}

class _$ThemeViewModelProxy {
  _$ThemeViewModelProxy(this._context, this._vm);

  final BuildContext _context;
  final ThemeViewModel _vm;

  ThemeMode get value {
    ThemeViewModelScope.of(_context);
    return _vm.value;
  }

  void toggle() => _vm.toggle();
}

class ThemeViewModelScope extends StatefulWidget {
  const ThemeViewModelScope({
    super.key,
    required this.args,
    required this.create,
    required this.child,
  }) : value = null;
  const ThemeViewModelScope.value({
    super.key,
    required ThemeViewModel this.value,
    required this.child,
  }) : args = null,
       create = null;

  final ThemeViewModelArgs Function(BuildContext context)? args;
  final ThemeViewModel Function(BuildContext context, ThemeViewModelArgs args)?
  create;
  final ThemeViewModel? value;
  final Widget child;

  static ThemeViewModel read(BuildContext context) {
    final scope =
        context
                .getElementForInheritedWidgetOfExactType<
                  _ThemeViewModelInherited
                >()
                ?.widget
            as _ThemeViewModelInherited?;
    if (scope == null)
      throw StateError('No ThemeViewModelScope found in context.');
    return scope.viewModel;
  }

  static ThemeViewModel of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<_ThemeViewModelInherited>();
    if (scope == null)
      throw StateError('No ThemeViewModelScope found in context.');
    return scope.viewModel;
  }

  @override
  State<ThemeViewModelScope> createState() => _ThemeViewModelScopeState();
}

class _ThemeViewModelScopeState extends State<ThemeViewModelScope> {
  @override
  Widget build(BuildContext context) {
    final external = widget.value;
    return external == null
        ? ViewModelOwner<ThemeViewModel, ThemeViewModelArgs>(
            args: widget.args!,
            create: widget.create!,
            builder: _buildInherited,
          )
        : ViewModelOwner<ThemeViewModel, ThemeViewModelArgs>.value(
            value: external,
            builder: _buildInherited,
          );
  }

  Widget _buildInherited(BuildContext context, ThemeViewModel viewModel) {
    return ListenableBuilder(
      listenable: viewModel,
      builder: (context, child) =>
          _ThemeViewModelInherited(viewModel: viewModel, child: child!),
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

class ThemeViewModelListener extends StatefulWidget {
  const ThemeViewModelListener({
    super.key,
    required this.listener,
    required this.child,
  });
  final void Function(BuildContext context, Object effect) listener;
  final Widget child;
  @override
  State<ThemeViewModelListener> createState() => _ThemeViewModelListenerState();
}

class _ThemeViewModelListenerState extends State<ThemeViewModelListener> {
  StreamSubscription<Object>? _sub;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sub?.cancel();
    _sub = ThemeViewModelScope.read(context).effects.listen((effect) {
      if (mounted) widget.listener(context, effect);
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

extension ThemeViewModelBuildContext on BuildContext {
  _$ThemeViewModelProxy watchThemeViewModel() =>
      _$ThemeViewModelProxy(this, ThemeViewModelScope.read(this));
  ThemeViewModel readThemeViewModel() => ThemeViewModelScope.read(this);
}
