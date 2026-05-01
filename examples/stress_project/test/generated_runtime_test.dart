import 'package:dust_stress_project/generated_models/model_00004.dart';
import 'package:dust_stress_project/generated_models/model_00005.dart';
import 'package:dust_stress_project/generated_models/model_00007.dart';
import 'package:dust_stress_project/generated_models/model_00008.dart';
import 'package:dust_stress_project/support/common.dart';
import 'package:test/test.dart';

void main() {
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
    expect(identical(copied.primary, linked.primary), isFalse);
    expect(identical(copied.items, linked.items), isFalse);
    expect(identical(copied.byId, linked.byId), isFalse);
  });
}
