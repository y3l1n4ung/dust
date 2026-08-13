use crate::{
    request::RouteTableRequest,
    result::{CommandResult, RouteTableReport, RouteTableRow},
    route_inspection::collect_route_inspection,
};

/// Runs read-only route table inspection across the discovered workspace.
pub fn run_route_table(request: RouteTableRequest) -> CommandResult {
    let inspection = collect_route_inspection(&request.cwd);
    if inspection.scanned_files == 0 && inspection.result.has_errors() {
        return inspection.result;
    }
    let routes = dust_route_plugin::inspect::route_table_rows(&inspection.snapshots)
        .into_iter()
        .map(|row| RouteTableRow {
            name: row.name,
            path: row.path,
            page: row.page,
            shell: row.shell,
            branch: row.branch,
            guards: row.guards,
            result_type: row.result_type,
        })
        .collect();
    let mut result = inspection.result;
    result.route_table = Some(RouteTableReport {
        scanned_files: inspection.scanned_files,
        routes,
    });
    result
}
