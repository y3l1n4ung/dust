use dust_ir::SerdeRenameRuleIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{enum_ir, enum_variant, library, renamed_enum};

#[test]
fn generates_serde_for_enums() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![enum_ir(
            "Status",
            vec![enum_variant("pending"), enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    assert_eq!(contribution.top_level_functions.len(), 2);

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("Object? _$StatusToJson(Status instance)"));
    assert!(to_json.contains("Status.pending => 'pending'"));
    assert!(to_json.contains("Status.active => 'active'"));

    assert!(from_json.contains("Status _$StatusFromJson(Object? json)"));
    assert!(from_json.contains("'pending' => Status.pending"));
    assert!(from_json.contains("'active' => Status.active"));
}

#[test]
fn supports_enum_renaming() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![renamed_enum(
            "UserRole",
            vec![enum_variant("superAdmin"), enum_variant("guestUser")],
            SerdeRenameRuleIr::SnakeCase,
        )],
    );
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("UserRole.superAdmin => 'super_admin'"));
    assert!(to_json.contains("UserRole.guestUser => 'guest_user'"));

    assert!(from_json.contains("'super_admin' => UserRole.superAdmin"));
    assert!(from_json.contains("'guest_user' => UserRole.guestUser"));
}
