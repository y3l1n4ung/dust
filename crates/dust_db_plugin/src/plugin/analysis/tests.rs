use dust_plugin_api::{WorkspaceAnalysis, WorkspaceAnalysisBuilder};

use super::*;

/// Builds a workspace analysis from packed values, as the scan phase would.
fn analysis(values: &[(&str, &str)]) -> WorkspaceAnalysis {
    let mut builder = WorkspaceAnalysisBuilder::default();
    for (key, value) in values {
        builder.add_string_set_value(*key, *value);
    }
    builder.build()
}

/// Packs one value the way `collect_db_workspace_analysis` does.
fn packed(fields: &[&str]) -> String {
    fields.join(&UNIT.to_string())
}

#[test]
fn reads_databases_declared_in_the_same_package_only() {
    let mine = packed(&["app", "AppDatabase", "sqlite3", "./migrations"]);
    let theirs = packed(&["other", "OtherDatabase", "sqlite3", "./elsewhere"]);
    let analysis = analysis(&[(DATABASES_KEY, &mine), (DATABASES_KEY, &theirs)]);

    assert_eq!(
        package_databases(&analysis, "app"),
        vec![PackageDatabase {
            name: "AppDatabase".to_owned(),
            driver: DbDriver::Sqlite3,
            migrations: "./migrations".to_owned(),
        }]
    );
}

#[test]
fn orders_duplicate_databases_by_class_name() {
    let second = packed(&["app", "SecondDatabase", "postgres", "./b"]);
    let first = packed(&["app", "FirstDatabase", "sqlite3", "./a"]);
    let analysis = analysis(&[(DATABASES_KEY, &second), (DATABASES_KEY, &first)]);

    let names = package_databases(&analysis, "app")
        .into_iter()
        .map(|db| db.name)
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["FirstDatabase", "SecondDatabase"]);
}

#[test]
fn expands_rows_flattened_across_libraries() {
    let order = packed(&["app", "Order", "c:id", "f:Money"]);
    let money = packed(&["app", "Money", "c:amount", "c:currency"]);
    let analysis = analysis(&[(ROW_COLUMNS_KEY, &order), (ROW_COLUMNS_KEY, &money)]);

    let columns = package_row_column_map(&analysis, "app");
    assert_eq!(
        columns.get("Order"),
        Some(
            &["id", "amount", "currency"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
}

#[test]
fn a_flatten_cycle_stops_rather_than_recursing() {
    let left = packed(&["app", "Left", "c:left_id", "f:Right"]);
    let right = packed(&["app", "Right", "c:right_id", "f:Left"]);
    let analysis = analysis(&[(ROW_COLUMNS_KEY, &left), (ROW_COLUMNS_KEY, &right)]);

    let columns = package_row_column_map(&analysis, "app");
    assert_eq!(
        columns.get("Left"),
        Some(
            &["left_id", "right_id"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
}

#[test]
fn a_flatten_target_outside_the_package_contributes_nothing() {
    let order = packed(&["app", "Order", "c:id", "f:Absent"]);
    let analysis = analysis(&[(ROW_COLUMNS_KEY, &order)]);

    let columns = package_row_column_map(&analysis, "app");
    assert_eq!(
        columns.get("Order"),
        Some(&["id"].into_iter().map(str::to_owned).collect())
    );
}

#[test]
fn malformed_values_are_skipped_rather_than_panicking() {
    let analysis = analysis(&[
        (DATABASES_KEY, "app"),
        (DATABASES_KEY, &packed(&["app", "Db", "mysql", "./m"])),
        (ROW_COLUMNS_KEY, "app"),
    ]);

    assert!(package_databases(&analysis, "app").is_empty());
    assert!(package_row_column_map(&analysis, "app").is_empty());
}
