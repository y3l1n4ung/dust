import 'dart:convert';

import 'package:dust_dart/serde.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// `responseFrom` is the rule that lets a handler return a model instead of a
/// response. Every row of its table is here, because a wrong row means an
/// endpoint answers with the wrong status and nothing else notices.

final class _Model implements Serializable {
  const _Model(this.name);

  final String name;

  @override
  Map<String, Object?> serialize() => <String, Object?>{'name': name};

  @override
  Map<String, Object?> toJson() => serialize();
}

final class _Teapot implements IntoResponse {
  const _Teapot();

  @override
  Response intoResponse() => Response(418, body: 'short and stout');
}

void main() {
  group('responseFrom', () {
    test('passes a Response through untouched', () {
      final original = Response(207, body: 'multi');

      expect(responseFrom(original), same(original));
    });

    test('asks an IntoResponse to build its own', () {
      expect(responseFrom(const _Teapot()).statusCode, 418);
    });

    test('answers 204 for null', () {
      expect(responseFrom(null).statusCode, 204);
    });

    test('serializes a model through the derive, not toString', () async {
      final response = responseFrom(const _Model('dust'));

      expect(jsonDecode(await response.readAsString()), {'name': 'dust'});
    });

    test('serializes every element of a list', () async {
      final response = responseFrom(const [_Model('a'), _Model('b')]);

      expect(jsonDecode(await response.readAsString()), [
        {'name': 'a'},
        {'name': 'b'},
      ]);
    });

    test('applies the success status to a plain value', () {
      expect(responseFrom(const _Model('dust'), status: 201).statusCode, 201);
    });

    test('unwraps Ok and keeps the success status', () {
      final response = responseFrom(
        const Ok<_Model, Rejection>(_Model('dust')),
        status: 201,
      );

      expect(response.statusCode, 201);
    });

    test('unwraps Err to the rejection it carries', () {
      final response = responseFrom(
        const Err<_Model, Rejection>(Rejection.notFound('gone')),
      );

      expect(response.statusCode, 404);
    });

    test('does not let the success status leak onto an Err', () {
      final response = responseFrom(
        const Err<_Model, Rejection>(Rejection.notFound('gone')),
        status: 201,
      );

      expect(response.statusCode, 404);
    });

    test('answers 500 for an Err with no opinion about status', () {
      final response = responseFrom(const Err<_Model, String>('boom'));

      expect(response.statusCode, 500);
    });

    test('unwraps Some to the value inside', () async {
      final response = responseFrom(const Some<_Model>(_Model('dust')));

      expect(jsonDecode(await response.readAsString()), {'name': 'dust'});
    });

    test('answers 404 for None', () {
      expect(responseFrom(const None<_Model>()).statusCode, 404);
    });

    test('encodes a bare map as JSON', () async {
      final response = responseFrom({'ok': true});

      expect(jsonDecode(await response.readAsString()), {'ok': true});
    });

    test('sends a string as text, not as a quoted JSON string', () async {
      // axum's rule. Encoding it gave `"hello"` with the quotes, which is right
      // only if you wanted a JSON string and a surprise every other time.
      final response = responseFrom('hello');

      expect(await response.readAsString(), 'hello');
      expect(response.headers['content-type'], startsWith('text/plain'));
    });

    test('sends a byte stream rather than trying to encode it', () async {
      final response = responseFrom(
        Stream<List<int>>.fromIterable([
          [104, 105],
        ]),
      );

      expect(response.statusCode, 200);
      expect(response.headers['content-type'], 'application/octet-stream');
      expect(response.context['shelf.io.buffer_output'], false);
      expect(await response.readAsString(), 'hi');
    });

    test('a list of strings is still JSON', () async {
      final response = responseFrom(const ['a', 'b']);

      expect(await response.readAsString(), '["a","b"]');
      expect(response.headers['content-type'], 'application/json');
    });

    test('the status still applies to a string', () async {
      expect(responseFrom('made', status: 201).statusCode, 201);
    });
  });
}
