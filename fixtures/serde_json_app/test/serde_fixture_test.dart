import 'package:serde_json_app/serde_json_app.dart';
import 'package:test/test.dart';

void main() {
  test('generated serde output analyzes and round trips fixture data', () {
    final fixture = SerdeFixture(
      id: 'account-1',
      displayName: 'Aye',
      tags: const ['vip'],
      accessLevel: AccessLevel.superAdmin,
      createdAt: DateTime.utc(2026, 6, 27, 1, 2, 3),
      homepage: Uri.parse('https://dust.dev'),
      largeNumber: BigInt.parse('9007199254740993'),
      endpoints: {Uri.parse('https://api.dust.dev')},
      metrics: const {
        'daily': [1, 2, 3],
      },
      profile: const NestedProfile(id: 'profile-1', nickname: 'Ace'),
      receipts: const [ExternalReceipt(id: 'receipt-1', cents: 4200)],
      serverOnly: 'server',
      clientOnly: 'client',
      hidden: 'hidden',
      token: const Token('token-1'),
    );

    expect(fixture.toJson(), {
      'id': 'account-1',
      'display_name': 'Aye',
      'tags': ['vip'],
      'access_level': 'super-admin',
      'created_at': '2026-06-27T01:02:03.000Z',
      'homepage': 'https://dust.dev',
      'large_number': '9007199254740993',
      'endpoints': ['https://api.dust.dev'],
      'metrics': {
        'daily': [1, 2, 3],
      },
      'profile': {'id': 'profile-1', 'nickname': 'Ace'},
      'receipts': [
        {'id': 'receipt-1', 'cents': 4200},
      ],
      'client_only': 'client',
      'token': 'token-1',
    });

    final decoded = SerdeFixture.fromJson({
      'id': 'account-2',
      'displayName': 'May',
      'access_level': 'guest-user',
      'created_at': '2026-06-27T01:02:03.000Z',
      'homepage': 'https://dust.dev',
      'large_number': '9007199254740993',
      'endpoints': ['https://api.dust.dev'],
      'metrics': {
        'daily': [1, 2, 3],
      },
      'profile': {'id': 'profile-2', 'nickname': null},
      'receipts': [
        {'id': 'receipt-2', 'cents': 1800},
      ],
      'server_only': 'server-json',
      'token': 'token-2',
    });

    expect(decoded.displayName, 'May');
    expect(decoded.tags, ['guest']);
    expect(decoded.accessLevel, AccessLevel.guestUser);
    expect(decoded.profile.id, 'profile-2');
    expect(decoded.receipts.single.cents, 1800);
    expect(decoded.serverOnly, 'server-json');
    expect(decoded.clientOnly, 'client-default');
    expect(decoded.hidden, 'hidden-default');
    expect(decoded.token.value, 'token-2');
  });

  test('strict mode rejects skipped deserialize keys', () {
    expect(
      () => SerdeFixture.fromJson({
        'id': 'account-3',
        'access_level': 'read-only',
        'created_at': '2026-06-27T01:02:03.000Z',
        'homepage': 'https://dust.dev',
        'large_number': '9007199254740993',
        'endpoints': const <String>[],
        'metrics': const <String, Object?>{},
        'profile': {'id': 'profile-3', 'nickname': null},
        'receipts': const <Object?>[],
        'client_only': 'not accepted',
        'token': 'token-3',
      }),
      throwsArgumentError,
    );
  });
}
