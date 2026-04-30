import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_enhanced_enum_bundle.g.dart';

@Derive([Serialize(), Deserialize()])
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

final class VehicleIndexListCodec implements SerDeCodec<List<Vehicle>, List<int>> {
  const VehicleIndexListCodec();

  @override
  List<int> serialize(List<Vehicle> value) =>
      value.map(vehicleIndexCodec.serialize).toList(growable: false);

  @override
  List<Vehicle> deserialize(List<int> value) =>
      value.map(vehicleIndexCodec.deserialize).toList(growable: false);
}

const vehicleIndexCodec = VehicleIndexCodec();
const vehicleIndexListCodec = VehicleIndexListCodec();

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonEnhancedEnumBundle with _$JsonEnhancedEnumBundleDust {
  const JsonEnhancedEnumBundle({
    required this.primaryVehicle,
    required this.fallbackVehicle,
    required this.fleet,
  });

  factory JsonEnhancedEnumBundle.fromJson(Map<String, Object?> json) =>
      _$JsonEnhancedEnumBundleFromJson(json);

  @SerDe(using: vehicleIndexCodec)
  final Vehicle primaryVehicle;

  @SerDe(using: vehicleIndexCodec)
  final Vehicle? fallbackVehicle;

  @SerDe(using: vehicleIndexListCodec)
  final List<Vehicle> fleet;
}
