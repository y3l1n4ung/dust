use crate::{
    request::RouteFixturesRequest,
    result::{CommandResult, RouteFixtureRow, RouteFixturesReport},
    route_inspection::collect_route_inspection,
};

/// Runs read-only route deep-link fixture inspection across the discovered workspace.
pub fn run_route_fixtures(request: RouteFixturesRequest) -> CommandResult {
    let inspection = collect_route_inspection(&request.cwd);
    if inspection.scanned_files == 0 && inspection.result.has_errors() {
        return inspection.result;
    }
    let fixtures = dust_route_plugin::inspect::route_fixture_rows(&inspection.snapshots)
        .into_iter()
        .map(|fixture| RouteFixtureRow {
            route: fixture.route,
            case_name: fixture.case_name,
            valid: fixture.valid,
            shape: fixture.shape,
            uri: fixture.uri,
            expected: fixture.expected,
        })
        .collect();
    let mut result = inspection.result;
    result.route_fixtures = Some(RouteFixturesReport {
        scanned_files: inspection.scanned_files,
        fixtures,
    });
    result
}
