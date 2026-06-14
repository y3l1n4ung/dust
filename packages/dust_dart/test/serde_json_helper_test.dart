import 'package:dust_dart/serde.dart';
import 'package:test/test.dart';

final class _IntCodec implements SerDeCodec<int, String> {
  const _IntCodec();

  @override
  String serialize(int value) => value.toString();

  @override
  int deserialize(String value) => int.parse(value);
}

void main() {
  group('JsonHelper', () {
    test('reads scalar values', () {
      expect(JsonHelper.as<String>('Ada', 'name', 'String'), 'Ada');
      expect(JsonHelper.as<int>(42, 'age', 'int'), 42);
    });

    test('reports type errors with the json key', () {
      expect(
        () => JsonHelper.as<String>(7, 'name', 'String'),
        throwsA(
          isA<ArgumentError>()
              .having((error) => error.name, 'name', 'name')
              .having((error) => error.invalidValue, 'invalidValue', 7),
        ),
      );
    });

    test('reads list and map values', () {
      expect(JsonHelper.asList(<Object?>['a'], 'tags'), <Object?>['a']);
      expect(
        JsonHelper.asMap(<String, Object?>{'id': 1}, 'payload'),
        <String, Object?>{'id': 1},
      );
    });

    test('rejects maps with non-string keys', () {
      expect(
        () => JsonHelper.asMap(<Object?, Object?>{1: 'bad'}, 'payload'),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('parses string-backed builtins', () {
      expect(
        JsonHelper.asDateTime('2026-06-04T00:00:00Z', 'createdAt'),
        DateTime.parse('2026-06-04T00:00:00Z'),
      );
      expect(
        JsonHelper.asUri('https://example.com', 'url').host,
        'example.com',
      );
      expect(
        JsonHelper.asBigInt('9007199254740993', 'id'),
        BigInt.parse('9007199254740993'),
      );
      expect(
        () => JsonHelper.asDateTime('not-a-date', 'createdAt'),
        throwsA(isA<ArgumentError>()),
      );
      expect(
        () => JsonHelper.asUri('http://[', 'url'),
        throwsA(isA<ArgumentError>()),
      );
      expect(
        () => JsonHelper.asBigInt('nan', 'id'),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('decodes with serde codec', () {
      const codec = _IntCodec();

      expect(codec.serialize(42), '42');
      expect(JsonHelper.decodeWithCodec<int>(codec, '42', 'id'), 42);
    });

    test('rejects null codec values', () {
      expect(
        () => JsonHelper.decodeWithCodec<int>(const _IntCodec(), null, 'id'),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('wraps serde codec failures', () {
      expect(
        () => JsonHelper.decodeWithCodec<int>(const _IntCodec(), 'nan', 'id'),
        throwsA(isA<ArgumentError>()),
      );
    });
  });
}
