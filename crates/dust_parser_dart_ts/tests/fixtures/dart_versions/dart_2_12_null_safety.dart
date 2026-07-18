// @dart=2.12

class LegacyNullSafetyUser {
  const LegacyNullSafetyUser({required this.id, this.nickname});

  final String id;
  final String? nickname;
}

mixin LegacyAudit {
  String get auditLabel => 'audit';
}

typedef LegacyMapper = Map<String, List<int?>>;
