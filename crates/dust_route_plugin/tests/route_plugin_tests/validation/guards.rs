use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

use crate::support::{library_with_classes, route_page_class};

use super::support::{add_import, diagnostic_messages};

#[test]
fn accepts_visible_route_guard_from_show_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(&mut library, "package:app/guards.dart", &["AuthGuard"], &[]);

    let diagnostics = plugin.validate(&library);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_route_guard_missing_from_show_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(
        &mut library,
        "package:app/guards.dart",
        &["BillingGuard"],
        &[],
    );

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route guard `AuthGuard` on `AdminPage` must be declared in the same library or imported"
        ]
    );
}

#[test]
fn rejects_route_guard_hidden_by_import() {
    let plugin = register_plugin();
    let class = route_page_class(
        "AdminPage",
        "('/admin', name: 'admin', guards: [AuthGuard])",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![class]);
    add_import(&mut library, "package:app/guards.dart", &[], &["AuthGuard"]);

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route guard `AuthGuard` on `AdminPage` must be declared in the same library or imported"
        ]
    );
}
