import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:notes_mvc/app.dart';
import 'package:test/test.dart';

/// The whole example is a model, a controller, and five routes. These drive it
/// over a real socket against a real database, because that is the only way to
/// know the three fit together.

void main() {
  late ServerHandle server;
  late NotesDatabase database;

  setUp(() async {
    database = NotesDatabase.open(':memory:');
    server = await serveRouter(
      buildApp(Notes(database.connection)),
      InternetAddress.loopbackIPv4,
      0,
    );
  });

  tearDown(() async {
    await server.close(drain: const Duration(seconds: 2));
    await database.connection.close();
  });

  String origin() => 'http://${server.address.host}:${server.port}';

  Future<http.Response> send(
    String method,
    String path, [
    Object? body,
  ]) {
    final uri = Uri.parse('${origin()}$path');
    const headers = {'content-type': 'application/json'};
    return switch (method) {
      'GET' => http.get(uri),
      'POST' => http.post(uri, headers: headers, body: jsonEncode(body)),
      'PUT' => http.put(uri, headers: headers, body: jsonEncode(body)),
      _ => http.delete(uri),
    };
  }

  Map<String, Object?> objectOf(http.Response response) =>
      (jsonDecode(response.body) as Map).cast<String, Object?>();

  Future<String> create({String title = 'first', String body = 'text'}) async {
    final response =
        await send('POST', '/notes', {'title': title, 'body': body});
    expect(response.statusCode, 201);
    return '${objectOf(response)['id']}';
  }

  group('the five routes', () {
    test('index starts empty', () async {
      expect(jsonDecode((await send('GET', '/notes')).body), isEmpty);
    });

    test('create answers 201 with the stored note', () async {
      final response =
          await send('POST', '/notes', {'title': 'buy milk', 'body': 'today'});

      expect(response.statusCode, 201);
      expect(objectOf(response), {
        'id': 1,
        'title': 'buy milk',
        'body': 'today',
      });
    });

    test('index lists what was created', () async {
      await create(title: 'a');
      await create(title: 'b');

      final listed = jsonDecode((await send('GET', '/notes')).body)! as List;

      expect([for (final note in listed) (note! as Map)['title']], ['a', 'b']);
    });

    test('show reads one back', () async {
      final id = await create(title: 'read me');

      expect(objectOf(await send('GET', '/notes/$id'))['title'], 'read me');
    });

    test('replace overwrites and answers the new note', () async {
      final id = await create();
      final response =
          await send('PUT', '/notes/$id', {'title': 'new', 'body': 'words'});

      expect(objectOf(response), {'id': 1, 'title': 'new', 'body': 'words'});
      expect(objectOf(await send('GET', '/notes/$id'))['title'], 'new');
    });

    test('destroy answers 204 and stops serving it', () async {
      final id = await create();

      expect((await send('DELETE', '/notes/$id')).statusCode, 204);
      expect((await send('GET', '/notes/$id')).statusCode, 404);
    });

    test('health answers outside the notes prefix', () async {
      expect(objectOf(await send('GET', '/health')), {'ok': true});
    });
  });

  group('failures', () {
    test('show answers 404 for a note that is not there', () async {
      expect((await send('GET', '/notes/999')).statusCode, 404);
    });

    test('replace answers 404 rather than creating one', () async {
      final response =
          await send('PUT', '/notes/999', {'title': 'x', 'body': 'y'});

      expect(response.statusCode, 404);
      expect(jsonDecode((await send('GET', '/notes')).body), isEmpty);
    });

    test('destroy answers 404 for a note that is not there', () async {
      expect((await send('DELETE', '/notes/999')).statusCode, 404);
    });

    test('an id that is not a number is a 400, before any query', () async {
      expect((await send('GET', '/notes/abc')).statusCode, 400);
    });

    test('a title that breaks its constraint is a 422 naming it', () async {
      final response = await send('POST', '/notes', {'title': '', 'body': 'x'});

      expect(response.statusCode, 422);
      expect(
        (objectOf(response)['fields']! as Map)['title'],
        ['must be 1 to 120 characters'],
      );
    });

    test('a body of the wrong shape is a 422 with no field to blame', () async {
      final response = await send('POST', '/notes', {'title': 7});

      expect(response.statusCode, 422);
      expect(objectOf(response), isNot(contains('fields')));
    });

    test('an absent body field takes its declared default', () async {
      final response = await send('POST', '/notes', {'title': 'no body key'});

      expect(objectOf(response)['body'], '');
    });

    test('a method the path does not serve is a 405', () async {
      expect((await send('PUT', '/notes', {'title': 'x'})).statusCode, 405);
    });

    test('every response carries a request id', () async {
      final response = await send('GET', '/health');

      expect(response.headers['x-request-id'], isNotNull);
    });
  });
}
