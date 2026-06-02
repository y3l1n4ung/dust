use std::fs;

use dust_db_plugin::register_plugin;
use dust_ir::{MethodIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::*;

#[test]
fn emits_sqlx_style_dao_redirecting_factory_impl() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![row_class(), dao_class()]),
        &SymbolPlan::default(),
    );

    assert_eq!(contribution.support_types[0], expected_default_dao_output());
}

#[test]
fn emits_driver_method_for_each_query_cardinality() {
    let mut dao = dao_class();
    dao.methods = vec![
        dao_method(
            "findRequired",
            result_type(TypeIr::named("UserProfile")),
            vec![method_param("id", TypeIr::int())],
            "(r'SELECT id, display_name FROM users WHERE id = $1')",
        ),
        dao_method(
            "list",
            result_type(TypeIr::generic("List", vec![TypeIr::named("UserProfile")])),
            Vec::new(),
            "(r'SELECT id, display_name FROM users')",
        ),
        dao_method(
            "rawRows",
            result_type(TypeIr::generic("List", vec![TypeIr::named("Row")])),
            Vec::new(),
            "(r'SELECT id, display_name FROM users')",
        ),
        dao_method(
            "deleteAll",
            result_type(TypeIr::named("Unit")),
            Vec::new(),
            "(r'DELETE FROM users')",
        ),
    ];

    let contribution =
        register_plugin().emit(&library(vec![row_class(), dao]), &SymbolPlan::default());

    assert_eq!(contribution.support_types[0], expected_cardinality_output());
}

#[test]
fn emits_dao_mapper_for_imported_from_row_return_type() {
    let root = temp_root("imported_dao_rows");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/user_profile.dart"),
        "@Derive([FromRow()])\nfinal class UserProfile {}\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_imports(
            &root,
            "lib/dao/user_dao.dart",
            vec!["../models/user_profile.dart"],
            vec![dao_class()],
        ),
        &SymbolPlan::default(),
    );

    assert_eq!(
        contribution.support_types[0],
        expected_imported_dao_output()
    );

    let _ = fs::remove_dir_all(root);
}

fn dao_method(
    name: &str,
    return_type: TypeIr,
    params: Vec<dust_ir::MethodParamIr>,
    query: &str,
) -> MethodIr {
    MethodIr {
        name: name.to_owned(),
        is_static: false,
        is_external: false,
        return_type,
        has_body: false,
        body_source: None,
        params,
        span: span(),
        traits: Vec::new(),
        configs: vec![config("dust_db_annotation::Query", query)],
    }
}

fn expected_default_dao_output() -> &'static str {
    r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readOrNull<Object?>('bio') == null ? '' : row.read<String>('bio'),
      sessionActive: false,
      preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
      status: const UserStatusFromInt().decode(row.read<int>('status')),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name, bio FROM users WHERE id = $1''',
      [id],
      UserProfileFromRow.fromRow,
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

fn expected_cardinality_output() -> &'static str {
    r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readOrNull<Object?>('bio') == null ? '' : row.read<String>('bio'),
      sessionActive: false,
      preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
      status: const UserStatusFromInt().decode(row.read<int>('status')),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<UserProfile, SqlxError>> findRequired(int id) {
    return _db.fetchOne<UserProfile>(
      r'''SELECT id, display_name FROM users WHERE id = $1''',
      [id],
      UserProfileFromRow.fromRow,
    );
  }

  @override
  Future<Result<List<UserProfile>, SqlxError>> list() {
    return _db.fetchAll<UserProfile>(
      r'''SELECT id, display_name FROM users''',
      [],
      UserProfileFromRow.fromRow,
    );
  }

  @override
  Future<Result<List<Row>, SqlxError>> rawRows() {
    return _db.raw.fetch(
      r'''SELECT id, display_name FROM users''',
      [],
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

fn expected_imported_dao_output() -> &'static str {
    r#"final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name, bio FROM users WHERE id = $1''',
      [id],
      UserProfileFromRow.fromRow,
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
