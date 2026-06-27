import 'package:dust_dart/derive.dart';
import 'package:dust_benchmark_project/generated_models/model_00004.dart';
import 'package:dust_benchmark_project/generated_models/model_00005.dart';
import 'package:dust_benchmark_project/generated_models/model_00007.dart';
import 'package:dust_benchmark_project/generated_models/model_00008.dart';
import 'package:dust_benchmark_project/generated_models/model_00009.dart';
import 'package:dust_benchmark_project/generated_models/validation_showcase.dart';
import 'package:dust_benchmark_project/benchmark_project.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('benchmark enum serde honors variant rename and skip metadata', () {
    const cold = BenchmarkState(
      mode: BenchmarkMode.cold,
      activeFeature: 'serde',
      buildsRun: 3,
    );

    expect(cold.toJson(), {
      'mode': 'cold-start',
      'activeFeature': 'serde',
      'buildsRun': 3,
    });
    expect(
      BenchmarkState.fromJson({
        'mode': 'cold-start',
        'activeFeature': 'serde',
        'buildsRun': 3,
      }).mode,
      BenchmarkMode.cold,
    );

    const skipped = BenchmarkState(mode: BenchmarkMode.invalidated);
    expect(skipped.toJson, throwsArgumentError);
  });

  test('serde scalar models round-trip primitive adapters', () {
    final model = SerdeScalarModel4(
      id: 'user-4',
      displayName: 'User Four',
      createdAt: DateTime.utc(2024, 1, 2, 3, 4, 5),
      website: Uri.parse('https://example.com/users/4'),
      balance: BigInt.parse('42'),
    );

    expect(SerdeScalarModel4.fromJson(model.toJson()), equals(model));
  });

  test(
    'serde options models honor aliases, defaults, and unknown-key checks',
    () {
      final decoded = SerdeOptionsModel5.fromJson({
        'id': 'user-5',
        'kind': 'teamAdmin',
        'displayName': 'Ops Team',
      });

      expect(decoded.displayName, 'Ops Team');
      expect(decoded.tags, ['guest']);
      expect(decoded.serverOnly, 'server-default');
      expect(decoded.clientOnly, 'client-default');
      expect(decoded.hidden, 'hidden-default');
      expect(decoded.toJson().containsKey('server_only'), isFalse);
      expect(decoded.toJson()['client_only'], 'client-default');

      expect(
        () => SerdeOptionsModel5.fromJson({
          'id': 'user-5',
          'kind': 'teamAdmin',
          'display_name': 'Ops Team',
          'unexpected': true,
        }),
        throwsArgumentError,
      );
    },
  );

  test('linked serde models round-trip nested codec-backed values', () {
    final primary = CodecEnvelope7(
      createdAt: DateTime.utc(2024, 2, 3),
      primaryVehicle: Vehicle7.bicycle,
      fleet: const [Vehicle7.bicycle, Vehicle7.car],
      status: SharedStatus.active,
    );
    final linked = LinkedSerdeModel8(
      primary: primary,
      items: [primary.copyWith(status: SharedStatus.paused)],
      byId: {'primary': primary},
      note: 'ready',
    );

    final decoded = LinkedSerdeModel8.fromJson(linked.toJson());
    final copied = linked.copyWith();

    expect(decoded, equals(linked));
    expect(identical(copied.primary, linked.primary), isTrue);
    expect(identical(copied.items, linked.items), isTrue);
    expect(identical(copied.byId, linked.byId), isTrue);
  });

  test('linked serde benchmark model keeps imported workspace facts live', () {
    const profile = BenchmarkWorkspaceProfile(
      id: 'bench-profile',
      kind: BenchmarkWorkspaceKind.primary,
    );
    const account = BenchmarkWorkspaceAccount(
      profile: profile,
      score: 42,
    );
    final json = account.toJson();

    expect(json['profile'], profile.toJson());
    final decoded = BenchmarkWorkspaceAccount.fromJson(json);
    expect(decoded.profile.id, profile.id);
    expect(decoded.profile.kind, profile.kind);
    expect(decoded.score, account.score);
  });

  test(
    'sealed serde metadata sample keeps concrete variants round-trippable',
    () {
      final event = SealedEvent9.manualAccept(id: 'case-9', score: 99);

      expect(event, isA<SealedAccepted9>());
      final accepted = event as SealedAccepted9;
      expect(event.toJson(), {
        'kind': 'manual_accept',
        'payload': {'id': 'case-9', 'score': 99},
      });
      final decodedAccepted = SealedEvent9.fromJson(event.toJson());
      expect(decodedAccepted, isA<SealedAccepted9>());
      expect((decodedAccepted as SealedAccepted9).id, accepted.id);
      expect(decodedAccepted.score, accepted.score);
      final directAccepted = SealedAccepted9.fromJson({
        'id': 'case-9',
        'score': 99,
      });
      expect(directAccepted.id, accepted.id);
      expect(directAccepted.score, accepted.score);

      final rejected = SealedEvent9.autoReject(
        id: 'case-10',
        reason: 'timeout',
      );
      expect(rejected, isA<SealedRejected9>());
      final rejectedVariant = rejected as SealedRejected9;
      expect(rejected.toJson(), {
        'kind': 'auto_reject',
        'payload': {'id': 'case-10', 'reason': 'timeout'},
      });
      expect(SealedEvent9.fromJson(rejected.toJson()), isA<SealedRejected9>());
      final directRejected = SealedRejected9.fromJson({
        'id': 'case-10',
        'reason': 'timeout',
      });
      expect(directRejected.id, rejectedVariant.id);
      expect(directRejected.reason, rejectedVariant.reason);
    },
  );

  test('validation showcase reports nested and field-level custom errors', () {
    final invalid = BenchmarkSignupValidation(
      email: 'blocked@blocked.example',
      password: 'password',
      confirmPassword: 'different',
      age: 17,
      tags: const [],
      address: const BenchmarkAddressValidation(city: 'A', zipCode: 'abc'),
      website: 'not-a-url',
    ).validate();

    switch (invalid) {
      case Valid():
        fail('expected invalid validation result');
      case Invalid(:final errors):
        expect(
          errors.map((error) => '${error.field}:${error.message}').toList(),
          orderedEquals([
            'email:Blocked email domain',
            'password:Password is too weak',
            'confirmPassword:Passwords must match',
            'age:Age is invalid',
            'tags:Tags are invalid',
            'address.city:City is invalid',
            'address.zipCode:ZIP is invalid',
            'website:Website is invalid',
          ]),
        );
    }
  });

  test('validation showcase accepts valid optional URL', () {
    final valid = BenchmarkSignupValidation(
      email: 'valid@example.com',
      password: 'strong-secret',
      confirmPassword: 'strong-secret',
      age: 42,
      tags: const ['flutter'],
      address: const BenchmarkAddressValidation(
        city: 'Yangon',
        zipCode: '11111',
      ),
      website: 'https://example.com',
    ).validate();

    expect(valid, isA<Valid>());
  });
}
