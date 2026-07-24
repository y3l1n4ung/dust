use std::{fs, path::Path};

use dust_cli::run_cli;
use serde_json::Value;

use super::helpers::{make_workspace, write_file};

#[test]
fn compatibility_contract_covers_v013_packages() {
    let root = repo_root();
    let contract = read_json(root.join("compatibility/dust-cli-packages.json").as_path());
    assert_eq!(contract["schemaVersion"], 1);
    let current_cli_version = workspace_version(root.join("Cargo.toml").as_path());
    assert_eq!(current_cli_version, "0.1.3");

    let entry = contract["entries"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["cliVersion"].as_str() == Some(&current_cli_version))
        })
        .expect("current CLI compatibility entry");
    let constraints = entry["packageConstraints"]
        .as_object()
        .expect("package constraints");

    assert_eq!(
        constraints["dust_dart"],
        format!(
            ">={} <0.2.0",
            package_version(root.join("packages/dust_dart/pubspec.yaml").as_path())
        )
    );
    assert_eq!(
        constraints["dust_flutter"],
        format!(
            ">={} <0.2.0",
            package_version(root.join("packages/dust_flutter/pubspec.yaml").as_path())
        )
    );
    assert_eq!(
        constraints["dust_db_sqlite3"],
        format!(
            ">={} <0.2.0",
            package_version(root.join("packages/dust_db_sqlite3/pubspec.yaml").as_path())
        )
    );
}

#[test]
fn release_workflow_validates_compatibility_contract() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/release.yml"))
        .expect("read release workflow");

    assert!(workflow.contains("Validate compatibility contract"));
    assert!(workflow.contains("scripts/validate_compatibility_contract.py"));
    assert!(workflow.contains("--release-tag \"$RELEASE_TAG\""));
}

#[test]
fn ci_runs_compatibility_script_self_test_and_real_check() {
    let workflow =
        fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).expect("read CI workflow");

    assert!(workflow.contains("Validate compatibility contract"));
    assert!(workflow.contains("scripts/validate_compatibility_contract.py --self-test"));
    assert!(workflow.contains("scripts/validate_compatibility_contract.py"));
}

#[test]
fn cli_build_renders_unsupported_package_version() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.2")]);
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let run = run_cli(["build", "--root", workspace.path().to_str().unwrap()]);

    assert_ne!(run.exit_code, 0);
    assert!(run.stdout.is_empty());
    assert!(run.stderr.contains("unsupported Dust package version"));
    assert!(run.stderr.contains("CLI 0.1.3"));
    assert!(run.stderr.contains("`dust_dart` >=0.1.3 <0.2.0"));
    assert!(run.stderr.contains("resolves 0.1.2"));
    assert!(run.stderr.contains("Upgrade the Dust package dependency"));
    assert!(!workspace.path().join("lib/user.g.dart").exists());
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

fn read_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("read json");
    serde_json::from_str(&contents).expect("parse json")
}

fn package_version(path: &Path) -> String {
    fs::read_to_string(path)
        .expect("read pubspec")
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .expect("pubspec version")
        .to_owned()
}

fn workspace_version(path: &Path) -> String {
    let mut in_workspace_package = false;
    for line in fs::read_to_string(path).expect("read Cargo.toml").lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if in_workspace_package {
            if let Some(version) = trimmed
                .strip_prefix("version = \"")
                .and_then(|value| value.strip_suffix('"'))
            {
                return version.to_owned();
            }
        }
    }

    panic!("workspace package version missing");
}

fn write_resolved_dust_packages(root: &Path, packages: &[(&str, &str)]) {
    for (name, version) in packages {
        write_file(
            &root.join(format!("deps/{name}/pubspec.yaml")),
            &format!("name: {name}\nversion: {version}\n"),
        );
    }

    let package_entries = packages
        .iter()
        .map(|(name, _)| {
            format!(
                r#"{{"name":"{name}","rootUri":"../deps/{name}","packageUri":"lib/","languageVersion":"3.6"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    write_file(
        &root.join(".dart_tool/package_config.json"),
        &format!(
            r#"{{"configVersion":2,"packages":[{{"name":"dust_test","rootUri":"../","packageUri":"lib/","languageVersion":"3.6"}},{package_entries}]}}"#
        ),
    );
}
