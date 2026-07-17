use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// Migration file that Dust applies during normal schema setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MigrationFile {
    /// File name used as the migration identity.
    pub(crate) name: String,
    /// Absolute path to the migration source file.
    pub(crate) path: PathBuf,
}

/// Returns sorted migration files that should be applied to the database.
///
/// Simple `*.sql` migrations are applied directly. SQLx reversible migrations
/// use `*.up.sql` and `*.down.sql` pairs; Dust applies only the up migration
/// during normal validation and runtime startup.
pub(crate) fn applied_migration_files(path: &Path) -> Result<Vec<MigrationFile>, String> {
    let mut simple = BTreeMap::<String, MigrationFile>::new();
    let mut up = BTreeMap::<String, MigrationFile>::new();
    let mut down = BTreeSet::<String>::new();

    for entry in fs::read_dir(path)
        .map_err(|error| format!("failed to read migrations `{}`: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read migration entry in `{}`: {error}",
                path.display()
            )
        })?;
        let file_path = entry.path();
        if file_path.extension().and_then(|ext| ext.to_str()) != Some("sql") {
            continue;
        }
        let Some(name) = file_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::to_owned)
        else {
            return Err(format!(
                "migration file `{}` is not valid UTF-8",
                file_path.display()
            ));
        };

        if let Some(id) = name.strip_suffix(".up.sql") {
            validate_reversible_id(path, id, &name)?;
            up.insert(
                id.to_owned(),
                MigrationFile {
                    name,
                    path: file_path,
                },
            );
        } else if let Some(id) = name.strip_suffix(".down.sql") {
            validate_reversible_id(path, id, &name)?;
            down.insert(id.to_owned());
        } else if let Some(id) = name.strip_suffix(".sql") {
            simple.insert(
                id.to_owned(),
                MigrationFile {
                    name,
                    path: file_path,
                },
            );
        }
    }

    for id in simple.keys() {
        if up.contains_key(id) || down.contains(id) {
            return Err(format!(
                "Database migrations contain both simple and reversible files for `{id}` in `{}`",
                path.display()
            ));
        }
    }
    for id in up.keys() {
        if !down.contains(id) {
            return Err(format!(
                "Database reversible migration `{id}` in `{}` has an .up.sql file without matching .down.sql file",
                path.display()
            ));
        }
    }
    for id in &down {
        if !up.contains_key(id) {
            return Err(format!(
                "Database reversible migration `{id}` in `{}` has a .down.sql file without matching .up.sql file",
                path.display()
            ));
        }
    }

    let mut files = simple
        .into_values()
        .chain(up.into_values())
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(files)
}

/// Validates the shared name segment of a reversible migration pair.
fn validate_reversible_id(path: &Path, id: &str, name: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err(format!(
            "Database reversible migration `{name}` in `{}` is missing a migration name",
            path.display()
        ));
    }
    Ok(())
}
