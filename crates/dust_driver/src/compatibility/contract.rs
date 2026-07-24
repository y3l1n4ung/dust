use std::collections::BTreeMap;

use dust_diagnostics::Diagnostic;
use serde_json::Value;

use super::COMPATIBILITY_CONTRACT;

/// Compatibility contract parsed from the embedded JSON file.
pub(super) struct CompatibilityContract {
    /// CLI compatibility entries.
    entries: Vec<CompatibilityEntry>,
    /// Suggested actions for incompatible versions.
    pub(super) actions: CompatibilityActions,
}

impl CompatibilityContract {
    /// Parses the embedded contract.
    pub(super) fn load() -> Result<Self, Diagnostic> {
        let value = serde_json::from_str::<Value>(COMPATIBILITY_CONTRACT).map_err(|error| {
            Diagnostic::error(format!(
                "failed to parse embedded Dust compatibility contract: {error}"
            ))
        })?;
        let entries = value
            .get("entries")
            .and_then(Value::as_array)
            .ok_or_else(|| malformed_contract("missing `entries` array"))?
            .iter()
            .map(CompatibilityEntry::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        let actions = CompatibilityActions::from_json(value.get("incompatibleVersionAction"));

        Ok(Self { entries, actions })
    }

    /// Finds the contract entry for the current CLI version.
    pub(super) fn entry_for_cli(
        &self,
        cli_version: &str,
    ) -> Result<&CompatibilityEntry, Diagnostic> {
        self.entries
            .iter()
            .find(|entry| entry.cli_version == cli_version)
            .ok_or_else(|| {
                Diagnostic::error(format!(
                    "Dust CLI {cli_version} has no embedded package compatibility entry"
                ))
                .with_note(self.actions.cli_too_old.clone())
            })
    }
}

/// One CLI version compatibility entry.
pub(super) struct CompatibilityEntry {
    /// Dust CLI version.
    cli_version: String,
    /// Package constraints keyed by Dart package name.
    pub(super) constraints: BTreeMap<String, String>,
}

impl CompatibilityEntry {
    /// Parses one compatibility entry from JSON.
    fn from_json(value: &Value) -> Result<Self, Diagnostic> {
        let cli_version = value
            .get("cliVersion")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed_contract("entry missing `cliVersion`"))?
            .to_owned();
        let constraints = value
            .get("packageConstraints")
            .and_then(Value::as_object)
            .ok_or_else(|| malformed_contract("entry missing `packageConstraints` object"))?
            .iter()
            .filter_map(|(package, constraint)| {
                constraint
                    .as_str()
                    .map(|constraint| (package.clone(), constraint.to_owned()))
            })
            .collect();

        Ok(Self {
            cli_version,
            constraints,
        })
    }
}

/// User-facing action strings from the compatibility contract.
pub(super) struct CompatibilityActions {
    /// Action when the CLI has no matching contract entry.
    cli_too_old: String,
    /// Action when a resolved package is older than the supported range.
    pub(super) package_too_old: String,
    /// Action when a resolved package is newer than the supported range.
    pub(super) package_too_new: String,
}

impl CompatibilityActions {
    /// Parses action strings with conservative defaults.
    fn from_json(value: Option<&Value>) -> Self {
        let cli_too_old = action_string(
            value,
            "cliTooOld",
            "Install a newer Dust CLI release before running generation.",
        );
        let package_too_old = action_string(
            value,
            "packageTooOld",
            "Upgrade the Dust package dependency in pubspec.yaml.",
        );
        let package_too_new = action_string(
            value,
            "packageTooNew",
            "Upgrade the Dust CLI first, or pin the package to a supported range.",
        );

        Self {
            cli_too_old,
            package_too_old,
            package_too_new,
        }
    }
}

/// Reads one action string from the contract.
fn action_string(value: Option<&Value>, key: &str, fallback: &str) -> String {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

/// Builds a diagnostic for a malformed embedded contract.
fn malformed_contract(message: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "invalid embedded Dust compatibility contract: {message}"
    ))
}
