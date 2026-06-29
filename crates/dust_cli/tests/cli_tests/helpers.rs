use std::fs;

use tempfile::tempdir;

pub(crate) fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, fixture_contents(path, contents)).expect("write file");
}

fn fixture_contents(path: &std::path::Path, contents: &str) -> String {
    if path.extension().and_then(|ext| ext.to_str()) != Some("dart")
        || contents.contains("package:dust_dart/")
        || contents.contains("package:dust_flutter/")
    {
        return contents.to_owned();
    }

    let Some(import) = fixture_dust_import(contents) else {
        return contents.to_owned();
    };

    format!("{import}{contents}")
}

fn fixture_dust_import(contents: &str) -> Option<String> {
    let mut imports = Vec::new();
    let has = |name: &str| has_annotation(contents, name);

    if has("AppRoute") || has("AppRouter") {
        imports.push("import 'package:dust_flutter/route.dart';\n");
    }
    if has("Derive")
        || has("ToString")
        || has("Debug")
        || has("Eq")
        || has("CopyWith")
        || has("Validate")
        || has("SerDe")
        || contents.contains("Serialize()")
        || contents.contains("Deserialize()")
    {
        imports.push("import 'package:dust_dart/derive.dart';\n");
    }

    if imports.is_empty() {
        None
    } else {
        Some(imports.concat())
    }
}

fn has_annotation(contents: &str, name: &str) -> bool {
    contents.match_indices('@').any(|(index, _)| {
        let mut start = index + 1;
        let bytes = contents.as_bytes();
        while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
            start += 1;
        }
        let mut end = start;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
        {
            end += 1;
        }
        contents[start..end].rsplit('.').next() == Some(name)
    })
}

pub(crate) fn make_workspace() -> tempfile::TempDir {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    root
}

pub(crate) fn make_pub_workspace_member() -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempdir().unwrap();
    write_file(
        &root.path().join("pubspec.yaml"),
        "name: dust_workspace\nworkspace:\n  - examples/product_showcase\n",
    );
    write_file(
        &root.path().join(".dart_tool/package_config.json"),
        "{\"configVersion\":2,\"packages\":[]}\n",
    );
    let package_root = root.path().join("examples/product_showcase");
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\nresolution: workspace\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1,\"roots\":[\"product_showcase\"],\"packages\":[]}\n",
    );
    (root, package_root)
}

#[cfg(test)]
mod tests {
    use super::fixture_dust_import;

    #[test]
    fn fixture_dust_import_detects_prefixed_and_mixed_annotations() {
        let contents = "@r.AppRoute('/profile')\n@ d.Derive([d.ToString()])\nclass Profile {}\n";

        assert_eq!(
            fixture_dust_import(contents).as_deref(),
            Some(
                "import 'package:dust_flutter/route.dart';\nimport 'package:dust_dart/derive.dart';\n"
            )
        );
    }
}
