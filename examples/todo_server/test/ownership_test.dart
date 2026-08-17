import 'package:test/test.dart';

import 'support/server.dart';

/// An authenticating extractor returns the caller, and what the caller is
/// decides what they see. A guard whose result is thrown away checks only that
/// someone is logged in — these check that it is the right someone.
///
/// The rule: your own todos always, anyone else's only with `todos:admin`.
/// Something you may not see answers 404 rather than 403, because telling you
/// it exists is already more than you should learn.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.start());
  tearDown(() => server.stop());

  Future<int> createFor(String assignTo, {String token = adminToken}) async {
    final created = objectOf(
      await server.post(
        '/api/v1/todos',
        validBody(assignTo: assignTo),
        token: token,
      ),
    );
    return created['id']! as int;
  }

  group('who the caller is', () {
    test('comes back from the extractor', () async {
      final me = objectOf(await server.get('/api/v1/me'));

      expect(me['id'], owner);
    });

    test('carries the scopes the token named', () async {
      final me = objectOf(await server.get('/api/v1/me', token: writeToken));

      expect(me['scopes'], ['todos:read', 'todos:write']);
    });
  });

  group('listing', () {
    test('shows only the caller their own todos', () async {
      await createFor(other);

      final mine = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(mine, hasLength(1));
      expect((mine.single! as Map)['owner'], owner);
    });

    test('shows another user only theirs', () async {
      await createFor(other);

      final theirs =
          bodyOf(await server.get('/api/v1/todos', token: otherToken))! as List;

      expect(theirs, hasLength(1));
      expect((theirs.single! as Map)['owner'], other);
    });

    test('shows an admin everything', () async {
      await createFor(other);

      final all =
          bodyOf(await server.get('/api/v1/todos', token: adminToken))! as List;

      expect(all, hasLength(2));
    });

    test('still honours the done filter inside the caller scope', () async {
      final id = await createFor(owner);
      await server.patch('/api/v1/todos/$id?done=true');

      final done = bodyOf(await server.get('/api/v1/todos?done=true'))! as List;

      expect(done, hasLength(1));
    });
  });

  group('reading one', () {
    test('answers 404 for a todo belonging to someone else', () async {
      final id = await createFor(other);

      expect((await server.get('/api/v1/todos/$id')).statusCode, 404);
    });

    test('does not leak that it exists', () async {
      final id = await createFor(other);
      final hidden = await server.get('/api/v1/todos/$id');
      final absent = await server.get('/api/v1/todos/999999');

      expect(hidden.statusCode, absent.statusCode);
      expect(hidden.body, absent.body);
    });

    test('lets an admin read it', () async {
      final id = await createFor(other);

      expect(
        (await server.get('/api/v1/todos/$id', token: adminToken)).statusCode,
        200,
      );
    });
  });

  group('creating', () {
    test('assigns to the caller without a special scope', () async {
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(assignTo: owner)),
      );

      expect(created['owner'], owner);
    });

    test('refuses to assign to someone else', () async {
      final response = await server.post(
        '/api/v1/todos',
        validBody(assignTo: other),
      );

      expect(response.statusCode, 403);
      expect(
        objectOf(response)['error'],
        'assigning to another user needs todos:admin',
      );
    });

    test('lets an admin assign to anyone', () async {
      final created = objectOf(
        await server.post(
          '/api/v1/todos',
          validBody(assignTo: other),
          token: adminToken,
        ),
      );

      expect(created['owner'], other);
    });

    test('checks the scope before the ownership rule', () async {
      // A read-only token is 403 for lacking `todos:write`, not for the
      // assignment, so the extractor's own failure wins.
      final response = await server.post(
        '/api/v1/todos',
        validBody(assignTo: other),
        token: readToken,
      );

      expect(objectOf(response)['error'], 'requires scope todos:write');
    });
  });

  group('changing and removing', () {
    test("refuses to complete someone else's todo", () async {
      final id = await createFor(other);

      expect(
        (await server.patch('/api/v1/todos/$id?done=true')).statusCode,
        404,
      );
    });

    test('leaves it untouched when it refused', () async {
      final id = await createFor(other);
      await server.patch('/api/v1/todos/$id?done=true');

      final still = objectOf(
        await server.get('/api/v1/todos/$id', token: adminToken),
      );

      expect(still['done'], isFalse);
    });

    test("refuses to delete someone else's todo", () async {
      final id = await createFor(other);

      expect((await server.delete('/api/v1/todos/$id')).statusCode, 404);
    });

    test('leaves it in place when it refused', () async {
      final id = await createFor(other);
      await server.delete('/api/v1/todos/$id');

      expect(
        (await server.get('/api/v1/todos/$id', token: adminToken)).statusCode,
        200,
      );
    });

    test('lets an admin delete it', () async {
      final id = await createFor(other);

      expect(
        (await server.delete('/api/v1/todos/$id', token: adminToken))
            .statusCode,
        204,
      );
    });
  });
}
