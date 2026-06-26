import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test('generated serde features support enums and enum collections', () {
    const bundle = JsonEnumBundle(
      primaryLevel: AccessLevel.superAdmin,
      fallbackState: ReviewState.approved,
      levels: [AccessLevel.superAdmin, AccessLevel.readOnly],
      stateByRegion: {
        'yangon': ReviewState.pending,
        'mandalay': ReviewState.archived,
      },
      states: {ReviewState.pending, ReviewState.approved},
    );

    expect(bundle.toJson(), {
      'primary_level': 'super-admin',
      'fallbackState': 'approved',
      'levels': ['super-admin', 'read-only'],
      'stateByRegion': {'yangon': 'pending', 'mandalay': 'archived'},
      'states': ['pending', 'approved'],
    });

    final roundTrip = JsonEnumBundle.fromJson({
      'primaryLevel': 'guest-user',
      'fallbackState': null,
      'levels': ['guest-user', 'read-only'],
      'stateByRegion': {'yangon': 'approved', 'mandalay': 'pending'},
      'states': ['approved', 'archived'],
    });

    expect(
      roundTrip,
      equals(
        const JsonEnumBundle(
          primaryLevel: AccessLevel.guestUser,
          fallbackState: null,
          levels: [AccessLevel.guestUser, AccessLevel.readOnly],
          stateByRegion: {
            'yangon': ReviewState.approved,
            'mandalay': ReviewState.pending,
          },
          states: {ReviewState.approved, ReviewState.archived},
        ),
      ),
    );
  });

  test('generated serde diagnostics include unknown enum values', () {
    expect(
      () => JsonEnumBundle.fromJson({
        'primary_level': 'power-user',
        'fallbackState': 'approved',
        'levels': ['super-admin'],
        'stateByRegion': {'yangon': 'pending'},
        'states': ['approved'],
      }),
      throwsA(
        isA<ArgumentError>().having(
          (error) => '${error.message}',
          'message',
          contains('unknown value for AccessLevel'),
        ),
      ),
    );
  });

  test('generated serde features support enhanced enums with index codecs', () {
    const bundle = JsonEnhancedEnumBundle(
      primaryVehicle: Vehicle.car,
      fallbackVehicle: Vehicle.bicycle,
      fleet: [Vehicle.car, Vehicle.unicycle],
    );

    expect(bundle.toJson(), {
      'primaryVehicle': 0,
      'fallbackVehicle': 1,
      'fleet': [0, 2],
    });

    final roundTrip = JsonEnhancedEnumBundle.fromJson({
      'primaryVehicle': 2,
      'fallbackVehicle': null,
      'fleet': [1, 0],
    });

    expect(roundTrip.primaryVehicle, Vehicle.unicycle);
    expect(roundTrip.primaryVehicle.tires, 1);
    expect(roundTrip.primaryVehicle.isMotorized, isFalse);
    expect(roundTrip.fallbackVehicle, isNull);
    expect(roundTrip.fleet, [Vehicle.bicycle, Vehicle.car]);
    expect(roundTrip.fleet[0].tires, 2);
    expect(roundTrip.fleet[1].isMotorized, isTrue);
  });

  test('generated serde diagnostics include codec enum index failures', () {
    expect(
      () => JsonEnhancedEnumBundle.fromJson({
        'primaryVehicle': 99,
        'fallbackVehicle': null,
        'fleet': [0],
      }),
      throwsA(
        isA<ArgumentError>().having(
          (error) => '${error.message}',
          'message',
          contains('failed SerDeCodec decode'),
        ),
      ),
    );
  });

  test(
    'generated serde diagnostics include the failing key for codec fields',
    () {
      expect(
        () => JsonCodecBundle.fromJson({
          'createdAt': null,
          'updatedAt': 1706745600000,
        }),
        throwsA(
          isA<ArgumentError>()
              .having((error) => error.name, 'name', 'createdAt')
              .having(
                (error) => '${error.message}',
                'message',
                contains('SerDeCodec'),
              ),
        ),
      );
    },
  );

  test('generated serde features cover all supported SerDe flags', () {
    const options = JsonSerdeOptions(
      id: 'user-2',
      displayName: 'May',
      e: MyEnum.A,
      tags: ['vip'],
      serverOnly: 'server-secret',
      clientOnly: 'client-visible',
      hidden: 'hidden-secret',
    );

    expect(options.toJson(), {
      'id': 'user-2',
      'display_name': 'May',
      'e': 'A',
      'tags': ['vip'],
      'client_only': 'client-visible',
    });

    final fromJson = JsonSerdeOptions.fromJson({
      'id': 'user-2',
      'displayName': 'May',
      'e': 'B',
      'server_only': 'from-server',
    });

    expect(
      fromJson,
      equals(
        const JsonSerdeOptions(
          id: 'user-2',
          displayName: 'May',
          e: MyEnum.B,
          tags: ['guest'],
          serverOnly: 'from-server',
          clientOnly: 'client-default',
          hidden: 'hidden-default',
        ),
      ),
    );

    expect(
      () => JsonSerdeOptions.fromJson({
        'id': 'user-2',
        'display_name': 'May',
        'e': 'A',
        'client_only': 'ignored-client',
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'json')
            .having(
              (error) => error.invalidValue,
              'invalidValue',
              'client_only',
            )
            .having(
              (error) => '${error.message}',
              'message',
              'unknown key for JsonSerdeOptions',
            ),
      ),
    );

    expect(
      () => JsonSerdeOptions.fromJson({
        'id': 'user-2',
        'display_name': 'May',
        'e': 'A',
        'hidden': 'ignored-hidden',
      }),
      throwsA(
        isA<ArgumentError>()
            .having((error) => error.name, 'name', 'json')
            .having((error) => error.invalidValue, 'invalidValue', 'hidden')
            .having(
              (error) => '${error.message}',
              'message',
              'unknown key for JsonSerdeOptions',
            ),
      ),
    );

    expect(
      () => JsonSerdeOptions.fromJson({
        'id': 'user-2',
        'display_name': 'May',
        'e': 'A',
        'unexpected': true,
      }),
      throwsArgumentError,
    );
  });
}
