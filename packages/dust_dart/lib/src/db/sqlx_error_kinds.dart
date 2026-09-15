/// Broad category for one SQLx-style runtime error.
enum SqlxErrorCategory {
  /// Generic driver failure when a narrower category is not known.
  driver,

  /// Opening, configuring, closing, or using a closed connection failed.
  connection,

  /// Applying startup migrations failed.
  migration,

  /// Running a query or statement failed.
  query,

  /// Decoding a row or scalar value failed.
  decode,

  /// A query returned the wrong number of rows.
  cardinality,

  /// Beginning, committing, rolling back, or running a transaction failed.
  transaction,
}

/// Which integrity constraint a failed statement broke.
///
/// Named as `sqlx` names its `ErrorKind`, and portable across drivers: SQLite
/// reports these as extended result codes and PostgreSQL as SQLSTATE codes,
/// and each driver maps its own. Match on this rather than on message text,
/// which belongs to the driver and changes with it.
enum SqlxErrorKind {
  /// A `UNIQUE` or `PRIMARY KEY` constraint rejected a duplicate value.
  uniqueViolation,

  /// A `FOREIGN KEY` constraint rejected a reference to a missing row.
  foreignKeyViolation,

  /// A `NOT NULL` constraint rejected a null value.
  notNullViolation,

  /// A `CHECK` constraint rejected a value.
  checkViolation,
}
