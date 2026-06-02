use std::fs;

use dust_workspace::SupportedAnnotations;

pub(crate) fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

pub(crate) fn test_annotations() -> SupportedAnnotations {
    ["Derive", "ToString", "Client"].into_iter().collect()
}
