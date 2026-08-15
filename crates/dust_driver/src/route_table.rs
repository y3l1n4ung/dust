use crate::{
    request::RouteTableRequest,
    result::{CommandResult, RouteInspectionEntry, RouteTableReport},
    route_inspection::collect_route_inspection,
};

/// Runs read-only route table inspection across the discovered workspace.
pub fn run_route_table(request: RouteTableRequest) -> CommandResult {
    let inspection = collect_route_inspection(&request.cwd);
    if inspection.scanned_files == 0 && inspection.result.has_errors() {
        return inspection.result;
    }
    let routes = dust_route_plugin::inspect::route_inspection_entries(&inspection.snapshots)
        .into_iter()
        .map(|entry| RouteInspectionEntry {
            name: entry.name,
            path: entry.path,
            page: entry.page,
            shell: entry.shell,
            branch: entry.branch,
            guards: entry.guards,
            requires_auth: entry.requires_auth,
            result_type: entry.result_type,
        })
        .collect();
    let mut result = inspection.result;
    result.route_table = Some(RouteTableReport {
        scanned_files: inspection.scanned_files,
        routes,
    });
    result
}
