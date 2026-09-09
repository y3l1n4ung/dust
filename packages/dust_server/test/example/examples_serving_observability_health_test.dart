import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import '../../example/background_tasks.dart' as background_tasks;
import '../../example/health_checks.dart' as health_checks;
import '../../example/print_request_response.dart' as print_both;
import 'serve.dart';

/// Running the server: isolates, shutdown, TLS and observability.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Metrics, sessions, health checks and request logging.
/// Health checks, request logging and background tasks.
void main() {
  group('print_request_response', () {
    test('the handler still receives the body the layer read', () async {
      // A body reads once. Without handing a fresh one down, the handler finds
      // an empty body and answers a puzzling 400.
      final app = await example(print_both.buildApp(log: (_) {}));

      final response = await app.post('/notes', const {'title': 'buy milk'});

      expect(response.statusCode, 201);
      expect(app.object(response), {'id': 1, 'title': 'buy milk'});
    });

    test('it logs the request and the response', () async {
      final lines = <String>[];
      final app = await example(print_both.buildApp(log: lines.add));

      await app.post('/notes', const {'title': 'buy milk'});

      expect(lines.first, startsWith('--> POST /notes'));
      expect(lines, contains('--> {"title":"buy milk"}'));
      expect(lines.last, startsWith('<-- 201'));
    });

    test('credential headers are redacted', () async {
      final lines = <String>[];
      final app = await example(print_both.buildApp(log: lines.add));

      await app.post(
        '/notes',
        const {'title': 'x'},
        headers: {'authorization': 'Bearer secret-token'},
      );

      expect(lines.first, contains('<redacted>'));
      expect(lines.join(), isNot(contains('secret-token')));
    });
  });

  group('health_checks', () {
    test('liveness answers without touching a dependency', () async {
      // When the database blips, liveness must not fail on every instance at
      // once and get the whole fleet restarted.
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': false},
      );
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/live')).statusCode, 200);
    });

    test('readiness fails when a dependency is down', () async {
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': false},
      );
      final app = await example(health_checks.buildApp(checks));

      final response = await app.get('/health/ready');

      expect(response.statusCode, 503);
      expect(app.object(response)['error'], 'a dependency is down');
    });

    test('readiness reports up or down and nothing else', () async {
      // No versions, no connection strings, no error text: the endpoint is
      // unauthenticated.
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': true, 'cache': true},
      );
      final app = await example(health_checks.buildApp(checks));

      expect(app.object(await app.get('/health/ready'))['dependencies'], {
        'database': true,
        'cache': true,
      });
    });

    test('draining fails readiness while liveness still passes', () async {
      final checks = health_checks.HealthChecks();
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/ready')).statusCode, 200);
      checks.markDraining();

      expect((await app.get('/health/ready')).statusCode, 503);
      // Still alive, so nothing kills it mid-drain.
      expect((await app.get('/health/live')).statusCode, 200);
    });

    test('startup fails until warm-up finishes', () async {
      final checks = health_checks.HealthChecks();
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/startup')).statusCode, 503);
      checks.markStarted();
      expect((await app.get('/health/startup')).statusCode, 200);
    });
  });

  group('background_tasks', () {
    test('the response does not wait for the task', () async {
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));

      final response =
          await app.post('/orders', const {'email': 'ada@example.com'});

      expect(response.statusCode, 201);
      expect(app.object(response), {'placed': true, 'receiptQueued': true});
      // Not sent yet: the handler returned before the work finished.
      expect(app.object(await app.get('/receipts'))['sent'], isEmpty);
    });

    test('the task finishes afterwards', () async {
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));

      await app.post('/orders', const {'email': 'ada@example.com'});
      await tasks.settled(const Duration(seconds: 2));

      expect(
          app.object(await app.get('/receipts'))['sent'], ['ada@example.com']);
    });

    test('a draining registry refuses, and the handler is told', () async {
      // The order was placed and the receipt will not be sent. Knowing that is
      // the difference between an outbox row and a silent loss.
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));
      await tasks.close(within: const Duration(milliseconds: 10));

      final response =
          await app.post('/orders', const {'email': 'ada@example.com'});

      expect(app.object(response)['receiptQueued'], isFalse);
      expect(app.object(await app.get('/receipts'))['sent'], isEmpty);
    });
  });
}
