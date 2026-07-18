// @dart=3.12

class PrivateNamedParameter {
  const PrivateNamedParameter({
    required this.value,
    String? _traceId,
  });

  final String value;
}
