use std::fs;

use dust_driver::{BuildRequest, CheckRequest, CleanRequest, run_build, run_check, run_clean};

use super::support::{make_workspace, write_file};

#[test]
fn build_writes_http_client_auxiliary_test_output() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/api.dart"),
        "part 'api.g.dart';\n\
         @HttpClient()\n\
         @GenerateTest()\n\
         abstract interface class Api {\n\
           factory Api(Dio dio, {String? baseUrl}) = _$Api;\n\
           @GET('/users/{id}')\n\
           Future<User> getUser(@Path('id') String id);\n\
         }\n\
         class User {\n\
           const User();\n\
           factory User.fromJson(Map<String, dynamic> json) => const User();\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    let primary = workspace.path().join("lib/api.g.dart");
    let auxiliary = workspace.path().join("test/generated/api_test.dart");
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(primary.exists());
    assert!(auxiliary.exists(), "{:?}", result.build_artifacts);
    let auxiliary_source = fs::read_to_string(&auxiliary).unwrap();
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(
        result.build_artifacts[0].auxiliary_output_paths,
        vec![auxiliary.clone()]
    );
    assert!(auxiliary_source.contains("void main() {"));
    assert!(auxiliary_source.contains("group('Api request mapping'"));
    assert!(auxiliary_source.contains("package:dust_test/api.dart"));
    assert!(auxiliary_source.contains("getUser("), "{auxiliary_source}");
}

#[test]
fn check_marks_http_client_output_stale_when_auxiliary_file_is_missing() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/api.dart"),
        "part 'api.g.dart';\n\
         @HttpClient()\n\
         @GenerateTest()\n\
         abstract interface class Api {\n\
           factory Api(Dio dio, {String? baseUrl}) = _$Api;\n\
           @GET('/users/{id}')\n\
           Future<User> getUser(@Path('id') String id);\n\
         }\n\
         class User {\n\
           const User();\n\
           factory User.fromJson(Map<String, dynamic> json) => const User();\n\
         }\n",
    );

    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    assert!(!build.has_errors(), "{:?}", build.diagnostics);

    fs::remove_file(workspace.path().join("test/generated/api_test.dart")).unwrap();
    let check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert_eq!(check.checked_libraries.len(), 1);
    assert!(check.checked_libraries[0].stale);
    assert_eq!(
        check.checked_libraries[0].auxiliary_output_paths,
        vec![workspace.path().join("test/generated/api_test.dart")]
    );
}

#[test]
fn clean_removes_http_client_auxiliary_test_output() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/api.dart"),
        "part 'api.g.dart';\n\
         @HttpClient()\n\
         @GenerateTest()\n\
         abstract interface class Api {\n\
           factory Api(Dio dio, {String? baseUrl}) = _$Api;\n\
           @GET('/users/{id}')\n\
           Future<User> getUser(@Path('id') String id);\n\
         }\n\
         class User {\n\
           const User();\n\
           factory User.fromJson(Map<String, dynamic> json) => const User();\n\
         }\n",
    );

    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    assert!(!build.has_errors(), "{:?}", build.diagnostics);

    let result = run_clean(CleanRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!workspace.path().join("lib/api.g.dart").exists());
    assert!(
        !workspace
            .path()
            .join("test/generated/api_test.dart")
            .exists()
    );
    assert!(result.clean.unwrap().removed_files >= 2);
}
