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
        r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readNullable<Object?>('bio') == null ? '' : row.read<String>('bio'),
      sessionActive: false,
      preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
      status: const UserStatusFromInt().decode(row.read<int>('status')),
    );
  }
}

/// Row deserializer for [UserProfile].
///
/// Pass it as the `using:` argument of a typed row query, so a row type with
/// no mapping is an analyzer error rather than a failure on the first request.
final class $UserProfileFromRow implements RowDeserializer<UserProfile> {
  const $UserProfileFromRow();

  @override
  UserProfile deserialize(Row row) => UserProfileFromRow.fromRow(row);
}"#
    );
}
