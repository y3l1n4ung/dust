/// Embedded compatibility contract parsing.
mod contract;
/// Dart package_config and pubspec version resolution.
mod package_config;
/// Minimal semantic version and constraint handling.
mod version;

use dust_diagnostics::Diagnostic;
use dust_workspace::WorkspacePlan;

use std::collections::BTreeSet;

use self::{
    contract::CompatibilityContract,
    package_config::{read_package_version, resolved_dust_package_roots},
    version::{VersionConstraint, VersionMismatch},
};
use crate::result::{DoctorPackageCompatibility, DoctorPackageCompatibilityStatus};

/// Embedded CLI-to-package compatibility contract.
const COMPATIBILITY_CONTRACT: &str = include_str!("../../../compatibility/dust-cli-packages.json");

/// Dust packages whose public runtime APIs can be called by generated code.
const DUST_PACKAGES: &[&str] = &["dust_dart", "dust_flutter", "dust_db_sqlite3"];

/// Validates resolved Dust runtime package versions for a workspace.
pub(crate) fn validate_workspace_package_versions(
    workspace: &WorkspacePlan,
) -> Result<(), Diagnostic> {
    if workspace.dust_packages.is_empty() {
        return Ok(());
    }

    let resolved_packages = resolved_dust_package_roots(&workspace.package_config.path)?;
    if resolved_packages.is_empty() {
        return Ok(());
    }

    let cli_version = env!("CARGO_PKG_VERSION");
    let contract = CompatibilityContract::load()?;
    let entry = contract.entry_for_cli(cli_version)?;

    for package in workspace
        .dust_packages
        .iter()
        .filter(|package| is_dust_package(package))
    {
        let Some(package_root) = resolved_packages.get(package.as_str()) else {
            continue;
        };
        let (version_text, version) = read_package_version(package_root, package)?;
        let Some(constraint_text) = entry.constraints.get(package.as_str()) else {
            return Err(Diagnostic::error(format!(
                "Dust CLI {cli_version} has no compatibility rule for `{package}`"
            )));
        };
        let constraint = VersionConstraint::parse(constraint_text).map_err(|error| {
            Diagnostic::error(format!(
                "invalid compatibility rule for `{package}` in embedded Dust contract: {error}"
            ))
        })?;

        if !constraint.is_satisfied_by(&version) {
            let action = constraint.mismatch_action(&version, &contract.actions);
            return Err(Diagnostic::error(format!(
                "unsupported Dust package version: CLI {cli_version} supports `{package}` {constraint_text}, but package_config resolves {version_text}"
            ))
            .with_note(action));
        }
    }

    Ok(())
}

/// Collects `dust doctor` package compatibility rows and diagnostics.
pub(crate) fn doctor_package_compatibility(
    workspace: &WorkspacePlan,
) -> Result<(Vec<DoctorPackageCompatibility>, Vec<Diagnostic>), Diagnostic> {
    let resolved_packages = resolved_dust_package_roots(&workspace.package_config.path)?;
    let cli_version = env!("CARGO_PKG_VERSION");
    let contract = CompatibilityContract::load()?;
    let entry = contract.entry_for_cli(cli_version)?;
    let used_packages = workspace
        .dust_packages
        .iter()
        .filter(|package| is_dust_package(package))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();

    for package in DUST_PACKAGES {
        let supported_constraint = entry.constraints.get(*package).cloned();
        let used_by_workspace = used_packages.contains(*package);
        let Some(package_root) = resolved_packages.get(*package) else {
            let action = used_by_workspace.then(|| {
                format!(
                    "Add `{package}` to pubspec.yaml and run `dart pub get` or `flutter pub get`."
                )
            });
            if let Some(action) = &action {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "missing Dust package resolution: workspace imports `{package}`, but package_config does not resolve it"
                    ))
                    .with_note(action.clone()),
                );
            }
            rows.push(DoctorPackageCompatibility {
                package_name: (*package).to_owned(),
                used_by_workspace,
                resolved_version: None,
                supported_constraint,
                status: if used_by_workspace {
                    DoctorPackageCompatibilityStatus::Missing
                } else {
                    DoctorPackageCompatibilityStatus::NotResolved
                },
                action,
            });
            continue;
        };

        let (version_text, version) = read_package_version(package_root, package)?;
        let Some(constraint_text) = supported_constraint.as_deref() else {
            let message =
                format!("Dust CLI {cli_version} has no compatibility rule for `{package}`");
            diagnostics.push(Diagnostic::error(message.clone()));
            rows.push(DoctorPackageCompatibility {
                package_name: (*package).to_owned(),
                used_by_workspace,
                resolved_version: Some(version_text),
                supported_constraint,
                status: DoctorPackageCompatibilityStatus::UnknownRule,
                action: Some(message),
            });
            continue;
        };
        let constraint = VersionConstraint::parse(constraint_text).map_err(|error| {
            Diagnostic::error(format!(
                "invalid compatibility rule for `{package}` in embedded Dust contract: {error}"
            ))
        })?;

        if constraint.is_satisfied_by(&version) {
            rows.push(DoctorPackageCompatibility {
                package_name: (*package).to_owned(),
                used_by_workspace,
                resolved_version: Some(version_text),
                supported_constraint,
                status: DoctorPackageCompatibilityStatus::Compatible,
                action: None,
            });
            continue;
        }

        let action = constraint
            .mismatch_action(&version, &contract.actions)
            .to_owned();
        let status = match constraint
            .mismatch_kind(&version)
            .unwrap_or(VersionMismatch::TooNew)
        {
            VersionMismatch::TooOld => DoctorPackageCompatibilityStatus::TooOld,
            VersionMismatch::TooNew => DoctorPackageCompatibilityStatus::TooNew,
        };
        diagnostics.push(
            Diagnostic::error(format!(
                "unsupported Dust package version: CLI {cli_version} supports `{package}` {constraint_text}, but package_config resolves {version_text}"
            ))
            .with_note(action.clone()),
        );
        rows.push(DoctorPackageCompatibility {
            package_name: (*package).to_owned(),
            used_by_workspace,
            resolved_version: Some(version_text),
            supported_constraint,
            status,
            action: Some(action),
        });
    }

    Ok((rows, diagnostics))
}

/// Returns whether the package name is part of Dust's version contract.
fn is_dust_package(package: &str) -> bool {
    DUST_PACKAGES.contains(&package)
}
