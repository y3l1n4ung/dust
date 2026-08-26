import 'dart:io';

import 'package:dust_server/server.dart';

/// Liveness and readiness, which are different questions.
///
/// Conflating them is the classic outage amplifier:
///
/// * **Liveness** — "is this process wedged?" A failure gets it **killed and
///   restarted**. It must not check the database: when the database blips,
///   every instance fails liveness at once and the orchestrator restarts the
///   entire fleet, turning a recoverable outage into a total one.
/// * **Readiness** — "should this instance receive traffic right now?" A failure
///   takes it **out of the load balancer** and nothing more. This is where a
///   dependency check belongs, and it is the one that should be slow to recover.
///
/// A third is worth having on a slow start: **startup**, so an instance that
/// takes a minute to warm caches is not killed for failing liveness while it
/// does.
///
/// > **Health endpoints are unauthenticated and therefore reconnaissance.** They
/// > name your dependencies, their versions, and when a deploy happened. Report
/// > `up`/`down` per dependency and nothing else — no connection strings, no
/// > versions, no error text — or put them on a port the internet cannot reach.
///
/// Run it with `dart run example/health_checks.dart`:
///
/// ```bash
/// curl -s  localhost:8080/health/live    # never touches a dependency
/// curl -s  localhost:8080/health/ready
/// curl -si localhost:8080/health/ready   # 503 once the database is marked down
/// curl -s  localhost:8080/health/startup
/// ```
Future<void> main() async {
  final checks = HealthChecks();
  final server = await serve(
    buildApp(checks),
    InternetAddress.anyIPv4,
    8080,
  );
  // Warm-up finished: the instance may now be sent traffic.
  checks.markStarted();
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  // Fail readiness *before* draining, so the load balancer stops sending work
  // while the server finishes what it already accepted.
  checks.markDraining();
  await server.close(drain: const Duration(seconds: 15));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(HealthChecks checks) {
  return Router()
    ..route('/health/live', get(live))
    ..route('/health/ready', get(ready))
    ..route('/health/startup', get(started))
    ..withState(checks);
}

/// `GET /health/live` — is the isolate still turning over.
///
/// Deliberately checks nothing else. Answering at all is the whole signal.
Map<String, Object?> live(Request request) => const {'status': 'live'};

/// `GET /health/ready` — should this instance be sent traffic.
Future<Result<Map<String, Object?>, Rejection>> ready(Request request) async {
  final checks = await request.state<HealthChecks>();
  final report = await checks.dependencies();

  if (checks.draining) {
    return const Err(Rejection.status(503, 'shutting down'));
  }
  if (report.values.any((up) => !up)) {
    // 503, so the load balancer removes this instance rather than the
    // orchestrator killing it.
    return Err(Rejection.status(503, 'a dependency is down'));
  }

  return Ok({'status': 'ready', 'dependencies': report});
}

/// `GET /health/startup` — has warm-up finished.
Future<Result<Map<String, Object?>, Rejection>> started(Request request) async {
  final checks = await request.state<HealthChecks>();

  return checks.started
      ? const Ok({'status': 'started'})
      : const Err(Rejection.status(503, 'still starting'));
}

/// What the three endpoints report on.
final class HealthChecks {
  /// Creates the checks, optionally with a [probe] for dependencies.
  HealthChecks({Future<Map<String, bool>> Function()? probe}) : _probe = probe;

  final Future<Map<String, bool>> Function()? _probe;

  /// Whether warm-up finished.
  bool started = false;

  /// Whether the instance is shutting down.
  bool draining = false;

  /// Marks warm-up complete.
  void markStarted() => started = true;

  /// Marks the instance as leaving the pool.
  void markDraining() => draining = true;

  /// Each dependency, up or down. No versions, no error text.
  Future<Map<String, bool>> dependencies() async =>
      await _probe?.call() ?? const {'database': true};
}
