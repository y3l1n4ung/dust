use std::{fs, thread, time::Duration};

use dust_driver::{CommandRequest, WatchRequest, run, run_watch};

use super::support::{make_workspace, write_file};

#[test]
fn watch_runs_initial_build_for_existing_candidates() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 10,
        max_cycles: Some(1),
    });

    assert!(workspace.path().join("lib/user.g.dart").exists());
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(result.watch.as_ref().unwrap().cycles, 1);
    assert_eq!(result.watch.as_ref().unwrap().rebuild_batches, 0);
}

#[test]
fn watch_rebuilds_only_the_changed_library() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/team.dart"),
        "part 'team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );

    let root = workspace.path().to_path_buf();
    let user_path = root.join("lib/user.dart");
    let modifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        write_file(
            &user_path,
            "part 'user.g.dart';\n\
             @ToString()\n\
             class User {\n\
               final String id;\n\
               final int age;\n\
               const User(this.id, this.age);\n\
             }\n",
        );
    });

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 20,
        max_cycles: Some(3),
    });
    modifier.join().unwrap();

    let watch = result.watch.unwrap();
    let user_output = fs::read_to_string(workspace.path().join("lib/user.g.dart")).unwrap();
    let team_output = fs::read_to_string(workspace.path().join("lib/team.g.dart")).unwrap();

    assert_eq!(watch.rebuild_batches, 1);
    assert_eq!(
        watch.rebuilt_libraries,
        vec![workspace.path().join("lib/user.dart")]
    );
    assert!(user_output.contains("return 'User('"));
    assert!(user_output.contains("'id: ${_dustSelf.id}, '"));
    assert!(user_output.contains("'age: ${_dustSelf.age}'"));
    assert!(team_output.contains("Team copyWith({"));
    assert_eq!(result.build_artifacts.len(), 3);
}

#[test]
fn watch_rebuilds_all_libraries_when_package_config_changes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/team.dart"),
        "part 'team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );

    let root = workspace.path().to_path_buf();
    let package_config = root.join(".dart_tool/package_config.json");
    let modifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        write_file(&package_config, "{\"configVersion\":2}\n");
    });

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 20,
        max_cycles: Some(3),
    });
    modifier.join().unwrap();

    let watch = result.watch.unwrap();
    assert_eq!(watch.rebuild_batches, 1);
    assert_eq!(
        watch.rebuilt_libraries,
        vec![
            workspace.path().join("lib/team.dart"),
            workspace.path().join("lib/user.dart"),
        ]
    );
}

#[test]
fn run_dispatches_watch_requests() {
    let workspace = make_workspace();
    let result = run(CommandRequest::Watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 10,
        max_cycles: Some(1),
    }));

    assert!(result.watch.is_some());
}
