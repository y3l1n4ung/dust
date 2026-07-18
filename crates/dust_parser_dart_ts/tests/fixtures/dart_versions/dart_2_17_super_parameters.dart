// @dart=2.17

class ParentRecord {
  const ParentRecord(this.id, {this.enabled = true});

  final String id;
  final bool enabled;
}

class ChildRecord extends ParentRecord {
  const ChildRecord(super.id, {required this.title, super.enabled = false});

  final String title;
}
