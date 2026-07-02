use dust_driver::{BuildRequest, CleanRequest, WatchRequest, run_build, run_clean, run_watch};

use super::support::{generated_output, make_pub_workspace_member, write_file};

#[test]
fn build_uses_member_cache_root_and_shared_package_config_for_pub_workspace() {
    let (workspace, package_root) = make_pub_workspace_member();
    write_file(
        &package_root.join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let first = run_build(BuildRequest {
        cwd: package_root.clone(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let second = run_build(BuildRequest {
        cwd: package_root.clone(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(!first.has_errors(), "{:?}", first.diagnostics);
    assert!(!second.has_errors(), "{:?}", second.diagnostics);
    assert_eq!(
        second.cache.as_ref().unwrap().path,
        package_root.join(".dart_tool/dust/build_cache_v1.json")
    );
    assert!(
        package_root
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );
    assert!(
        !workspace
            .path()
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );
    assert!(first.build_artifacts[0].written);
    assert!(second.build_artifacts[0].cached);
}

#[test]
fn watch_rebuilds_member_package_when_shared_workspace_config_changes() {
    let (workspace, package_root) = make_pub_workspace_member();
    write_file(
        &package_root.join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &package_root.join("lib/team.dart"),
        "part 'team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );

    let initial_output = package_root.join("lib/user.g.dart");
    let shared_package_config = workspace.path().join(".dart_tool/package_config.json");
    let modifier = std::thread::spawn(move || {
        wait_for_path(&initial_output);
        let replacement = shared_package_config.with_extension("json.next");
        std::fs::write(&replacement, "{\"configVersion\":3}\n").unwrap();
        std::fs::rename(replacement, shared_package_config).unwrap();
    });

    let result = run_watch(WatchRequest {
        cwd: package_root.clone(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 20,
        max_cycles: Some(50),
    });
    modifier.join().unwrap();

    let watch = result.watch.unwrap();
    assert_eq!(watch.rebuild_batches, 1);
    assert_eq!(
        watch.rebuilt_libraries,
        vec![
            package_root.join("lib/team.dart"),
            package_root.join("lib/user.dart")
        ]
    );
}

fn wait_for_path(path: &std::path::Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !path.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for `{}`",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn clean_only_clears_member_package_outputs_and_cache_in_pub_workspace() {
    let (workspace, package_root) = make_pub_workspace_member();
    write_file(
        &package_root.join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/root.g.dart"),
        &generated_output(""),
    );
    write_file(
        &workspace.path().join(".dart_tool/dust/build_cache_v1.json"),
        "{}\n",
    );

    let built = run_build(BuildRequest {
        cwd: package_root.clone(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    assert!(!built.has_errors());
    assert!(
        package_root
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );

    let result = run_clean(CleanRequest {
        cwd: package_root.clone(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let clean = result.clean.unwrap();
    assert_eq!(clean.package_root, package_root);
    assert!(clean.cache_cleared);
    assert!(!package_root.join("lib/user.g.dart").exists());
    assert!(!package_root.join(".dart_tool/dust").exists());
    assert!(workspace.path().join("lib/root.g.dart").exists());
    assert!(
        workspace
            .path()
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );
}
