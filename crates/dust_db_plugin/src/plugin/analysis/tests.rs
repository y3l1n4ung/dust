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
    let order = packed(&["app", "Order", "lib/order.dart", "c:id", "f:Money"]);
    let money = packed(&["app", "Money", "lib/money.dart", "c:amount", "c:currency"]);
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
    let left = packed(&["app", "Left", "lib/left.dart", "c:left_id", "f:Right"]);
    let right = packed(&["app", "Right", "lib/right.dart", "c:right_id", "f:Left"]);
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
    let order = packed(&["app", "Order", "lib/order.dart", "c:id", "f:Absent"]);
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
    assert!(duplicate_row_types(&analysis, "app").is_empty());
}

#[test]
fn reports_a_row_class_name_two_libraries_share() {
    let inventory = packed(&["app", "Stock", "lib/inventory.dart", "c:item"]);
    let archive = packed(&["app", "Stock", "lib/archive.dart", "c:archived"]);
    let unique = packed(&["app", "Order", "lib/order.dart", "c:id"]);
    let analysis = analysis(&[
        (ROW_COLUMNS_KEY, &inventory),
        (ROW_COLUMNS_KEY, &archive),
        (ROW_COLUMNS_KEY, &unique),
    ]);

    assert_eq!(
        duplicate_row_types(&analysis, "app"),
        vec![(
            "Stock".to_owned(),
            vec![
                "lib/archive.dart".to_owned(),
                "lib/inventory.dart".to_owned()
            ],
        )]
    );
}

#[test]
fn one_row_class_declared_once_is_not_a_duplicate() {
    let order = packed(&["app", "Order", "lib/order.dart", "c:id"]);
    let theirs = packed(&["other", "Order", "lib/order.dart", "c:id"]);
    let analysis = analysis(&[(ROW_COLUMNS_KEY, &order), (ROW_COLUMNS_KEY, &theirs)]);

    assert!(duplicate_row_types(&analysis, "app").is_empty());
}
