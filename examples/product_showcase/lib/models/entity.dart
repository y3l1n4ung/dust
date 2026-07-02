import 'package:dust_dart/derive.dart';

import 'audit.dart';

part 'entity.g.dart';

/// Entity model for the product showcase example.
@Derive([ToString(), Eq()])
abstract class Entity extends CatalogNode with AuditStamp, _$Entity {
  /// Creates an [Entity].
  const Entity(this.id);

  /// Unique identifier.
  final String id;
}

/// Detailed entity model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class DetailedEntity extends Entity with _$DetailedEntity {
  /// Creates a [DetailedEntity].
  const DetailedEntity(super.id, {required this.label, required this.tags});

  /// Label.
  final String label;

  /// Tags.
  final List<String> tags;
}

/// Entity view model for the product showcase example.
class EntityView extends Entity {
  /// Creates an [EntityView].
  const EntityView(super.id);
}
