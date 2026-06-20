use dust_workspace::{PackageConfigKind, discover_workspace};
use tempfile::tempdir;

use crate::support::{test_annotations, write_file};

#[test]
fn discover_workspace_composes_root_config_and_library_scan() {
    let root = tempdir().unwrap();
    let workspace_root = root.path();
    let package_root = workspace_root.join("examples/product_showcase");
    write_file(
        &workspace_root.join("pubspec.yaml"),
        "name: dust_workspace\n",
    );
    write_file(
        &workspace_root.join(".dart_tool/package_config.json"),
        "{\"configVersion\":2}\n",
    );
    write_file(
        &workspace_root.join("lib/root.dart"),
        "part 'root.g.dart';\n@ToString()\nclass Root {}\n",
    );
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1}\n",
    );
    write_file(
        &package_root.join("lib/models/user.dart"),
        "import 'package:dust_dart/derive.dart';\npart 'user.g.dart';\n@Derive([ToString(), Eq()])\nclass User {}\n",
    );

    let nested = package_root.join("lib/models");
    let supported_annotations = test_annotations();
    let plan = discover_workspace(&nested, &supported_annotations).unwrap();

    assert_eq!(plan.package_root, package_root);
    assert_eq!(plan.cache_root, plan.package_root);
    assert_eq!(plan.package_name, "product_showcase");
    assert_eq!(
        plan.package_config.path,
        workspace_root.join(".dart_tool/package_config.json")
    );
    assert_eq!(
        plan.package_config.kind,
        PackageConfigKind::WorkspaceShared {
            package_graph_path: plan.package_root.join(".dart_tool/package_graph.json"),
        }
    );
    assert_eq!(plan.libraries.len(), 1);
    assert_eq!(
        plan.libraries[0].output_path,
        plan.package_root.join("lib/models/user.g.dart")
    );
    assert_eq!(plan.dust_config.outputs.primary_suffix, ".g.dart");
}
