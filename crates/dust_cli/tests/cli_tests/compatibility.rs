use std::{fs, path::Path};

use serde_json::Value;

#[test]
fn compatibility_contract_covers_v013_packages() {
    let root = repo_root();
    let contract = read_json(root.join("compatibility/dust-cli-packages.json").as_path());
    assert_eq!(contract["schemaVersion"], 1);

    let entry = contract["entries"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["cliVersion"].as_str() == Some("0.1.3"))
        })
        .expect("v0.1.3 compatibility entry");
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
    assert!(workflow.contains("compatibility/dust-cli-packages.json"));
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
