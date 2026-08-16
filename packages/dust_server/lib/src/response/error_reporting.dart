import 'dart:async';

/// Where errors that escape a handler are reported.
///
/// `Router(onError:)` installs a reporter for the requests that router serves,
/// scoped to a zone rather than to the process. Two routers in one isolate,
/// which is normal in tests and in anything serving several ports, keep their
/// own sinks.
///
/// ```dart
/// ServerErrors.reporter = (error, stack) => log.severe('handler failed', error);
/// ```
abstract final class ServerErrors {
  static const _zoneKey = #dust_server.errorReporter;

  static void Function(Object error, StackTrace stack)? _fallback;

  /// The reporter for the current zone, or the process-wide one.
  ///
  /// Reading prefers whatever [runWith] installed, so a router's own sink wins
  /// over anything set globally.
  static void Function(Object error, StackTrace stack)? get reporter {
    final scoped = Zone.current[_zoneKey];
    if (scoped is void Function(Object, StackTrace)) return scoped;
    return _fallback;
  }

  /// Sets the reporter used wherever no router installed one.
  static set reporter(void Function(Object error, StackTrace stack)? sink) {
    _fallback = sink;
  }

  /// Runs [body] with [sink] as the reporter for everything it awaits.
  static R runWith<R>(
    void Function(Object error, StackTrace stack) sink,
    R Function() body,
  ) {
    return runZoned(body, zoneValues: {_zoneKey: sink});
  }

  /// Reports an error that escaped a handler.
  static void report(Object error, StackTrace stack) {
    reporter?.call(error, stack);
  }
}
