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

/// Token used by view models to ignore stale async work.
@immutable
final class ViewModelActionToken {
  const ViewModelActionToken._(this.key, this.version);

  /// Logical action key, usually a private string or symbol.
  final Object key;

  final int version;
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
  final Map<Object, int> _actionVersions = <Object, int>{};
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

  /// Starts or supersedes an async action identified by [key].
  ///
  /// Store the returned token before awaiting. After the await, call
  /// [isCurrentAction] before emitting state. This prevents older async work
  /// from overwriting newer state.
  @protected
  ViewModelActionToken beginAction(Object key) {
    final version = (_actionVersions[key] ?? 0) + 1;
    _actionVersions[key] = version;
    return ViewModelActionToken._(key, version);
  }

  /// Returns whether [token] still belongs to the latest action for its key.
  @protected
  bool isCurrentAction(ViewModelActionToken token) {
    return !_isDisposed && _actionVersions[token.key] == token.version;
  }

  /// Invalidates any pending action for [key].
  @protected
  void cancelAction(Object key) {
    _actionVersions[key] = (_actionVersions[key] ?? 0) + 1;
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
    this.debugName,
    required this.args,
    required this.create,
    required this.builder,
  }) : value = null;

  /// Creates a provider for an externally owned view model.
  const ViewModelOwner.value({
    super.key,
    this.debugName,
    required TViewModel this.value,
    required this.builder,
  }) : args = null,
       create = null;

  /// Args factory for the owned constructor.
  final ViewModelArgsFactory<TArgs>? args;

  /// Human-readable scope name used in dependency-injection errors.
  final String? debugName;

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
    late final TViewModel created;
    try {
      created = create(context, argsFactory(context));
    } catch (error, stackTrace) {
      final ownerName = widget.debugName ?? 'ViewModelOwner<$TViewModel>';
      Error.throwWithStackTrace(
        StateError(
          '$ownerName failed to create its view model. Check the generated '
          'scope args/create dependency injection. Original error: $error',
        ),
        stackTrace,
      );
    }
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
