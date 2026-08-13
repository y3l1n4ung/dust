use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;
use dust_route_plugin::register_plugin;

use crate::support::{
    constructor_param, defaulted_param, library_with_classes, positional_param, route_page_class,
    shell_class,
};

use super::support::{add_import, diagnostic_messages};

#[test]
fn accepts_local_route_shell_with_named_widget_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![constructor_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_route_shell_from_show_import() {
    let plugin = register_plugin();
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![page]);
    add_import(
        &mut library,
        "package:app/app_shell.dart",
        &["AppShell"],
        &[],
    );

    let diagnostics = plugin.validate(&library);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_route_shell_hidden_by_import() {
    let plugin = register_plugin();
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );
    let mut library = library_with_classes(vec![page]);
    add_import(
        &mut library,
        "package:app/app_shell.dart",
        &[],
        &["AppShell"],
    );

    let diagnostics = plugin.validate(&library);

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` must be declared in the same library or imported"
        ]
    );
}

#[test]
fn rejects_local_route_shell_without_named_widget_child() {
    let plugin = register_plugin();
    let shell = shell_class("AppShell", Vec::new());
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` needs an unnamed generative constructor with a required named `Widget child` parameter, for example `const AppShell({required Widget child, super.key})`"
        ]
    );
}

#[test]
fn rejects_local_route_shell_with_positional_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![positional_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` needs an unnamed generative constructor with a required named `Widget child` parameter, for example `const AppShell({required Widget child, super.key})`"
        ]
    );
}

#[test]
fn rejects_local_route_shell_with_nullable_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![constructor_param(
            "child",
            TypeIr::named("Widget").nullable(),
        )],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` needs an unnamed generative constructor with a required named `Widget child` parameter, for example `const AppShell({required Widget child, super.key})`"
        ]
    );
}

#[test]
fn rejects_local_route_shell_with_defaulted_child() {
    let plugin = register_plugin();
    let shell = shell_class(
        "AppShell",
        vec![defaulted_param("child", TypeIr::named("Widget"))],
    );
    let page = route_page_class(
        "DashboardPage",
        "('/dashboard', name: 'dashboard', shell: AppShell)",
        Vec::new(),
    );

    let diagnostics = plugin.validate(&library_with_classes(vec![shell, page]));

    assert_eq!(
        diagnostic_messages(&diagnostics),
        vec![
            "route shell `AppShell` on `DashboardPage` needs an unnamed generative constructor with a required named `Widget child` parameter, for example `const AppShell({required Widget child, super.key})`"
        ]
    );
}
