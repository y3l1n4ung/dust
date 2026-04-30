use std::time::Instant;

use dust_workspace::discover_workspace;

use crate::{
    build::default_registry,
    request::DoctorRequest,
    result::{CommandResult, DoctorReport},
};

/// Runs workspace and plugin readiness checks without parsing libraries.
pub fn run_doctor(request: DoctorRequest) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let workspace = match discover_workspace(&request.cwd) {
        Ok(workspace) => workspace,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let registry = default_registry();
    result.doctor = Some(DoctorReport {
        package_root: workspace.package_root,
        package_config_path: workspace.package_config.path,
        library_count: workspace.libraries.len(),
        plugin_names: registry
            .plugin_names()
            .into_iter()
            .map(str::to_owned)
            .collect(),
        libraries: workspace
            .libraries
            .into_iter()
            .map(|library| library.source_path)
            .collect(),
    });
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
