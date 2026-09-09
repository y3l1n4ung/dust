//! The Dart a DAO is expected to emit, one fixture per shape.

pub(super) fn expected_default_dao_output() -> &'static str {
    r#"UserProfile _$UserProfileFromRow(Row row) {
  return UserProfile(
    id: row.read<int>('id'),
    name: row.read<String>('display_name'),
    bio: row.readNullable<Object?>('bio') == null ? '' : row.read<String>('bio'),
    sessionActive: false,
    preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
    status: const UserStatusFromInt().decode(row.read<int>('status')),
  );
}

/// Row deserializer for [UserProfile].
final class $UserProfileRowDeserializer implements RowDeserializer<UserProfile> {
  const $UserProfileRowDeserializer();

  @override
  UserProfile deserialize(Row row) => _$UserProfileFromRow(row);
}

/// Typed row query terminals for [UserProfile].
///
/// Resolved from the static type of the receiver, so a row type with no
/// `FromRow` has no terminals and the call does not compile.
extension $UserProfileQuery on QueryAs<UserProfile> {
  /// Fetches exactly one row.
  Future<Result<UserProfile, SqlxError>> fetchOne(Executor db) =>
      fetchOneWith(db, _$UserProfileFromRow);

  /// Fetches zero or one row.
  Future<Result<UserProfile?, SqlxError>> fetchOptional(Executor db) =>
      fetchOptionalWith(db, _$UserProfileFromRow);

  /// Fetches every row.
  Future<Result<List<UserProfile>, SqlxError>> fetchAll(Executor db) =>
      fetchAllWith(db, _$UserProfileFromRow);
}

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final Executor _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name, bio FROM users WHERE id = $1''',
      [id],
      const $UserProfileRowDeserializer().deserialize,
    );
  }

  @override
  Future<Result<int, SqlxError>> count() {
    return _db.fetchScalar<int>(
      r'''SELECT COUNT(*) FROM users''',
      [],
    );
  }

  @override
  Future<Result<ExecResult, SqlxError>> rename(String name, int id) {
    return _db.execute(
      r'''UPDATE users SET display_name = $1 WHERE id = $2''',
      [name, id],
    );
  }
}"#
}

pub(super) fn expected_cardinality_output() -> &'static str {
    r#"UserProfile _$UserProfileFromRow(Row row) {
  return UserProfile(
    id: row.read<int>('id'),
    name: row.read<String>('display_name'),
    bio: row.readNullable<Object?>('bio') == null ? '' : row.read<String>('bio'),
    sessionActive: false,
    preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
    status: const UserStatusFromInt().decode(row.read<int>('status')),
  );
}

/// Row deserializer for [UserProfile].
final class $UserProfileRowDeserializer implements RowDeserializer<UserProfile> {
  const $UserProfileRowDeserializer();

  @override
  UserProfile deserialize(Row row) => _$UserProfileFromRow(row);
}

/// Typed row query terminals for [UserProfile].
///
/// Resolved from the static type of the receiver, so a row type with no
/// `FromRow` has no terminals and the call does not compile.
extension $UserProfileQuery on QueryAs<UserProfile> {
  /// Fetches exactly one row.
  Future<Result<UserProfile, SqlxError>> fetchOne(Executor db) =>
      fetchOneWith(db, _$UserProfileFromRow);

  /// Fetches zero or one row.
  Future<Result<UserProfile?, SqlxError>> fetchOptional(Executor db) =>
      fetchOptionalWith(db, _$UserProfileFromRow);

  /// Fetches every row.
  Future<Result<List<UserProfile>, SqlxError>> fetchAll(Executor db) =>
      fetchAllWith(db, _$UserProfileFromRow);
}

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final Executor _db;

  @override
  Future<Result<UserProfile, SqlxError>> findRequired(int id) {
    return _db.fetchOne<UserProfile>(
      r'''SELECT id, display_name FROM users WHERE id = $1''',
      [id],
      const $UserProfileRowDeserializer().deserialize,
    );
  }

  @override
  Future<Result<List<UserProfile>, SqlxError>> list() {
    return _db.fetchAll<UserProfile>(
      r'''SELECT id, display_name FROM users''',
      [],
      const $UserProfileRowDeserializer().deserialize,
    );
  }

  @override
  Future<Result<List<Row>, SqlxError>> rawRows() {
    return Err<List<Row>, SqlxError>(
      SqlxError.decode('A DAO cannot return untyped rows. Use a row type with @Derive([FromRow()]), or the unsafe escape hatch on the database facade.'),
    );
  }

  @override
  Future<Result<Unit, SqlxError>> deleteAll() {
    return _db.execute(
      r'''DELETE FROM users''',
      [],
    ).then(
      (result) => result.andThen<Unit>((_) => const Ok<Unit, SqlxError>(unit)),
    );
  }
}"#
}

pub(super) fn expected_imported_dao_output() -> &'static str {
    r#"final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final Executor _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name, bio FROM users WHERE id = $1''',
      [id],
      const $UserProfileRowDeserializer().deserialize,
    );
  }

  @override
  Future<Result<int, SqlxError>> count() {
    return _db.fetchScalar<int>(
      r'''SELECT COUNT(*) FROM users''',
      [],
    );
  }

  @override
  Future<Result<ExecResult, SqlxError>> rename(String name, int id) {
    return _db.execute(
      r'''UPDATE users SET display_name = $1 WHERE id = $2''',
      [name, id],
    );
  }
}"#
}

pub(super) fn expected_reordered_sqlite_output() -> &'static str {
    r#"UserProfile _$UserProfileFromRow(Row row) {
  return UserProfile(
    id: row.read<int>('id'),
    name: row.read<String>('display_name'),
  );
}

/// Row deserializer for [UserProfile].
final class $UserProfileRowDeserializer implements RowDeserializer<UserProfile> {
  const $UserProfileRowDeserializer();

  @override
  UserProfile deserialize(Row row) => _$UserProfileFromRow(row);
}

/// Typed row query terminals for [UserProfile].
///
/// Resolved from the static type of the receiver, so a row type with no
/// `FromRow` has no terminals and the call does not compile.
extension $UserProfileQuery on QueryAs<UserProfile> {
  /// Fetches exactly one row.
  Future<Result<UserProfile, SqlxError>> fetchOne(Executor db) =>
      fetchOneWith(db, _$UserProfileFromRow);

  /// Fetches zero or one row.
  Future<Result<UserProfile?, SqlxError>> fetchOptional(Executor db) =>
      fetchOptionalWith(db, _$UserProfileFromRow);

  /// Fetches every row.
  Future<Result<List<UserProfile>, SqlxError>> fetchAll(Executor db) =>
      fetchAllWith(db, _$UserProfileFromRow);
}

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final Executor _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findForOrg(int id, int orgId) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name FROM users WHERE org_id = $2 OR id = $1 OR backup_id = $1''',
      [id, orgId],
      const $UserProfileRowDeserializer().deserialize,
    );
  }
}"#
}
