import 'dart:convert';

import 'package:dust_dart/serde.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// `jsonResponse` installs its own `toEncodable`, which replaces the one
/// `jsonEncode` would have used. That makes every type it has to handle its
/// responsibility: Dust models through `serialize`, the few core types a JSON
/// document cannot carry, and anything outside Dust that still offers
/// `toJson`. A gap here is a runtime failure on a response nobody tested.

final class _Model implements Serializable {
  const _Model(this.name);

  final String name;

  @override
  Map<String, Object?> serialize() => <String, Object?>{'name': name};

  @override
  Map<String, Object?> toJson() => throw StateError('serialize must win');
}

/// A type from outside Dust: no `Serializable`, but the Dart convention.
final class _Legacy {
  const _Legacy(this.id);

  final int id;

  Map<String, Object?> toJson() => <String, Object?>{'id': id};
}

final class _Opaque {
  const _Opaque();
}

Future<Object?> decode(Response response) async =>
    jsonDecode(await response.readAsString());

void main() {
  group('a Dust model', () {
    test('is written through serialize, not toJson', () async {
      expect(
          await decode(jsonResponse(const _Model('dust'))), {'name': 'dust'});
    });

    test('is written through serialize when nested', () async {
      expect(
        await decode(jsonResponse({'inner': const _Model('dust')})),
        {
          'inner': {'name': 'dust'},
        },
      );
    });

    test('is written through serialize inside a list', () async {
      expect(
        await decode(jsonResponse(const [_Model('a'), _Model('b')])),
        [
          {'name': 'a'},
          {'name': 'b'},
        ],
      );
    });
  });

  group('core types JSON cannot carry', () {
    test('a DateTime becomes an ISO-8601 string', () async {
      final moment = DateTime.utc(2026, 8, 16, 12, 30);

      expect(await decode(jsonResponse(moment)), '2026-08-16T12:30:00.000Z');
    });

    test('a DateTime nested in a model round-trips as text', () async {
      final moment = DateTime.utc(2026, 1, 2);

      expect(
        await decode(jsonResponse({'at': moment})),
        {'at': '2026-01-02T00:00:00.000Z'},
      );
    });

    test('a Uri becomes its string form', () async {
      expect(
        await decode(jsonResponse(Uri.parse('https://dust.test/a?b=1'))),
        'https://dust.test/a?b=1',
      );
    });
  });

  group('a type from outside Dust', () {
    test('falls back to toJson, as jsonEncode would have', () async {
      expect(await decode(jsonResponse(const _Legacy(7))), {'id': 7});
    });
  });

  group('a type that can not be encoded', () {
    test('is reported as unsupported rather than as a missing method', () {
      expect(
        () => jsonResponse(const _Opaque()),
        throwsA(isA<JsonUnsupportedObjectError>()),
      );
    });

    test('names the value that could not be encoded', () {
      expect(
        () => jsonResponse(const _Opaque()),
        throwsA(
          isA<JsonUnsupportedObjectError>().having(
            (error) => error.unsupportedObject,
            'unsupportedObject',
            isA<_Opaque>(),
          ),
        ),
      );
    });
  });
}
