/// Who is calling, with no framework coupling.
final class Caller {
  /// Creates a [Caller].
  const Caller(this.id, this.scopes);

  /// Who they are.
  final String id;

  /// What they may do.
  final List<String> scopes;
}
