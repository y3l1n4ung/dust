use std::time::Instant;

use crate::{
    build::RegistrySelection,
    compatibility::doctor_package_compatibility,
    context::DriverContext,
    request::DoctorRequest,
    result::{CommandResult, DoctorReport},
};

/// Runs workspace and plugin readiness checks without parsing libraries.
pub fn run_doctor(request: DoctorRequest) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let DriverContext {
        workspace,
        registry,
        ..
    } = match DriverContext::load(&request.cwd, RegistrySelection::All) {
        Ok(context) => context,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let (package_compatibility, compatibility_diagnostics) =
        match doctor_package_compatibility(&workspace) {
            Ok(report) => report,
            Err(diagnostic) => {
                result.diagnostics.push(diagnostic);
                (Vec::new(), Vec::new())
            }
        };
    result.diagnostics.extend(compatibility_diagnostics);
    result.doctor = Some(DoctorReport {
        cli_version: env!("CARGO_PKG_VERSION").to_owned(),
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
        package_compatibility,
    });
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
