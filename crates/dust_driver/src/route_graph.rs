use crate::{
    request::RouteGraphRequest,
    result::{CommandResult, RouteGraphNode, RouteGraphReport},
    route_inspection::collect_route_inspection,
};

/// Runs read-only route graph inspection across the discovered workspace.
pub fn run_route_graph(request: RouteGraphRequest) -> CommandResult {
    let inspection = collect_route_inspection(&request.cwd);
    if inspection.scanned_files == 0 && inspection.result.has_errors() {
        return inspection.result;
    }
    let nodes = dust_route_plugin::inspect::route_graph_nodes(&inspection.snapshots)
        .into_iter()
        .map(|node| RouteGraphNode {
            name: node.name,
            path: node.path,
            parent_path: node.parent_path,
            page: node.page,
            shell: node.shell,
            branch: node.branch,
            guards: node.guards,
        })
        .collect();
    let mut result = inspection.result;
    result.route_graph = Some(RouteGraphReport {
        scanned_files: inspection.scanned_files,
        nodes,
    });
    result
}
