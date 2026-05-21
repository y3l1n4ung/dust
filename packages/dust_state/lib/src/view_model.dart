import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// Shared dependency bundle base for generated view models.
base class ViewModelArgs {
  /// Creates base args with optional instrumentation hooks.
  const ViewModelArgs({this.observer});

  /// Observer used for debugging, analytics, and tests.
  final StateObserver? observer;
}

/// Receives state transitions and one-shot effects from view models.
abstract interface class StateObserver {
  /// Called after [viewModel] changes from [previous] to [next].
  void onChanged(Object viewModel, Object? previous, Object? next);

  /// Called when [viewModel] emits a one-shot [effect].
  void onEffect(Object viewModel, Object effect);
}

/// Debug observer that logs state transitions and effects.
final class LoggingStateObserver implements StateObserver {
  /// Creates a logging observer.
  const LoggingStateObserver();

  @override
  void onChanged(Object viewModel, Object? previous, Object? next) {
    if (!kDebugMode) return;
    debugPrint('STATE CHANGE: ${viewModel.runtimeType}');
    debugPrint('FROM: $previous');
    debugPrint('TO:   $next');
  }

  @override
  void onEffect(Object viewModel, Object effect) {
    if (!kDebugMode) return;
    debugPrint('EFFECT: ${viewModel.runtimeType} -> $effect');
  }
}

/// No-op observer for tests and production apps that do not want logging.
final class SilentStateObserver implements StateObserver {
  /// Creates a no-op observer.
  const SilentStateObserver();

  @override
  void onChanged(Object viewModel, Object? previous, Object? next) {}

  @override
  void onEffect(Object viewModel, Object effect) {}
}

/// One-shot event emitted by a [ViewModelBase].
final class StateEffect {
  /// Creates an effect payload.
  const StateEffect(this.value);

  /// User-defined effect value.
  final Object value;
}

/// Base class used by generated Dust view model bases.
abstract class ViewModelBase<TState, TArgs extends ViewModelArgs>
    extends ValueNotifier<TState> {
  /// Creates a view model with typed [args] and [initialState].
  ViewModelBase(this.args, {required TState initialState})
    : super(initialState);

  /// Typed dependencies for this view model.
  final TArgs args;

  final StreamController<Object> _effects =
      StreamController<Object>.broadcast();
  Future<void>? _initFuture;
  bool _didInit = false;
  bool _isDisposed = false;

  /// Current state. Prefer this over [value] in app code.
  TState get state => value;

  @override
  set value(TState next) {
    if (_isDisposed || super.value == next) return;
    final previous = super.value;
    super.value = next;
    observer?.onChanged(this, previous, next);
  }

  /// Observer inherited from [args].
  StateObserver? get observer => args.observer;

  /// Broadcast stream of one-shot effects.
  Stream<Object> get effects => _effects.stream;

  /// Emits [next] if it differs from the current state.
  @protected
  void emit(TState next) {
    value = next;
  }

  /// Emits a one-shot [effect] without mutating state.
  @protected
  void emitEffect(Object effect) {
    if (_isDisposed) return;
    observer?.onEffect(this, effect);
    _effects.add(effect);
  }

  /// Override for one-time initialization.
  @protected
  FutureOr<void> onInit() {}

  /// Runs [onInit] at most once, even under concurrent scope rebuilds.
  Future<void> init() {
    if (_didInit) return Future<void>.value();
    return _initFuture ??= _runInit();
  }

  Future<void> _runInit() async {
    try {
      await onInit();
      _didInit = true;
    } finally {
      _initFuture = null;
    }
  }

  @override
  void dispose() {
    _isDisposed = true;
    _effects.close();
    super.dispose();
  }
}

/// Creates args for a generated view model scope.
typedef ViewModelArgsFactory<TArgs extends ViewModelArgs> =
    TArgs Function(BuildContext context);

/// Creates a view model for a generated scope.
typedef ViewModelFactory<
  TViewModel extends ViewModelBase<dynamic, dynamic>,
  TArgs extends ViewModelArgs
> = TViewModel Function(BuildContext context, TArgs args);

/// Generic owner used by generated scopes.
///
/// Generated code should wrap this with typed APIs instead of exposing it
/// directly to app code.
class ViewModelOwner<
  TViewModel extends ViewModelBase<dynamic, dynamic>,
  TArgs extends ViewModelArgs
>
    extends StatefulWidget {
  /// Creates an owner that constructs and disposes the view model.
  const ViewModelOwner({
    super.key,
    required this.args,
    required this.create,
    required this.builder,
  }) : value = null;

  /// Creates a provider for an externally owned view model.
  const ViewModelOwner.value({
    super.key,
    required TViewModel this.value,
    required this.builder,
  }) : args = null,
       create = null;

  /// Args factory for the owned constructor.
  final ViewModelArgsFactory<TArgs>? args;

  /// View model factory for the owned constructor.
  final ViewModelFactory<TViewModel, TArgs>? create;

  /// External view model for `.value` usage.
  final TViewModel? value;

  /// Builds the subtree with a ready view model.
  final Widget Function(BuildContext context, TViewModel viewModel) builder;

  @override
  State<ViewModelOwner<TViewModel, TArgs>> createState() =>
      _ViewModelOwnerState<TViewModel, TArgs>();
}

class _ViewModelOwnerState<
  TViewModel extends ViewModelBase<dynamic, dynamic>,
  TArgs extends ViewModelArgs
>
    extends State<ViewModelOwner<TViewModel, TArgs>> {
  TViewModel? _owned;

  TViewModel get _viewModel {
    final value = widget.value;
    if (value != null) return value;
    final owned = _owned;
    if (owned == null) {
      throw StateError('ViewModelOwner was read before initialization.');
    }
    return owned;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (widget.value != null || _owned != null) return;
    final argsFactory = widget.args;
    final create = widget.create;
    if (argsFactory == null || create == null) {
      throw StateError('Owned ViewModelOwner requires args and create.');
    }
    final created = create(context, argsFactory(context));
    _owned = created;
    scheduleMicrotask(() {
      if (mounted && identical(_owned, created)) {
        created.init();
      }
    });
  }

  @override
  void dispose() {
    _owned?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, _viewModel);
}
