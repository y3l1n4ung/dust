import 'dart:async';

import 'package:dust_server/server.dart';
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
}
