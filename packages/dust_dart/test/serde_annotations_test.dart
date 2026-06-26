import 'package:dust_dart/serde.dart';
import 'package:test/test.dart';

void main() {
  test('serde derive markers and config preserve options', () {
    const serialize = Serialize();
    const deserialize = Deserialize();
    const config = SerDe(
      rename: 'display_name',
      renameAll: SerDeRename.snakeCase,
      tag: 'type',
      content: 'data',
      untagged: true,
      defaultValue: 'anon',
      skip: true,
      skipSerializing: true,
      skipDeserializing: true,
      aliases: ['displayName'],
      using: _StringCodec(),
      disallowUnrecognizedKeys: true,
    );

    expect(serialize, isA<Serialize>());
    expect(deserialize, isA<Deserialize>());
    expect(config.rename, 'display_name');
    expect(config.renameAll, SerDeRename.snakeCase);
    expect(config.tag, 'type');
    expect(config.content, 'data');
    expect(config.untagged, isTrue);
    expect(config.defaultValue, 'anon');
    expect(config.skip, isTrue);
    expect(config.skipSerializing, isTrue);
    expect(config.skipDeserializing, isTrue);
    expect(config.aliases, ['displayName']);
    expect(config.using, isA<_StringCodec>());
    expect(config.disallowUnrecognizedKeys, isTrue);
  });
}

final class _StringCodec implements SerDeCodec<String, String> {
  const _StringCodec();

  @override
  String serialize(String value) => value;

  @override
  String deserialize(String value) => value;
}
