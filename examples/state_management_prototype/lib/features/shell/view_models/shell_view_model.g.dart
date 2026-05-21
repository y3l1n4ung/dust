part of 'shell_view_model.dart';

abstract class $ShellViewModel
    extends ViewModelBase<ShellTab, ShellViewModelArgs> {
  $ShellViewModel(super.args) : super(initialState: ShellTab.dashboard);
}

class _$ShellViewModelProxy {
  _$ShellViewModelProxy(this._context, this._vm);

  final BuildContext _context;
  final ShellViewModel _vm;

  ShellTab get value {
    ShellViewModelScope.of(_context);
    return _vm.value;
  }

  void selectTab(ShellTab tab) => _vm.selectTab(tab);
}

class ShellViewModelScope extends StatefulWidget {
  const ShellViewModelScope({
    super.key,
    required this.args,
    required this.create,
    required this.child,
  }) : value = null;

  const ShellViewModelScope.value({
    super.key,
    required ShellViewModel this.value,
    required this.child,
  }) : args = null,
       create = null;

  final ShellViewModelArgs Function(BuildContext context)? args;
  final ShellViewModel Function(BuildContext context, ShellViewModelArgs args)?
  create;
  final ShellViewModel? value;
  final Widget child;

  static ShellViewModel read(BuildContext context) {
    final scope =
        context
                .getElementForInheritedWidgetOfExactType<
                  _ShellViewModelInherited
                >()
                ?.widget
            as _ShellViewModelInherited?;
    if (scope == null) {
      throw StateError('No ShellViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  static ShellViewModel of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<_ShellViewModelInherited>();
    if (scope == null) {
      throw StateError('No ShellViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  @override
  State<ShellViewModelScope> createState() => _ShellViewModelScopeState();
}

class _ShellViewModelScopeState extends State<ShellViewModelScope> {
  @override
  Widget build(BuildContext context) {
    final external = widget.value;
    final owner = external == null
        ? ViewModelOwner<ShellViewModel, ShellViewModelArgs>(
            args: widget.args!,
            create: widget.create!,
            builder: _buildInherited,
          )
        : ViewModelOwner<ShellViewModel, ShellViewModelArgs>.value(
            value: external,
            builder: _buildInherited,
          );
    return owner;
  }

  Widget _buildInherited(BuildContext context, ShellViewModel viewModel) {
    return ListenableBuilder(
      listenable: viewModel,
      builder: (context, child) =>
          _ShellViewModelInherited(viewModel: viewModel, child: child!),
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

class ShellViewModelListener extends StatefulWidget {
  const ShellViewModelListener({
    super.key,
    required this.listener,
    required this.child,
  });

  final void Function(BuildContext context, Object effect) listener;
  final Widget child;

  @override
  State<ShellViewModelListener> createState() => _ShellViewModelListenerState();
}

class _ShellViewModelListenerState extends State<ShellViewModelListener> {
  StreamSubscription<Object>? _sub;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sub?.cancel();
    _sub = ShellViewModelScope.read(context).effects.listen((effect) {
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

extension ShellViewModelBuildContext on BuildContext {
  _$ShellViewModelProxy watchShellViewModel() =>
      _$ShellViewModelProxy(this, ShellViewModelScope.read(this));
  ShellViewModel readShellViewModel() => ShellViewModelScope.read(this);
}
