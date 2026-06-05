/// Result of a SQL execute statement.
final class ExecResult {
  /// Creates one execute result.
  const ExecResult({required this.rowsAffected, this.lastInsertId});

  /// Number of rows affected by the statement.
  final int rowsAffected;

  /// Last inserted row id when the driver can report it.
  final int? lastInsertId;
}
