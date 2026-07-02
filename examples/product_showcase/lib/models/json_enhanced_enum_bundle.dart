import 'package:dust_dart/serde.dart';

part 'json_enhanced_enum_bundle.g.dart';

/// Vehicle values for the product showcase example.
@Derive([Serialize(), Deserialize()])
enum Vehicle {
  /// Car vehicle.
  car(tires: 4),

  /// Bicycle vehicle.
  bicycle(tires: 2),

  /// Unicycle vehicle.
  unicycle(tires: 1);

  /// Creates a [Vehicle].
  const Vehicle({required this.tires});

  /// Tires.
  final int tires;

  /// Whether the value is motorized.
  bool get isMotorized => this == Vehicle.car;
}

/// Vehicle index codec model for the product showcase example.
final class VehicleIndexCodec implements SerDeCodec<Vehicle, int> {
  /// Creates a [VehicleIndexCodec].
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

/// Vehicle index list codec for the product showcase example.
final class VehicleIndexListCodec
    implements SerDeCodec<List<Vehicle>, List<int>> {
  /// Creates a [VehicleIndexListCodec].
  const VehicleIndexListCodec();

  @override
  List<int> serialize(List<Vehicle> value) =>
      value.map(vehicleIndexCodec.serialize).toList(growable: false);

  @override
  List<Vehicle> deserialize(List<int> value) =>
      value.map(vehicleIndexCodec.deserialize).toList(growable: false);
}

/// Vehicle index codec.
const vehicleIndexCodec = VehicleIndexCodec();

/// Vehicle index list codec.
const vehicleIndexListCodec = VehicleIndexListCodec();

/// JSON enhanced enum bundle model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonEnhancedEnumBundle with _$JsonEnhancedEnumBundle {
  /// Creates a [JsonEnhancedEnumBundle].
  const JsonEnhancedEnumBundle({
    required this.primaryVehicle,
    required this.fallbackVehicle,
    required this.fleet,
  });

  /// Creates a [JsonEnhancedEnumBundle] from JSON.
  factory JsonEnhancedEnumBundle.fromJson(Map<String, Object?> json) =>
      _$JsonEnhancedEnumBundleFromJson(json);

  /// Primary vehicle.
  @SerDe(using: vehicleIndexCodec)
  final Vehicle primaryVehicle;

  /// Fallback vehicle.
  @SerDe(using: vehicleIndexCodec)
  final Vehicle? fallbackVehicle;

  /// Fleet.
  @SerDe(using: vehicleIndexListCodec)
  final List<Vehicle> fleet;
}
