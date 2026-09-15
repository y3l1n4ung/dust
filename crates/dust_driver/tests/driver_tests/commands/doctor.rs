//! Doctor's package compatibility reporting.

use super::*;

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
    write_resolved_dust_packages(workspace.path(), &[("dust_flutter", "0.4.0")]);
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
    assert_eq!(dust_flutter.resolved_version.as_deref(), Some("0.4.0"));
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
