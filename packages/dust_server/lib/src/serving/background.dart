import 'dart:async';

import 'package:meta/meta.dart';

import '../extraction/state.dart';
import '../response/error_reporting.dart';
import '../router/router_base.dart';
import '../tracing/tracing_layer.dart';

/// Work that outlives the response that started it.
///
/// Sending a receipt, warming a cache, writing an audit row: things a caller
/// should not wait for, and that still have to finish.
///
/// The reason this exists rather than a bare `unawaited(...)`: shutdown counts
/// **requests**, so work spawned outside one is invisible to it. On every
/// deploy that work is killed mid-flight, and nothing logs it — a customer gets
/// a 201 and never gets their email, with no trace of why. A task registered
/// here is drained with everything else.
///
/// ```dart
/// final tasks = BackgroundTasks();
/// final server = await serveRouter(app, address, 8080, background: tasks);
///
/// Future<Order> checkout(Request request) async {
///   final order = await shop.place(basket);
///   final tasks = await request.state<BackgroundTasks>();
///
///   tasks.run('receipt', () => mail.sendReceipt(order));
///   return order;
/// }
/// ```
///
/// This is in-process and unpersisted. A task lost to a crash is gone, and it
/// does not retry. That is the right shape for work that is *nice to finish* and
/// the wrong one for work that must happen — an outbox table or a real queue is
/// the answer there, and this is not a substitute for one.
final class BackgroundTasks {
  /// Creates an empty registry.
  ///
  /// [onError] is told when a task throws. It defaults to the router's error
  /// reporter, so a failing task surfaces the same way a failing handler does
  /// rather than vanishing.
  BackgroundTasks({void Function(Object error, StackTrace stack)? onError})
      : _onError = onError;

  final void Function(Object error, StackTrace stack)? _onError;
  final _running = <Future<void>>{};

  Completer<void>? _idle;
  bool _closed = false;

  /// How many tasks are running right now.
  int get pending => _running.length;

  /// Whether the registry has stopped accepting work.
  bool get isClosed => _closed;

  /// Runs [body] in the background, tracked so shutdown waits for it.
  ///
  /// Returns whether it was accepted. A registry that is draining refuses, so a
  /// task started during shutdown is not begun and then abandoned — better to
  /// know it never ran than to half-run it.
  ///
  /// [name] appears in the error report when [body] throws. A task that fails
  /// anonymously at three in the morning is one nobody can place.
  bool run(String name, Future<void> Function() body) {
    if (_closed) return false;

    late final Future<void> task;
    // Detached from the request's span. A zone value is inherited by whatever is
    // spawned inside it, and the request's span ends when the response goes out
    // — so a task that kept it would write attributes onto a finished, already
    // exported span. Work that wants a trace should start its own.
    task = CurrentSpan.runDetached(() => Future<void>.sync(body)).then(
      (_) {},
      onError: (Object error, StackTrace stack) {
        // Swallowed here rather than escaping into the zone, where an unhandled
        // asynchronous error takes the isolate down with it.
        final report = _onError ?? ServerErrors.report;
        report(_TaskFailed(name, error), stack);
      },
    ).whenComplete(() {
      _running.remove(task);
      if (_running.isEmpty) {
        _idle?.complete();
        _idle = null;
      }
    });

    _running.add(task);
    return true;
  }

  /// Stops accepting new tasks, then waits [within] for the running ones.
  ///
  /// Returns `true` when everything finished, and `false` when the budget passed
  /// with work still going — the same contract as `ServerHandle.close`, and the
  /// only signal that something was abandoned.
  Future<bool> close({Duration within = const Duration(seconds: 15)}) async {
    _closed = true;
    return settled(within);
  }

  /// Waits [within] for the running tasks without closing the registry.
  Future<bool> settled(Duration within) async {
    if (_running.isEmpty) return true;

    final idle = _idle ??= Completer<void>();
    try {
      await idle.future.timeout(within);
      return true;
    } on TimeoutException {
      return false;
    }
  }
}

/// Names the task that failed, so the report says which one.
final class _TaskFailed {
  const _TaskFailed(this.name, this.cause);

  final String name;
  final Object cause;

  @override
  String toString() => 'background task "$name" failed: $cause';
}

/// The [BackgroundTasks] attached to [router] with `withState`, when there is
/// one.
///
/// Lets `serveRouter` find a registry the caller did not hand it, which is the
/// only way a clustered server can drain one: each isolate builds its own inside
/// the factory, so nothing outside can pass it in.
@internal
BackgroundTasks? backgroundTasksIn(Router router) {
  final attached = router.internals.state[stateKeyFor<BackgroundTasks>()];
  return attached is BackgroundTasks ? attached : null;
}
