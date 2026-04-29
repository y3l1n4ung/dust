import 'package:derive_serde_annotation/derive_serde_annotation.dart';
import 'package:test/test.dart';

void main() {
  group('Serde derive traits', () {
    test('serde markers are derive traits', () {
      const traits = <DeriveTrait>[Serialize(), Deserialize()];

      expect(traits, hasLength(2));
    });

    test('package re-exports the Derive container', () {
      const annotation = Derive([Serialize(), Deserialize()]);

      expect(annotation.traits, hasLength(2));
      expect(annotation.traits[0], isA<Serialize>());
      expect(annotation.traits[1], isA<Deserialize>());
    });
  });

  group('SerDe config', () {
    test('defaults are stable and serde-like', () {
      const config = SerDe();

      expect(config.rename, isNull);
      expect(config.renameAll, isNull);
      expect(config.defaultValue, isNull);
      expect(config.skip, isFalse);
      expect(config.skipSerializing, isFalse);
      expect(config.skipDeserializing, isFalse);
      expect(config.aliases, isEmpty);
      expect(config.disallowUnrecognizedKeys, isFalse);
      expect(config, isA<DeriveConfig>());
    });

    test('custom config values are preserved', () {
      const config = SerDe(
        rename: 'payload',
        renameAll: SerDeRename.snakeCase,
        defaultValue: 42,
        skip: false,
        skipSerializing: true,
        skipDeserializing: false,
        aliases: ['payload_v1', 'payload_v0'],
        disallowUnrecognizedKeys: true,
      );

      expect(config.rename, 'payload');
      expect(config.renameAll, SerDeRename.snakeCase);
      expect(config.defaultValue, 42);
      expect(config.skip, isFalse);
      expect(config.skipSerializing, isTrue);
      expect(config.skipDeserializing, isFalse);
      expect(config.aliases, ['payload_v1', 'payload_v0']);
      expect(config.disallowUnrecognizedKeys, isTrue);
    });
  });

  group('field-level serde metadata', () {
    test('field config preserves rename and skip flags', () {
      const field = SerDe(
        rename: 'user_id',
        skipSerializing: true,
        skipDeserializing: false,
      );

      expect(field.rename, 'user_id');
      expect(field.skip, isFalse);
      expect(field.skipSerializing, isTrue);
      expect(field.skipDeserializing, isFalse);
      expect(field.defaultValue, isNull);
      expect(field, isA<DeriveMeta>());
    });

    test('field config can carry aliases and a default value', () {
      const field = SerDe(
        aliases: ['old_name'],
        defaultValue: <String>['guest'],
      );

      expect(field.aliases, ['old_name']);
      expect(field.defaultValue, isA<List<String>>());
      expect(field.rename, isNull);
    });
  });

  test('rename enum exposes stable strategies', () {
    expect(
      SerDeRename.values,
      equals(const [
        SerDeRename.lowerCase,
        SerDeRename.upperCase,
        SerDeRename.pascalCase,
        SerDeRename.camelCase,
        SerDeRename.snakeCase,
        SerDeRename.screamingSnakeCase,
        SerDeRename.kebabCase,
        SerDeRename.screamingKebabCase,
      ]),
    );
  });
}
