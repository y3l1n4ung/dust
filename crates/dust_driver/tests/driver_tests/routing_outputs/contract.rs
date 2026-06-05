use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::helpers::{assert_route_snapshot, write_routing_workspace};
use crate::support::make_workspace;

#[test]
fn build_matches_full_route_output_snapshot() {
    let workspace = make_workspace();
    write_routing_workspace(workspace.path(), "dashboard");

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
        db: Default::default(),
    });
    let source = fs::read_to_string(workspace.path().join("lib/route.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_route_snapshot("dashboard_route.g.dart", &source);
}
