import 'package:derive_annotation/derive_annotation.dart';

import 'audit.dart';

part 'entity.g.dart';

@Derive([Debug(), PartialEq(), Hash()])
abstract class Entity extends CatalogNode with AuditStamp, _$EntityDust {
  const Entity(this.id);

  final String id;
}

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
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
