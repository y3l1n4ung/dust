import 'template_model.dart';
import 'template_support.dart';

String renderSerdeScalar(int index) {
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_serde_annotation/derive_serde_annotation.dart';",
    ],
    declarations: [
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class $className with ${mixinName(className)} {
  const $className({
    required this.id,
    required this.displayName,
    required this.createdAt,
    required this.website,
    required this.balance,
    this.active = true,
  });

${serdeFactory(className)}

  final String id;
  final String displayName;
  final DateTime createdAt;
  final Uri website;
  final BigInt balance;
  final bool active;
}''',
    ],
  );
}

String renderSerdeOptions(int index) {
  final number = index + 1;
  final enumName = 'SerdeKind$number';
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_serde_annotation/derive_serde_annotation.dart';",
    ],
    declarations: [
      '''
@Derive([Serialize(), Deserialize()])
enum $enumName { freeUser, teamAdmin, readOnly }''',
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)
class $className with ${mixinName(className)} {
  const $className({
    required this.id,
    required this.displayName,
    required this.kind,
    this.tags = const ['guest'],
    this.serverOnly = 'server-default',
    this.clientOnly = 'client-default',
    this.hidden = 'hidden-default',
  });

${serdeFactory(className)}

  final String id;
  final $enumName kind;

  @SerDe(rename: 'display_name', aliases: ['displayName'])
  final String displayName;

  @SerDe(defaultValue: ['guest'])
  final List<String> tags;

  @SerDe(skipSerializing: true, defaultValue: 'server-default')
  final String serverOnly;

  @SerDe(skipDeserializing: true, defaultValue: 'client-default')
  final String clientOnly;

  @SerDe(skip: true, defaultValue: 'hidden-default')
  final String hidden;
}''',
    ],
  );
}

String renderSerdeNested(int index) {
  final number = index + 1;
  final itemName = 'NestedItem$number';
  final stateName = 'NestedState$number';
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_serde_annotation/derive_serde_annotation.dart';",
    ],
    declarations: [
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class $itemName with ${mixinName(itemName)} {
  const $itemName({required this.code, required this.quantity});

${serdeFactory(itemName)}

  final String code;
  final int quantity;
}''',
      '''
@Derive([Serialize(), Deserialize()])
enum $stateName { pending, active, archived }''',
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class $className with ${mixinName(className)} {
  const $className({
    required this.primary,
    required this.items,
    required this.byId,
    required this.states,
  });

${serdeFactory(className)}

  final $itemName primary;
  final List<$itemName> items;
  final Map<String, $itemName> byId;
  final Set<$stateName> states;
}''',
    ],
  );
}

String renderSerdeCodec(int index) {
  final number = index + 1;
  final enumName = 'Vehicle$number';
  final codecName = 'VehicleIndexCodec$number';
  final listCodecName = 'VehicleIndexListCodec$number';
  final codecValue = 'vehicleIndexCodec$number';
  final listCodecValue = 'vehicleIndexListCodec$number';
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_serde_annotation/derive_serde_annotation.dart';",
      "import '../support/common.dart';",
    ],
    declarations: [
      '''
@Derive([Serialize(), Deserialize()])
enum $enumName {
  car(tires: 4),
  bicycle(tires: 2),
  unicycle(tires: 1);

  const $enumName({required this.tires});

  final int tires;
}''',
      '''
final class $codecName implements SerDeCodec<$enumName, int> {
  const $codecName();

  @override
  int serialize($enumName value) => switch (value) {
    $enumName.car => 0,
    $enumName.bicycle => 1,
    $enumName.unicycle => 2,
  };

  @override
  $enumName deserialize(int value) => switch (value) {
    0 => $enumName.car,
    1 => $enumName.bicycle,
    2 => $enumName.unicycle,
    _ => throw ArgumentError.value(value, 'value', 'unknown $enumName index'),
  };
}''',
      '''
final class $listCodecName implements SerDeCodec<List<$enumName>, List<int>> {
  const $listCodecName();

  @override
  List<int> serialize(List<$enumName> value) =>
      value.map($codecValue.serialize).toList(growable: false);

  @override
  List<$enumName> deserialize(List<int> value) =>
      value.map($codecValue.deserialize).toList(growable: false);
}''',
      'const $codecValue = $codecName();',
      'const $listCodecValue = $listCodecName();',
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class $className with ${mixinName(className)} {
  const $className({
    required this.createdAt,
    required this.primaryVehicle,
    required this.fleet,
    required this.status,
  });

${serdeFactory(className)}

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  @SerDe(using: $codecValue)
  final $enumName primaryVehicle;

  @SerDe(using: $listCodecValue)
  final List<$enumName> fleet;

  @SerDe(using: sharedStatusIndexCodec)
  final SharedStatus status;
}''',
    ],
  );
}

String renderSerdeLinked(int index) {
  final className = primaryClassNameForIndex(index);
  final previousClass = primaryClassNameForIndex(index - 1);
  final previousFile = fileNameForIndex(index - 1);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_serde_annotation/derive_serde_annotation.dart';",
      "import '$previousFile.dart';",
    ],
    declarations: [
      '''
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class $className with ${mixinName(className)} {
  const $className({
    required this.primary,
    required this.items,
    required this.byId,
    required this.note,
  });

${serdeFactory(className)}

  final $previousClass primary;
  final List<$previousClass> items;
  final Map<String, $previousClass> byId;
  final String note;
}''',
    ],
  );
}
