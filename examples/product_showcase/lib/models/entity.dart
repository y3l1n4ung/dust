import 'package:derive_annotation/derive_annotation.dart';

import 'audit.dart';

part 'entity.g.dart';

@Derive([ToString(), Eq()])
abstract class Entity extends CatalogNode with AuditStamp, _$EntityDust {
  const Entity(this.id);

  final String id;
}

@Derive([ToString(), Eq(), CopyWith()])
class DetailedEntity extends Entity with _$DetailedEntityDust {
  const DetailedEntity(
    super.id, {
    required this.label,
    required this.tags,
  });

  final String label;
  final List<String> tags;
}

class EntityView extends Entity {
  const EntityView(super.id);
}
