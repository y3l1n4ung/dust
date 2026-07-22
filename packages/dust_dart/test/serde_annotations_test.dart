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

  test('serde source and mirror contracts are usable', () {
    const user = _SerializableUser('u1');
    const serializer = _UserSerializer();
    const deserializer = _UserDeserializer();

    expect(user.serialize(), {'id': 'u1'});
    expect(user.toJson(), {'id': 'u1'});
    expect(serializer.serialize(user), {'id': 'u1'});
    expect(serializer.toJson(user), {'id': 'u1'});
    expect(
        deserializer.deserialize({'id': 'u2'}), const _SerializableUser('u2'));
    expect(deserializer.fromJson({'id': 'u3'}), const _SerializableUser('u3'));

    const codec = _StringCodec();
    expect(codec.serialize('dust'), 'dust');
    expect(codec.toJson('dust'), 'dust');
    expect(codec.deserialize('dust'), 'dust');
    expect(codec.fromJson('dust'), 'dust');
  });
}

final class _SerializableUser implements Serializable {
  const _SerializableUser(this.id);

  final String id;

  @override
  Map<String, Object?> serialize() => {'id': id};

  @override
  Map<String, Object?> toJson() => serialize();

  @override
  bool operator ==(Object other) =>
      other is _SerializableUser && other.id == id;

  @override
  int get hashCode => id.hashCode;
}

final class _UserSerializer
    implements Serializer<_SerializableUser, Map<String, Object?>> {
  const _UserSerializer();

  @override
  Map<String, Object?> serialize(_SerializableUser value) => value.serialize();

  @override
  Map<String, Object?> toJson(_SerializableUser value) => serialize(value);
}

final class _UserDeserializer
    implements Deserializer<_SerializableUser, Map<String, Object?>> {
  const _UserDeserializer();

  @override
  _SerializableUser deserialize(Map<String, Object?> json) =>
      _SerializableUser(JsonHelper.as<String>(json['id'], 'id', 'String'));

  @override
  _SerializableUser fromJson(Map<String, Object?> json) => deserialize(json);
}

final class _StringCodec implements SerDeCodec<String, String> {
  const _StringCodec();

  @override
  String serialize(String value) => value;

  @override
  String deserialize(String value) => value;
}
