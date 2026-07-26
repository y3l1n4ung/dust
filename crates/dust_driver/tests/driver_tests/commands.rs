use std::sync::{Arc, Mutex};

use dust_driver::{
    BuildRequest, CheckRequest, CommandRequest, DoctorPackageCompatibilityStatus, DoctorRequest,
    ProgressEvent, ProgressPhase, run, run_build, run_build_with_progress, run_check, run_doctor,
};

use super::support::{
    DustImport, make_pub_workspace_member, make_workspace, write_dust_file,
    write_resolved_dust_packages,
};

#[test]
fn check_reports_stale_before_build_and_fresh_after_build() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
        db: Default::default(),
    });
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let second_check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
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
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.3")]);
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let doctor = result.doctor.as_ref().unwrap();

    assert_eq!(doctor.cli_version, "0.1.3");
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
            "dust_plugin_serde".to_owned(),
            "HttpClient".to_owned(),
            "Route".to_owned(),
            "ViewModel".to_owned(),
            "Database".to_owned(),
        ]
    );
    assert_eq!(
        doctor.libraries,
        vec![workspace.path().join("lib/user.dart")]
    );
    let dust_dart = doctor_package(doctor, "dust_dart");
    assert_eq!(
        dust_dart.status,
        DoctorPackageCompatibilityStatus::Compatible
    );
    assert!(dust_dart.used_by_workspace);
    assert_eq!(dust_dart.resolved_version.as_deref(), Some("0.1.3"));
    assert_eq!(
        dust_dart.supported_constraint.as_deref(),
        Some(">=0.1.3 <0.2.0")
    );
    assert_eq!(
        doctor_package(doctor, "dust_flutter").status,
        DoctorPackageCompatibilityStatus::NotResolved
    );
}

#[test]
fn doctor_reports_member_package_root_and_shared_package_config() {
    let (workspace, package_root) = make_pub_workspace_member();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.3")]);
    write_dust_file(
        &package_root.join("lib/user.dart"),
        &[DustImport::Derive],
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
    let doctor = result.doctor.as_ref().unwrap();

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
fn doctor_reports_too_old_dust_package() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.2")]);
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
    let doctor = result.doctor.as_ref().unwrap();
    let dust_dart = doctor_package(doctor, "dust_dart");

    assert!(result.has_errors());
    assert_eq!(dust_dart.status, DoctorPackageCompatibilityStatus::TooOld);
    assert_eq!(dust_dart.resolved_version.as_deref(), Some("0.1.2"));
    assert_eq!(
        dust_dart.action.as_deref(),
        Some("Upgrade the Dust package dependency in pubspec.yaml.")
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("unsupported Dust package version")
    );
}

#[test]
fn doctor_reports_too_new_dust_package() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_flutter", "0.2.0")]);
    write_dust_file(
        &workspace.path().join("lib/counter.dart"),
        &[DustImport::State],
        "part 'counter.g.dart';\n\
         @ViewModel()\n\
         class CounterViewModel {}\n",
    );

    let result = run_doctor(DoctorRequest {
        cwd: workspace.path().to_path_buf(),
    });
    let doctor = result.doctor.as_ref().unwrap();
    let dust_flutter = doctor_package(doctor, "dust_flutter");

    assert!(result.has_errors());
    assert_eq!(
        dust_flutter.status,
        DoctorPackageCompatibilityStatus::TooNew
    );
    assert_eq!(dust_flutter.resolved_version.as_deref(), Some("0.2.0"));
    assert_eq!(
        dust_flutter.action.as_deref(),
        Some("Upgrade the Dust CLI first, or pin the package to a supported range.")
    );
}

#[test]
fn doctor_reports_missing_used_dust_package() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
    let doctor = result.doctor.as_ref().unwrap();
    let dust_dart = doctor_package(doctor, "dust_dart");

    assert!(result.has_errors());
    assert_eq!(dust_dart.status, DoctorPackageCompatibilityStatus::Missing);
    assert!(dust_dart.used_by_workspace);
    assert_eq!(dust_dart.resolved_version, None);
    assert!(
        dust_dart
            .action
            .as_deref()
            .is_some_and(|action| action.contains("dart pub get"))
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
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
            db: Default::default(),
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

fn doctor_package<'a>(
    doctor: &'a dust_driver::DoctorReport,
    package_name: &str,
) -> &'a dust_driver::DoctorPackageCompatibility {
    doctor
        .package_compatibility
        .iter()
        .find(|package| package.package_name == package_name)
        .unwrap_or_else(|| panic!("missing doctor compatibility row for {package_name}"))
}
