use std::{
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

/// The discovered package configuration for one Dart workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageConfig {
    /// The absolute path to `.dart_tool/package_config.json`.
    pub path: PathBuf,
    /// How the package configuration was resolved for this package root.
    pub kind: PackageConfigKind,
}

/// The origin of the resolved package configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageConfigKind {
    /// The package root contains its own `.dart_tool/package_config.json`.
    Direct,
    /// The package root is a pub workspace member that points at a shared config.
    WorkspaceShared {
        /// The member package's `.dart_tool/package_graph.json` signal file.
        package_graph_path: PathBuf,
    },
}

/// Loads the package configuration path for the detected package root.
pub fn load_package_config(package_root: &Path) -> Result<PackageConfig, Diagnostic> {
    let path = package_root.join(".dart_tool/package_config.json");
    if path.is_file() {
        return Ok(PackageConfig {
            path,
            kind: PackageConfigKind::Direct,
        });
    }

    let package_graph_path = package_root.join(".dart_tool/package_graph.json");
    if !package_graph_path.is_file() {
        return Err(Diagnostic::error(format!(
            "missing package configuration at `{}`",
            path.display()
        )));
    }

    fs::read_to_string(&package_graph_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read workspace package graph `{}`: {error}",
            package_graph_path.display()
        ))
    })?;

    let Some(mut current) = package_root.parent().map(Path::to_path_buf) else {
        return Err(missing_shared_workspace_config(
            package_root,
            &package_graph_path,
        ));
    };

    loop {
        let candidate = current.join(".dart_tool/package_config.json");
        if candidate.is_file() {
            return Ok(PackageConfig {
                path: candidate,
                kind: PackageConfigKind::WorkspaceShared { package_graph_path },
            });
        }

        if !current.pop() {
            break;
        }
    }

    Err(missing_shared_workspace_config(
        package_root,
        &package_graph_path,
    ))
}

fn missing_shared_workspace_config(package_root: &Path, package_graph_path: &Path) -> Diagnostic {
    Diagnostic::error(format!(
        "workspace member `{}` uses `{}` but no shared package configuration was found above it; expected an ancestor `.dart_tool/package_config.json`",
        package_root.display(),
        package_graph_path.display()
    ))
}
