import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('generated serde features handle rename aliases and defaults', () {
    const profile = JsonProfile(
      id: 'user-1',
      displayName: 'May',
      tags: ['vip'],
    );

    expect(profile.toJson(), {
      'id': 'user-1',
      'display_name': 'May',
      'tags': ['vip'],
    });

    final fromAlias = JsonProfile.fromJson({
      'id': 'user-1',
      'displayName': 'May',
    });
    expect(
      fromAlias,
      equals(
        const JsonProfile(id: 'user-1', displayName: 'May', tags: ['guest']),
      ),
    );

    expect(
      () => JsonProfile.fromJson({'id': 'user-1', 'unknown': true}),
      throwsArgumentError,
    );
  });

  test(
    'generated serde diagnostics include the failing key for wrong types',
    () {
      expect(
        () => JsonProfile.fromJson({'id': 42, 'display_name': 'May'}),
        throwsA(
          isA<ArgumentError>()
              .having((error) => error.name, 'name', 'id')
              .having(
                (error) => '${error.message}',
                'message',
                'expected String at id',
              ),
        ),
      );
    },
  );

  test('generated serde features support nested models and collections', () {
    const profile = JsonProfile(
      id: 'user-9',
      displayName: 'Aye',
      tags: ['featured'],
    );
    final account = JsonAccount(
      profile: profile,
      metrics: const {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      archived: false,
    );

    expect(account.toJson(), {
      'profile': {
        'id': 'user-9',
        'display_name': 'Aye',
        'tags': ['featured'],
      },
      'metrics': {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      'archived': false,
    });

    final roundTrip = JsonAccount.fromJson({
      'profile': {
        'id': 'user-9',
        'display_name': 'Aye',
        'tags': ['featured'],
      },
      'metrics': {
        'yangon': [1, 2],
        'mandalay': [3],
      },
      'archived': false,
    });

    expect(roundTrip, equals(account));
  });

  test('imported workspace serde models round-trip across files', () {
    const profile = JsonWorkspaceProfile(
      id: 'workspace-profile',
      kind: JsonWorkspaceKind.retail,
    );
    const account = JsonWorkspaceAccount(
      profile: profile,
      active: true,
    );

    final json = account.toJson();

    expect(json['profile'], profile.toJson());
    final decoded = JsonWorkspaceAccount.fromJson(json);
    expect(decoded.profile.id, profile.id);
    expect(decoded.profile.kind, profile.kind);
    expect(decoded.active, account.active);
  });

  test('generated serde diagnostics include nested collection paths', () {
    expect(
      () => JsonAccount.fromJson({
        'profile': {
          'id': 'user-9',
          'display_name': 'Aye',
          'tags': ['featured'],
        },
        'metrics': {
          'daily': [1, 'bad'],
        },
        'archived': false,
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'metrics.daily[1]')
            .having(
              (error) => '${error.message}',
              'message',
              'expected int at metrics.daily[1]',
            ),
      ),
    );
  });

  test('generated serde features support DateTime Uri and BigInt', () {
    final bundle = JsonScalarBundle(
      createdAt: DateTime.utc(2026, 4, 29, 12, 30, 45),
      updatedAt: null,
      website: Uri.parse('https://dust.dev/products/runner'),
      largeNumber: BigInt.parse('900719925474099312345'),
      endpoints: {
        Uri.parse('https://a.dust.dev'),
        Uri.parse('https://b.dust.dev'),
      },
      checkpoints: {
        'draft': DateTime.utc(2026, 4, 1, 8, 0, 0),
        'live': DateTime.utc(2026, 4, 29, 12, 30, 45),
      },
    );

    expect(bundle.toJson(), {
      'createdAt': '2026-04-29T12:30:45.000Z',
      'updatedAt': null,
      'website': 'https://dust.dev/products/runner',
      'largeNumber': '900719925474099312345',
      'endpoints': ['https://a.dust.dev', 'https://b.dust.dev'],
      'checkpoints': {
        'draft': '2026-04-01T08:00:00.000Z',
        'live': '2026-04-29T12:30:45.000Z',
      },
    });

    final roundTrip = JsonScalarBundle.fromJson({
      'createdAt': '2026-04-29T12:30:45.000Z',
      'updatedAt': '2026-05-01T09:15:00.000Z',
      'website': 'https://dust.dev/products/runner',
      'largeNumber': '900719925474099312345',
      'endpoints': ['https://a.dust.dev', 'https://b.dust.dev'],
      'checkpoints': {
        'draft': '2026-04-01T08:00:00.000Z',
        'live': '2026-04-29T12:30:45.000Z',
      },
    });

    expect(roundTrip.createdAt, DateTime.utc(2026, 4, 29, 12, 30, 45));
    expect(roundTrip.updatedAt, DateTime.utc(2026, 5, 1, 9, 15, 0));
    expect(roundTrip.website, Uri.parse('https://dust.dev/products/runner'));
    expect(roundTrip.largeNumber, BigInt.parse('900719925474099312345'));
    expect(roundTrip.endpoints, {
      Uri.parse('https://a.dust.dev'),
      Uri.parse('https://b.dust.dev'),
    });
    expect(roundTrip.checkpoints, {
      'draft': DateTime.utc(2026, 4, 1, 8, 0, 0),
      'live': DateTime.utc(2026, 4, 29, 12, 30, 45),
    });
  });

  test(
    'generated serde diagnostics include the failing key for bad formats',
    () {
      expect(
        () => JsonScalarBundle.fromJson({
          'createdAt': 'not-a-date',
          'updatedAt': null,
          'website': 'https://dust.dev/products/runner',
          'largeNumber': '900719925474099312345',
          'endpoints': const <String>[],
          'checkpoints': const <String, String>{},
        }),
        throwsA(
          isA<ArgumentError>()
              .having((error) => error.name, 'name', 'createdAt')
              .having(
                (error) => '${error.message}',
                'message',
                contains('ISO-8601 DateTime string'),
              ),
        ),
      );
    },
  );

  test('generated serde features support custom SerDeCodec fields', () {
    final bundle = JsonCodecBundle(
      createdAt: DateTime.fromMillisecondsSinceEpoch(
        1704067200000,
        isUtc: true,
      ),
      updatedAt: DateTime.fromMillisecondsSinceEpoch(
        1706745600000,
        isUtc: true,
      ),
    );

    expect(bundle.toJson(), {
      'createdAt': 1704067200000,
      'updatedAt': 1706745600000,
    });

    final roundTrip = JsonCodecBundle.fromJson({
      'createdAt': 1704067200000,
      'updatedAt': 1706745600000,
    });

    expect(roundTrip, equals(bundle));
  });
}
