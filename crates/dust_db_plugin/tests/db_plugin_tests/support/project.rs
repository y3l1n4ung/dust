use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn write_sqlite_project(root: &std::path::Path, source: &str) {
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::create_dir_all(root.join("migrations")).unwrap();
    fs::write(root.join("lib/db.dart"), source).unwrap();
    fs::write(
        root.join("migrations/001_schema.sql"),
        "CREATE TABLE users(id INTEGER PRIMARY KEY, display_name TEXT NOT NULL);\n",
    )
    .unwrap();
}

pub(crate) fn temp_root(name: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dust_db_plugin_{name}_{stamp}"))
}
