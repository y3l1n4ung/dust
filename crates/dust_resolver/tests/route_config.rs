//! Integration tests for resolver-normalized route configuration.

use dust_ir::NormalizedConfigIr;
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, resolve_library};
use dust_text::{FileId, SourceText};

#[test]
fn normalizes_app_route_from_parser_owned_values() {
    let source = SourceText::new(
        FileId::new(42),
        r#"
part 'project.g.dart';

@AppRoute(
  '/projects/:id',
  name: 'project',
  result: bool,
  shell: ui.AppShell,
  branch: 'mainTabs',
  guards: [auth.AuthGuard, BillingGuard],
  transition: const cupertino.CupertinoPageTransitionsBuilder(),
  fullscreenDialog: true,
  maintainState: false,
)
class ProjectPage {}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("AppRoute", "dust_flutter::AppRoute");

    let resolved = resolve_library(
        FileId::new(42),
        "lib/project.dart",
        "lib/project.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let Some(NormalizedConfigIr::Route(route)) =
        resolved.library.classes[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized AppRoute configuration");
    };
    assert_eq!(route.path, "/projects/:id");
    assert_eq!(route.name.as_deref(), Some("project"));
    assert_eq!(route.result_type.as_deref(), Some("bool"));
    assert_eq!(route.shell.as_deref(), Some("ui.AppShell"));
    assert_eq!(route.branch.as_deref(), Some("mainTabs"));
    assert_eq!(route.guards, ["auth.AuthGuard", "BillingGuard"]);
    assert!(route.guards_configured);
    assert_eq!(
        route.transition.as_deref(),
        Some("CupertinoPageTransitionsBuilder()")
    );
    assert!(route.fullscreen_dialog);
    assert!(!route.maintain_state);
}

#[test]
fn normalizes_app_router_from_parser_owned_values() {
    let source = SourceText::new(
        FileId::new(43),
        "part 'router.g.dart'; @AppRouter(initial: '/', notFound: '/404') class RootRouter {}",
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("AppRouter", "dust_flutter::AppRouter");

    let resolved = resolve_library(
        FileId::new(43),
        "lib/router.dart",
        "lib/router.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let Some(NormalizedConfigIr::Router(router)) =
        resolved.library.classes[0].configs[0].normalized.as_ref()
    else {
        panic!("expected normalized AppRouter configuration");
    };
    assert_eq!(router.initial.as_deref(), Some("/"));
    assert_eq!(router.not_found.as_deref(), Some("/404"));
}
