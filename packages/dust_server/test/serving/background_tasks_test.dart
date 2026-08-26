import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

/// Work that outlives the response, and the shutdown that has to wait for it.
///
/// The hole this closes: `close(drain:)` counted requests, so anything spawned
/// outside one was invisible. On every deploy that work was killed mid-flight
/// with nothing logged.

void main() {
  group('a registry', () {
    test('reports nothing pending when idle', () {
      expect(BackgroundTasks().pending, 0);
    });

    test('settles immediately when there is nothing to wait for', () async {
      expect(
        await BackgroundTasks().settled(const Duration(milliseconds: 10)),
        isTrue,
      );
    });

    test('counts what is running and forgets it when it finishes', () async {
      final tasks = BackgroundTasks();
      final gate = Completer<void>();

      expect(tasks.run('slow', () => gate.future), isTrue);
      expect(tasks.pending, 1);

      gate.complete();
      await tasks.settled(const Duration(seconds: 1));

      expect(tasks.pending, 0);
    });

    test('waits for several at once', () async {
      final tasks = BackgroundTasks();
      final gates = [Completer<void>(), Completer<void>(), Completer<void>()];
      for (final (index, gate) in gates.indexed) {
        tasks.run('task-$index', () => gate.future);
      }

      expect(tasks.pending, 3);
      for (final gate in gates) {
        gate.complete();
      }

      expect(await tasks.settled(const Duration(seconds: 1)), isTrue);
      expect(tasks.pending, 0);
    });

    test('reports false when the budget passes with work running', () async {
      // The only signal that something was abandoned.
      final tasks = BackgroundTasks();
      final gate = Completer<void>();
      tasks.run('never', () => gate.future);

      expect(
        await tasks.settled(const Duration(milliseconds: 50)),
        isFalse,
      );

      gate.complete();
    });
  });

  group('a task that throws', () {
    test('is reported rather than taking the isolate down', () async {
      final faults = <Object>[];
      final tasks = BackgroundTasks(onError: (error, _) => faults.add(error));

      tasks.run('receipt', () async => throw StateError('smtp refused'));
      await tasks.settled(const Duration(seconds: 1));

      expect(faults, hasLength(1));
      expect(tasks.pending, 0);
    });

    test('is named in the report, so it can be placed', () async {
      final faults = <Object>[];
      final tasks = BackgroundTasks(onError: (error, _) => faults.add(error));

      tasks.run('send-receipt', () async => throw StateError('smtp refused'));
      await tasks.settled(const Duration(seconds: 1));

      expect(faults.single.toString(), contains('send-receipt'));
      expect(faults.single.toString(), contains('smtp refused'));
    });

    test('throwing synchronously is caught too', () async {
      final faults = <Object>[];
      final tasks = BackgroundTasks(onError: (error, _) => faults.add(error));

      tasks.run('immediate', () => throw StateError('right away'));
      await tasks.settled(const Duration(seconds: 1));

      expect(faults, hasLength(1));
    });

    test('does not stop the others finishing', () async {
      final faults = <Object>[];
      final tasks = BackgroundTasks(onError: (error, _) => faults.add(error));
      var finished = 0;

      tasks.run('bad', () async => throw StateError('boom'));
      tasks.run('good', () async => finished++);

      expect(await tasks.settled(const Duration(seconds: 1)), isTrue);
      expect(finished, 1);
      expect(faults, hasLength(1));
    });
  });

  group('discovery from state', () {
    test('serve finds a registry attached with withState', () async {
      // The only way a clustered server can drain one: each isolate builds its
      // registry inside the factory, so nothing outside can hand it in.
      final tasks = BackgroundTasks(onError: (_, __) {});
      final gate = Completer<void>();
      var finished = false;

      final app = Router()
        ..route('/go', post((request) async {
          (await request.state<BackgroundTasks>()).run('late', () async {
            await gate.future;
            finished = true;
          });
          return {'ok': true};
        }))
        ..withState(tasks);

      // No `background:` argument.
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);

      await http.post(
        Uri.parse('http://${server.address.host}:${server.port}/go'),
      );

      expect(server.pendingTasks, 1);

      Timer(const Duration(milliseconds: 30), gate.complete);
      expect(await server.close(drain: const Duration(seconds: 2)), isTrue);
      expect(finished, isTrue);
    });

    test('an explicit registry wins over one in state', () async {
      final passed = BackgroundTasks(onError: (_, __) {});
      final attached = BackgroundTasks(onError: (_, __) {});
      final app = Router()
        ..route('/', get((request) async => const {'ok': true}))
        ..withState(attached);

      final server = await serve(
        app,
        InternetAddress.loopbackIPv4,
        0,
        background: passed,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      passed.run('one', () => Completer<void>().future);

      expect(server.pendingTasks, 1, reason: 'the passed registry is the one');
    });

    test('no registry anywhere leaves pendingTasks at zero', () async {
      final server = await serve(
        Router()..route('/', get((request) async => const {'ok': true})),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      expect(server.pendingTasks, 0);
    });
  });

  group('tracing', () {
    test('a task does not inherit the request span', () async {
      // The span ends when the response goes out. A task that kept it would
      // write attributes onto a finished, already exported span — and race
      // another task doing the same on one map.
      final exported = <Span>[];
      final tasks = BackgroundTasks(onError: (_, __) {});
      final gate = Completer<void>();
      Span? seenInTask;

      final app = Router()
        ..layer(Tracing(_Exporter(exported), serviceName: 'test'))
        ..route('/go', post((request) async {
          (await request.state<BackgroundTasks>()).run('late', () async {
            await gate.future;
            seenInTask = CurrentSpan.value;
            CurrentSpan.setAttribute('written.after.request', true);
          });
          return {'ok': true};
        }))
        ..withState(tasks);

      final server = await serve(
        app,
        InternetAddress.loopbackIPv4,
        0,
        background: tasks,
      );
      await http.post(
        Uri.parse('http://${server.address.host}:${server.port}/go'),
      );

      expect(exported.single.isFinished, isTrue,
          reason: 'span ends with the response');

      gate.complete();
      await server.close(drain: const Duration(seconds: 2));

      expect(seenInTask, isNull);
      expect(
        exported.single.attributes.containsKey('written.after.request'),
        isFalse,
      );
    });

    test('a handler still has its span, so detaching is scoped to the task',
        () async {
      final exported = <Span>[];
      final tasks = BackgroundTasks(onError: (_, __) {});

      final app = Router()
        ..layer(Tracing(_Exporter(exported), serviceName: 'test'))
        ..route('/go', post((request) async {
          CurrentSpan.setAttribute('from.handler', true);
          (await request.state<BackgroundTasks>()).run('noop', () async {});
          return {'ok': true};
        }))
        ..withState(tasks);

      final server = await serve(
        app,
        InternetAddress.loopbackIPv4,
        0,
        background: tasks,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      await http.post(
        Uri.parse('http://${server.address.host}:${server.port}/go'),
      );

      expect(exported.single.attributes['from.handler'], isTrue);
    });
  });

  group('closing', () {
    test('refuses new work once it has begun', () async {
      // Better to know a task never ran than to half-run it during shutdown.
      final tasks = BackgroundTasks();
      await tasks.close(within: const Duration(milliseconds: 10));

      var ran = false;
      final accepted = tasks.run('late', () async => ran = true);

      expect(accepted, isFalse);
      expect(tasks.isClosed, isTrue);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(ran, isFalse);
    });

    test('waits for what was already accepted', () async {
      final tasks = BackgroundTasks();
      var finished = false;
      tasks.run('slow', () async {
        await Future<void>.delayed(const Duration(milliseconds: 50));
        finished = true;
      });

      expect(await tasks.close(within: const Duration(seconds: 2)), isTrue);
      expect(finished, isTrue);
    });
  });

  group('a served application', () {
    Future<({ServerHandle server, BackgroundTasks tasks})> startServer(
      Completer<void> gate,
      List<String> done,
    ) async {
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = Router()
        ..route('/checkout', post((request) async {
          final registry = await request.state<BackgroundTasks>();
          registry.run('receipt', () async {
            await gate.future;
            done.add('receipt');
          });
          return {'placed': true};
        }))
        ..withState(tasks);

      final server = await serve(
        app,
        InternetAddress.loopbackIPv4,
        0,
        background: tasks,
      );
      return (server: server, tasks: tasks);
    }

    test('the response does not wait for the task', () async {
      // The whole point of backgrounding it.
      final gate = Completer<void>();
      final done = <String>[];
      final served = await startServer(gate, done);

      final response = await http.post(
        Uri.parse('http://${served.server.address.host}:'
            '${served.server.port}/checkout'),
      );

      expect(response.statusCode, 200);
      expect(done, isEmpty);
      expect(served.server.pendingTasks, 1);

      gate.complete();
      await served.server.close(drain: const Duration(seconds: 2));
      expect(done, ['receipt']);
    });

    test('shutdown waits for a task the request left running', () async {
      // Before this existed the task was simply killed, silently.
      final gate = Completer<void>();
      final done = <String>[];
      final served = await startServer(gate, done);

      await http.post(
        Uri.parse('http://${served.server.address.host}:'
            '${served.server.port}/checkout'),
      );

      Timer(const Duration(milliseconds: 50), gate.complete);
      final settled = await served.server.close(
        drain: const Duration(seconds: 2),
      );

      expect(settled, isTrue);
      expect(done, ['receipt']);
    });

    test('shutdown reports false when a task outlasts the budget', () async {
      final gate = Completer<void>();
      final done = <String>[];
      final served = await startServer(gate, done);

      await http.post(
        Uri.parse('http://${served.server.address.host}:'
            '${served.server.port}/checkout'),
      );

      final settled = await served.server.close(
        drain: const Duration(milliseconds: 100),
      );

      expect(settled, isFalse, reason: 'the task never finished');
      gate.complete();
    });

    test('pendingTasks is zero without a registry', () async {
      final server = await serve(
        Router()..route('/', get((request) async => const {'ok': true})),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      expect(server.pendingTasks, 0);
    });
  });
}

/// Keeps every span so a test can look at one.
final class _Exporter implements SpanExporter {
  const _Exporter(this.into);

  final List<Span> into;

  @override
  void export(Span span) => into.add(span);
}
