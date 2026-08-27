use dust_db_plugin::register_row_plugin;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::{library, row_class};

#[test]
fn emits_sqlx_style_from_row_mapper() {
    let plugin = register_row_plugin();
    let contribution = plugin
        .generate(
            &library(vec![row_class()]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(
        contribution.support_types[0],
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
  Future<UserProfile> fetchOne(DatabaseExecutor db) =>
      fetchOneWith(db, _$UserProfileFromRow);

  /// Fetches zero or one row.
  Future<UserProfile?> fetchOptional(DatabaseExecutor db) =>
      fetchOptionalWith(db, _$UserProfileFromRow);

  /// Fetches every row.
  Future<List<UserProfile>> fetchAll(DatabaseExecutor db) =>
      fetchAllWith(db, _$UserProfileFromRow);
}"#
    );
}
