use std::fs;

use dust_workspace::{detect_workspace_root, discover_libraries};
use tempfile::tempdir;

use crate::support::{test_annotations, write_file};

#[test]
fn detects_workspace_root_from_nested_directory_and_file_path() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    let nested_dir = root.path().join("lib/src/models");
    fs::create_dir_all(&nested_dir).unwrap();
    let nested_file = nested_dir.join("user.dart");
    write_file(&nested_file, "class User {}\n");

    let from_dir = detect_workspace_root(&nested_dir).unwrap();
    let from_file = detect_workspace_root(&nested_file).unwrap();

    assert_eq!(from_dir, root.path());
    assert_eq!(from_file, root.path());
}

#[test]
fn discover_libraries_scans_recursively_in_stable_order() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/src/team.dart"),
        "import 'package:dust_dart/derive.dart';\npart 'team.g.dart';\n@Derive([ToString()])\nclass Team {}\n",
    );
    write_file(
        &root.path().join("lib/user.dart"),
        "import 'package:dust_dart/derive.dart';\npart 'user.g.dart';\n@ToString()\nclass User {}\n",
    );
    write_file(&root.path().join("lib/user.g.dart"), "// generated\n");
    write_file(
        &root.path().join("lib/ignored.dart"),
        "part 'ignored.g.dart';\nclass Ignored {}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 2);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/src/team.dart")
    );
    assert_eq!(
        libraries[0].output_path,
        root.path().join("lib/src/team.g.dart")
    );
    assert_eq!(libraries[1].source_path, root.path().join("lib/user.dart"));
    assert_eq!(
        libraries[1].output_path,
        root.path().join("lib/user.g.dart")
    );
}

#[test]
fn discover_libraries_uses_configured_primary_suffix() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    write_file(
        &root.path().join("dust.yaml"),
        "outputs:\n  primary_suffix: .d.dart\n",
    );
    write_file(
        &root.path().join("lib/client.dart"),
        "import 'package:dust_dart/http.dart';\npart 'client.d.dart';\n@Client()\nabstract class ApiClient {}\n",
    );
    write_file(&root.path().join("lib/client.d.dart"), "// generated\n");

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(
        libraries[0].output_path,
        root.path().join("lib/client.d.dart")
    );
}

#[test]
fn discover_libraries_accepts_double_quoted_part_and_dust_import() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/client.dart"),
        "import 'package:dust_dart/http.dart';\npart \"client.g.dart\";\n@Client()\nabstract class ApiClient {}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/client.dart")
    );
}

#[test]
fn discover_libraries_accepts_dust_package_imports() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/prefixed.dart"),
        "part 'prefixed.g.dart';\nimport 'package:dust_dart/derive.dart' as d;\n@d.Derive([d.ToString()])\nclass Prefixed {}\n",
    );
    write_file(
        &root.path().join("lib/flutter_state.dart"),
        "import 'package:dust_flutter/state.dart' as f;\npart 'flutter_state.g.dart';\n@f.ViewModel()\nclass DemoViewModel {}\n",
    );
    write_file(
        &root.path().join("lib/ignored.dart"),
        "part 'ignored.g.dart';\n@Derive()\nclass Ignored {}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 2);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/flutter_state.dart")
    );
    assert_eq!(
        libraries[1].source_path,
        root.path().join("lib/prefixed.dart")
    );
}

#[test]
fn discover_libraries_accepts_dust_reexport_imports() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/route.dart"),
        "import 'package:dust_flutter/route.dart' show Router;\nexport 'package:dust_flutter/route.dart';\npart 'route.g.dart';\n@Router(initial: '/')\nclass AppRouter {}\n",
    );
    write_file(
        &root.path().join("lib/pages/home.dart"),
        "import '../route.dart';\npart 'home.g.dart';\n@Route('/', name: 'home')\nclass HomePage {}\n",
    );
    write_file(
        &root.path().join("lib/main.dart"),
        "import 'route.dart';\nclass App {\n  @override\n  String toString() => 'app';\n}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 2);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/pages/home.dart")
    );
    assert_eq!(libraries[1].source_path, root.path().join("lib/route.dart"));
}

#[test]
fn discover_libraries_accepts_escaped_quote_in_local_dust_reexport_uri() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/dust's.dart"),
        "export 'package:dust_dart/derive.dart';\n",
    );
    write_file(
        &root.path().join("lib/user.dart"),
        "import 'dust\\'s.dart';\npart 'user.g.dart';\n@Derive()\nclass User {}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(libraries[0].source_path, root.path().join("lib/user.dart"));
}

#[test]
fn discover_libraries_ignores_override_and_unknown_annotations() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/view.dart"),
        "part 'view.g.dart';\nclass Demo {\n  @override\n  String toString() => 'demo';\n}\n",
    );
    write_file(
        &root.path().join("lib/view_model.dart"),
        "part 'view_model.g.dart';\n@ViewModel()\nclass DemoViewModel {}\n",
    );

    let supported_annotations = test_annotations();
    let libraries = discover_libraries(root.path(), &supported_annotations).unwrap();

    assert!(libraries.is_empty());
}
