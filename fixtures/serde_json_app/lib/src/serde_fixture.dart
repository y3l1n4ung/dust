import 'package:dust_dart/serde.dart';

import 'external_receipt.dart';

part 'serde_fixture.g.dart';

/// Access level values for the serde fixture app.
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AccessLevel {
  /// Super admin access level.
  superAdmin,

  /// Guest user access level.
  guestUser,

  /// Read only access level.
  readOnly,
}

/// Token model for the serde fixture app.
final class Token {
  /// Creates a [Token].
  const Token(this.value);

  /// Token value.
  final String value;
}

/// Token codec model for the serde fixture app.
final class TokenCodec implements SerDeCodec<Token, String> {
  /// Creates a [TokenCodec].
  const TokenCodec();

  @override
  String serialize(Token value) => value.value;

  @override
  Token deserialize(String value) => Token(value);
}

/// Token codec.
const tokenCodec = TokenCodec();

/// Nested profile model for the serde fixture app.
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
final class NestedProfile with _$NestedProfile {
  /// Creates a [NestedProfile].
  const NestedProfile({required this.id, this.nickname});

  /// Creates a [NestedProfile] from JSON.
  factory NestedProfile.fromJson(Map<String, Object?> json) =>
      _$NestedProfileFromJson(json);

  /// Unique identifier.
  final String id;

  /// Nickname.
  final String? nickname;
}

/// Serde fixture model for the serde fixture app.
@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
final class SerdeFixture with _$SerdeFixture {
  /// Creates a [SerdeFixture].
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

  /// Creates a [SerdeFixture] from JSON.
  factory SerdeFixture.fromJson(Map<String, Object?> json) =>
      _$SerdeFixtureFromJson(json);

  /// Unique identifier.
  final String id;

  /// Display name.
  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String? displayName;

  /// Tags.
  @SerDe(defaultValue: ['guest'])
  final List<String> tags;

  /// Access level.
  final AccessLevel accessLevel;

  /// Created at.
  final DateTime createdAt;

  /// Homepage.
  final Uri homepage;

  /// Large number.
  final BigInt largeNumber;

  /// Endpoints.
  final Set<Uri> endpoints;

  /// Metrics.
  final Map<String, List<int>> metrics;

  /// Profile.
  final NestedProfile profile;

  /// Receipts.
  final List<ExternalReceipt> receipts;

  /// Server only.
  @SerDe(skipSerializing: true, defaultValue: 'server-default')
  final String serverOnly;

  /// Client only.
  @SerDe(skipDeserializing: true, defaultValue: 'client-default')
  final String clientOnly;

  /// Hidden.
  @SerDe(skip: true, defaultValue: 'hidden-default')
  final String hidden;

  /// Token.
  @SerDe(using: tokenCodec)
  final Token token;
}
