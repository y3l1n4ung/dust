use std::fs;

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, WatchRequest, run_build, run_check, run_clean,
    run_watch,
};

use super::helpers::{write_dashboard_page, write_routing_workspace};
use crate::support::{make_pub_workspace_member, make_workspace};

#[test]
fn build_writes_route_output_only_from_router_root() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    let route_output = workspace.path().join("lib/route.g.dart");
    let dashboard_output = workspace.path().join("lib/pages/dashboard_page.g.dart");
    let not_found_output = workspace.path().join("lib/pages/not_found_page.g.dart");
    let source = fs::read_to_string(&route_output).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(route_output.exists());
    assert!(!dashboard_output.exists());
    assert!(!not_found_output.exists());
    assert!(result.build_artifacts.iter().any(|artifact| {
        artifact.source_path.ends_with("dashboard_page.dart")
            && artifact.routed
            && !artifact.written
    }));
    assert!(result.build_artifacts.iter().any(|artifact| {
        artifact.source_path.ends_with("not_found_page.dart")
            && artifact.routed
            && !artifact.written
    }));
    assert!(source.contains("import 'package:dust_flutter/route.dart';"));
    assert!(source.contains("import 'package:dust_test/pages/dashboard_page.dart';"));
    assert!(source.contains("final class DashboardRoute extends AppRoutePath"));
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'dashboard'"));
}
#[test]
fn build_refreshes_router_output_when_annotated_page_changes() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let first = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let second = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let second_source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();
    write_dashboard_page(workspace.path(), "home");
    let third = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    let source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();

    assert!(!first.has_errors(), "{:?}", first.diagnostics);
    assert!(!second.has_errors(), "{:?}", second.diagnostics);
    assert!(!third.has_errors(), "{:?}", third.diagnostics);
    assert_eq!(second.cache.as_ref().unwrap().misses, 0);
    assert!(second_source.contains("page: DashboardPage"));
    assert!(second_source.contains("name: 'dashboard'"));
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'home'"));
    assert!(source.contains("RouteNavigation<AppRoutePath> home()"));
    assert!(!source.contains("RouteNavigation<AppRoutePath> dashboard()"));
}

#[test]
fn check_reports_stale_route_output_before_build_and_fresh_after_build() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let before = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let after = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });

    assert!(!before.has_errors(), "{:?}", before.diagnostics);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert!(!after.has_errors(), "{:?}", after.diagnostics);
    assert!(
        before
            .checked_libraries
            .iter()
            .any(|library| library.output_path.ends_with("route.g.dart") && library.stale)
    );
    assert!(
        after
            .checked_libraries
            .iter()
            .any(|library| library.output_path.ends_with("route.g.dart") && !library.stale)
    );
}
#[test]
fn clean_removes_route_output_only_from_router_root() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert!(workspace.path().join("lib/route.g.dart").exists());

    let clean = run_clean(CleanRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!clean.has_errors(), "{:?}", clean.diagnostics);
    assert!(!workspace.path().join("lib/route.g.dart").exists());
    assert!(
        !workspace
            .path()
            .join("lib/pages/dashboard_page.g.dart")
            .exists()
    );
    assert!(clean.clean.unwrap().removed_files >= 1);
}

#[test]
fn watch_rebuilds_route_output_when_annotated_page_changes() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let root = workspace.path().to_path_buf();
    let modifier = std::thread::spawn({
        let root = root.clone();
        move || {
            std::thread::sleep(std::time::Duration::from_millis(1_000));
            write_dashboard_page(&root, "home");
        }
    });
    let result = run_watch(WatchRequest {
        cwd: root.clone(),
        fail_fast: true,
        jobs: None,
        poll_interval_ms: 50,
        max_cycles: Some(30),
    });
    modifier.join().unwrap();

    let source = fs::read_to_string(root.join("lib/route.g.dart")).unwrap();
    let watch = result.watch.as_ref().unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(watch.rebuild_batches, 1);
    assert!(source.contains("page: DashboardPage"));
    assert!(source.contains("name: 'home'"));
}
#[test]
fn route_generation_works_from_pub_workspace_member() {
    let (_workspace, package_root) = make_pub_workspace_member();
    write_routing_workspace(&package_root, "dashboard");

    let result = run_build(BuildRequest {
        cwd: package_root.clone(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let source = fs::read_to_string(package_root.join("lib/route.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(source.contains("import 'package:product_showcase/pages/dashboard_page.dart';"));
    assert!(source.contains("final class DashboardRoute extends AppRoutePath"));
}
