import 'package:derive_serde_annotation/derive_serde_annotation.dart';

@Derive([Serialize(), Deserialize()])
enum UserRole { admin, editor, guest }

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.kebabCase)
enum AuditChannel { webPortal, batchImport, supportTool }

enum Vehicle {
  car(tires: 4),
  bicycle(tires: 2),
  unicycle(tires: 1);

  const Vehicle({required this.tires});

  final int tires;

  bool get isMotorized => this == Vehicle.car;
}

final class VehicleIndexCodec implements SerDeCodec<Vehicle, int> {
  const VehicleIndexCodec();

  @override
  int serialize(Vehicle value) => switch (value) {
    Vehicle.car => 0,
    Vehicle.bicycle => 1,
    Vehicle.unicycle => 2,
  };

  @override
  Vehicle deserialize(int value) => switch (value) {
    0 => Vehicle.car,
    1 => Vehicle.bicycle,
    2 => Vehicle.unicycle,
    _ => throw ArgumentError.value(value, 'value', 'unknown Vehicle index'),
  };
}

const vehicleIndexCodec = VehicleIndexCodec();

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class AuditLog {
  @SerDe(rename: 'created_at')
  final String createdAt;
  final UserRole role;
  final List<AuditChannel> channels;
  @SerDe(using: vehicleIndexCodec)
  final Vehicle vehicle;

  const AuditLog(this.createdAt, this.role, this.channels, this.vehicle);
}

void main() {
  const log = AuditLog(
    '2026-04-30T00:00:00Z',
    UserRole.admin,
    [AuditChannel.webPortal, AuditChannel.supportTool],
    Vehicle.car,
  );
  print(log.createdAt);
  print(log.role);
  print(log.channels);
  print('${log.vehicle.name} ${log.vehicle.tires} ${log.vehicle.isMotorized}');
}
