use std::sync::{Arc, Mutex};

use dust_driver::{
    BuildRequest, CheckRequest, CommandRequest, DoctorRequest, ProgressEvent, ProgressPhase, run,
    run_build, run_build_with_progress, run_check, run_doctor,
};

use super::support::{make_pub_workspace_member, make_workspace, write_file};

#[test]
fn check_reports_stale_before_build_and_fresh_after_build() {
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

    let first_check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });
    let second_check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert_eq!(first_check.checked_libraries.len(), 1);
    assert!(first_check.checked_libraries[0].stale);
    assert!(build.build_artifacts[0].written);
    assert_eq!(second_check.checked_libraries.len(), 1);
    assert!(!second_check.checked_libraries[0].stale);
}

#[test]
fn doctor_reports_workspace_and_registered_plugins() {
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

    let result = run_doctor(DoctorRequest {
        cwd: workspace.path().to_path_buf(),
    });
    let doctor = result.doctor.unwrap();

    assert_eq!(doctor.package_root, workspace.path());
    assert_eq!(
        doctor.package_config_path,
        workspace.path().join(".dart_tool/package_config.json")
    );
    assert_eq!(doctor.library_count, 1);
    assert_eq!(
        doctor.plugin_names,
        vec![
            "dust_plugin_derive".to_owned(),
            "dust_plugin_serde".to_owned()
        ]
    );
    assert_eq!(
        doctor.libraries,
        vec![workspace.path().join("lib/user.dart")]
    );
}

#[test]
fn doctor_reports_member_package_root_and_shared_package_config() {
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

    let result = run_doctor(DoctorRequest {
        cwd: package_root.clone(),
    });
    let doctor = result.doctor.unwrap();

    assert_eq!(doctor.package_root, package_root);
    assert_eq!(
        doctor.package_config_path,
        workspace.path().join(".dart_tool/package_config.json")
    );
    assert_eq!(doctor.library_count, 1);
    assert_eq!(
        doctor.libraries,
        vec![doctor.package_root.join("lib/user.dart")]
    );
}

#[test]
fn run_dispatches_supported_commands() {
    let workspace = make_workspace();
    let result = run(CommandRequest::Doctor(DoctorRequest {
        cwd: workspace.path().to_path_buf(),
    }));

    assert!(result.doctor.is_some());
}

#[test]
fn build_emits_progress_events() {
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
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);

    let result = run_build_with_progress(
        BuildRequest {
            cwd: workspace.path().to_path_buf(),
            fail_fast: false,
            jobs: Some(2),
        },
        move |event| {
            sink.lock().unwrap().push(event);
        },
    );

    assert!(!result.has_errors());
    let events = events.lock().unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        ProgressEvent::StartedBatch {
            phase: ProgressPhase::Build,
            total: 1
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        ProgressEvent::FinishedLibrary {
            phase: ProgressPhase::Build,
            completed: 1,
            total: 1,
            cached: false,
            ..
        }
    )));
}
