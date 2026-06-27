import 'package:dust_dart/serde.dart';

import 'external_receipt.dart';

part 'serde_fixture.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel { superAdmin, guestUser, readOnly }

final class Token {
  const Token(this.value);

  final String value;
}

final class TokenCodec implements SerDeCodec<Token, String> {
  const TokenCodec();

  @override
  String serialize(Token value) => value.value;

  @override
  Token deserialize(String value) => Token(value);
}

const tokenCodec = TokenCodec();

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
final class NestedProfile with _$NestedProfile {
  const NestedProfile({required this.id, this.nickname});

  factory NestedProfile.fromJson(Map<String, Object?> json) =>
      _$NestedProfileFromJson(json);

  final String id;
  final String? nickname;
}

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
final class SerdeFixture with _$SerdeFixture {
  const SerdeFixture({
    required this.id,
    this.displayName,
    this.tags = const ['guest'],
    required this.accessLevel,
    required this.createdAt,
    required this.homepage,
    required this.largeNumber,
    required this.endpoints,
    required this.metrics,
    required this.profile,
    required this.receipts,
    this.serverOnly = 'server-default',
    this.clientOnly = 'client-default',
    this.hidden = 'hidden-default',
    required this.token,
  });

  factory SerdeFixture.fromJson(Map<String, Object?> json) =>
      _$SerdeFixtureFromJson(json);

  final String id;

  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String? displayName;

  @SerDe(defaultValue: ['guest'])
  final List<String> tags;

  final AccessLevel accessLevel;
  final DateTime createdAt;
  final Uri homepage;
  final BigInt largeNumber;
  final Set<Uri> endpoints;
  final Map<String, List<int>> metrics;
  final NestedProfile profile;
  final List<ExternalReceipt> receipts;

  @SerDe(skipSerializing: true, defaultValue: 'server-default')
  final String serverOnly;

  @SerDe(skipDeserializing: true, defaultValue: 'client-default')
  final String clientOnly;

  @SerDe(skip: true, defaultValue: 'hidden-default')
  final String hidden;

  @SerDe(using: tokenCodec)
  final Token token;
}
