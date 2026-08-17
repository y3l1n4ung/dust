@Tags(['stress'])
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';

import 'support/server.dart';

/// Sustained load against the real thing: a real socket, a real database, the
/// whole layer stack. Everything else in this suite sends one request at a
/// time, which is exactly the shape that never finds a shared-state bug.
///
/// These are slower than the rest, so they carry the `stress` tag:
///
/// ```bash
/// dart test -t stress
/// dart test -x stress   # everything else
/// ```

/// One client's worth of work, using its own connection.
Future<List<int>> _hammer(
  String origin,
  int count, {
  required String token,
  String path = '/api/v1/todos',
}) async {
  final client = HttpClient();
  final statuses = <int>[];
  try {
    for (var index = 0; index < count; index++) {
      final request = await client.getUrl(Uri.parse('$origin$path'));
      request.headers.set('authorization', 'Bearer $token');
      final response = await request.close();
      await response.drain<void>();
      statuses.add(response.statusCode);
    }
  } finally {
    client.close(force: true);
  }
  return statuses;
}

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.startWithSqlite());
  tearDown(() => server.stop());

  group('sustained reads', () {
    test('answers every one of 600 concurrent requests', () async {
      const clients = 20;
      const each = 30;

      final started = DateTime.now();
      final results = await Future.wait([
        for (var index = 0; index < clients; index++)
          _hammer(server.origin, each, token: readToken),
      ]);
      final elapsed = DateTime.now().difference(started);

      final statuses = results.expand((batch) => batch).toList();
      expect(statuses, hasLength(clients * each));
      expect(statuses.every((status) => status == 200), isTrue);
      expect(server.errors, isEmpty);

      printOnFailure('$statuses');
      // ignore: avoid_print
      print(
        '${statuses.length} reads in ${elapsed.inMilliseconds}ms '
        '(${(statuses.length * 1000 / elapsed.inMilliseconds).round()}/s)',
      );
    }, timeout: const Timeout(Duration(minutes: 2)));

    test('logs exactly one record per request', () async {
      await Future.wait([
        for (var index = 0; index < 10; index++)
          _hammer(server.origin, 10, token: readToken),
      ]);

      expect(server.records, hasLength(100));
    }, timeout: const Timeout(Duration(minutes: 2)));
  });

  group('concurrent writes', () {
    test('stores every row exactly once', () async {
      const writers = 15;
      const each = 20;

      await Future.wait([
        for (var writer = 0; writer < writers; writer++)
          Future(() async {
            for (var index = 0; index < each; index++) {
              final response = await server.post(
                '/api/v1/todos',
                validBody(title: 'w$writer-$index'),
              );
              expect(response.statusCode, 201);
            }
          }),
      ]);

      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      // One seeded row plus everything written, with no lost update and no
      // double insert.
      expect(listed, hasLength(writers * each + 1));
      expect(server.errors, isEmpty);
    }, timeout: const Timeout(Duration(minutes: 2)));

    test('gives every row a distinct id under contention', () async {
      await Future.wait([
        for (var writer = 0; writer < 10; writer++)
          Future(() async {
            for (var index = 0; index < 10; index++) {
              await server.post('/api/v1/todos', validBody());
            }
          }),
      ]);

      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;
      final ids = [for (final row in listed) (row! as Map)['id']];

      expect(ids.toSet(), hasLength(ids.length));
    }, timeout: const Timeout(Duration(minutes: 2)));
  });

  group('mixed traffic', () {
    test('keeps each caller inside their own rows', () async {
      // The failure this looks for is state leaking between concurrent
      // requests: one caller seeing another's todos because something was
      // shared that should not have been.
      await Future.wait([
        for (var index = 0; index < 20; index++)
          server.post('/api/v1/todos', validBody(assignTo: owner)),
        for (var index = 0; index < 20; index++)
          server.post(
            '/api/v1/todos',
            validBody(assignTo: other),
            token: adminToken,
          ),
      ]);

      final results = await Future.wait([
        for (var index = 0; index < 25; index++) server.get('/api/v1/todos'),
        for (var index = 0; index < 25; index++)
          server.get('/api/v1/todos', token: otherToken),
      ]);

      for (final response in results.take(25)) {
        final rows = jsonDecode(response.body)! as List;
        expect(rows.every((row) => (row! as Map)['owner'] == owner), isTrue);
      }
      for (final response in results.skip(25)) {
        final rows = jsonDecode(response.body)! as List;
        expect(rows.every((row) => (row! as Map)['owner'] == other), isTrue);
      }
    }, timeout: const Timeout(Duration(minutes: 2)));

    test('answers failures and successes side by side', () async {
      final results = await Future.wait([
        for (var index = 0; index < 40; index++)
          index.isEven
              ? server.get('/api/v1/todos')
              : server.get('/api/v1/todos', token: null),
      ]);

      final statuses = [for (final response in results) response.statusCode];
      expect(statuses.where((status) => status == 200), hasLength(20));
      expect(statuses.where((status) => status == 401), hasLength(20));
      expect(server.errors, isEmpty);
    }, timeout: const Timeout(Duration(minutes: 2)));
  });

  group('shutting down under load', () {
    // What draining promises is narrower than it first sounds: a request the
    // server has *accepted* runs to completion, and a connection opened after
    // the close does not. A client mid-send when the listener stops sees the
    // connection refused — correctly. Asserting otherwise would be asserting
    // that a stopped server still accepts work.
    test('finishes what it accepted and refuses the rest', () async {
      final sent = [
        for (var index = 0; index < 40; index++)
          server
              .post('/api/v1/todos', validBody(title: 'drain-$index'))
              .then<int?>((response) => response.statusCode)
              // A refused connection is the expected outcome for anything that
              // had not reached the server yet.
              .onError<Object>((_, __) => null),
      ];

      // Let some of them land, then stop accepting.
      await Future<void>.delayed(const Duration(milliseconds: 20));
      final accepted = server.inFlight;
      await server.stop();

      final statuses = await Future.wait(sent);
      final completed = statuses.whereType<int>().toList();

      // Everything the server answered, it answered properly.
      expect(completed, everyElement(201));
      expect(completed, isNotEmpty);
      expect(server.errors, isEmpty);

      // ignore: avoid_print
      print(
        '${completed.length}/40 completed across a shutdown '
        '($accepted in flight when it began)',
      );
    }, timeout: const Timeout(Duration(minutes: 2)));

    test('persists exactly what it answered for', () async {
      // The row count has to match the responses: a 201 the client never sees
      // is a lost write, and a row with no 201 is a phantom.
      //
      // A file rather than `:memory:`, because the check happens after the
      // server closed its connection and an in-memory database dies with it.
      final directory = await Directory.systemTemp.createTemp('todo_drain');
      addTearDown(() => directory.delete(recursive: true));
      final path = '${directory.path}/todos.db';

      final second =
          await ExampleServer.start(store: SqliteTodoStore.open(path));

      final sent = [
        for (var index = 0; index < 30; index++)
          second
              .post('/api/v1/todos', validBody(title: 'row-$index'))
              .then<int?>((response) => response.statusCode)
              .onError<Object>((_, __) => null),
      ];

      await Future<void>.delayed(const Duration(milliseconds: 20));
      await second.stop();

      final answered = (await Future.wait(sent)).whereType<int>().length;

      // Re-open the same file: whatever survived the shutdown is on disk.
      final reopened = SqliteTodoStore.open(path);
      final stored = await reopened.all();
      await reopened.close();

      expect(stored, hasLength(answered));
    }, timeout: const Timeout(Duration(minutes: 2)));
  });
}
