/// Instrumentation a generated view model reports through, and the argument
/// bundle that carries it.
library;

import 'package:flutter/foundation.dart';

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

/// Deprecated compatibility wrapper for one-shot effects.
///
/// Pass the payload object directly to `emitEffect(...)` instead. Dust unwraps
/// [StateEffect] before delivery so older code keeps working.
@Deprecated(
  'Pass the effect object directly to emitEffect(...). '
  'StateEffect is unwrapped for compatibility and will be removed.',
)
final class StateEffect {
  /// Creates an effect payload.
  const StateEffect(this.value);

  /// User-defined effect value.
  final Object value;
}
