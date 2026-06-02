use dust_db_plugin::register_row_plugin;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::support::{library, row_class};

#[test]
fn emits_sqlx_style_from_row_mapper() {
    let plugin = register_row_plugin();
    let contribution = plugin.emit(&library(vec![row_class()]), &SymbolPlan::default());

    assert_eq!(
        contribution.support_types[0],
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

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);"#
    );
}
