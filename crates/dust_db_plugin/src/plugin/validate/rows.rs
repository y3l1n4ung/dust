use std::collections::{HashMap, HashSet};

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_ir::{ClassIr, FieldIr, TypeIr};

use crate::plugin::{
    model::{RowClass, SqlxConfig},
    parse::{effective_column_name, sqlx_config},
};

use super::types::{is_supported_scalar_type, render_type};

/// Validates all row mapper classes in a library.
pub(super) fn validate_rows(rows: &[RowClass<'_>], diagnostics: &mut Vec<Diagnostic>) {
    let row_by_name = rows
        .iter()
        .map(|row| (row.class.name.as_str(), row))
        .collect::<HashMap<_, _>>();
    let row_names = row_by_name.keys().copied().collect::<HashSet<_>>();
    for row in rows {
        validate_row(row, &row_by_name, &row_names, diagnostics);
    }
}

/// Builds the required SQL column set for each row class.
pub(super) fn row_column_map(rows: &[RowClass<'_>]) -> HashMap<String, HashSet<String>> {
    let row_by_name = rows
        .iter()
        .map(|row| (row.class.name.as_str(), row))
        .collect::<HashMap<_, _>>();
    rows.iter()
        .map(|row| {
            (
                row.class.name.clone(),
                collect_row_columns(row, &row_by_name).into_iter().collect(),
            )
        })
        .collect()
}

/// Validates one row class for duplicate and unsupported field mappings.
fn validate_row(
    row: &RowClass<'_>,
    row_by_name: &HashMap<&str, &RowClass<'_>>,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = HashMap::<String, &FieldIr>::new();
    for field in &row.class.fields {
        let config = sqlx_config(&field.configs);
        validate_field_shape(row.class, field, &config, row_names, diagnostics);
        if config.skip {
            continue;
        }
        if config.flatten {
            insert_flattened_columns(row.class, field, row_by_name, &mut seen, diagnostics);
            continue;
        }
        let column = effective_column_name(&row.config, &field.name, &config);
        if let Some(existing) = seen.insert(column.clone(), field) {
            push_duplicate_column(row.class, field, existing, &column, diagnostics);
        }
    }
}

/// Collects columns required by a row class, expanding flattened rows.
fn collect_row_columns(
    row: &RowClass<'_>,
    row_by_name: &HashMap<&str, &RowClass<'_>>,
) -> Vec<String> {
    let mut columns = Vec::new();
    for field in &row.class.fields {
        let config = sqlx_config(&field.configs);
        if config.skip {
            continue;
        }
        if config.flatten {
            if let Some(flattened) = field.ty.name().and_then(|name| row_by_name.get(name)) {
                columns.extend(collect_row_columns(flattened, row_by_name));
            }
            continue;
        }
        columns.push(effective_column_name(&row.config, &field.name, &config));
    }
    columns
}

/// Inserts columns from a flattened row and reports duplicates.
fn insert_flattened_columns<'a>(
    class: &ClassIr,
    field: &'a FieldIr,
    row_by_name: &HashMap<&str, &RowClass<'_>>,
    seen: &mut HashMap<String, &'a FieldIr>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(flattened) = field.ty.name().and_then(|name| row_by_name.get(name)) {
        for column in collect_row_columns(flattened, row_by_name) {
            if let Some(existing) = seen.insert(column.clone(), field) {
                push_duplicate_column(class, field, existing, &column, diagnostics);
            }
        }
    }
}

/// Validates one row field's mapping options and supported type.
fn validate_field_shape(
    class: &ClassIr,
    field: &FieldIr,
    config: &SqlxConfig,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if config.skip
        && !field.has_default
        && config.default_value_source.is_none()
        && !constructor_param_has_default(class, &field.name)
    {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{}` on `{}` uses `Sqlx(skip: true)` without a default",
                field.name, class.name
            ))
            .with_label(SourceLabel::new(
                field.span.file_id,
                field.span.range,
                "add a Dart field default or `Sqlx(defaultValue: ...)`",
            )),
        );
    }
    validate_flatten_shape(field, config, row_names, diagnostics);
    validate_conflicting_options(field, config, diagnostics);
    if !config.flatten
        && !config.json
        && config.try_from_source.is_none()
        && !is_supported_row_type(&field.ty)
    {
        diagnostics.push(error_on_field(
            field,
            format!(
                "unsupported SQLx row field type `{}`",
                render_type(&field.ty)
            ),
        ));
    }
}

/// Validates that `flatten` points at another row mapper type.
fn validate_flatten_shape(
    field: &FieldIr,
    config: &SqlxConfig,
    row_names: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !config.flatten {
        return;
    }
    let Some(name) = field.ty.name() else {
        diagnostics.push(error_on_field(
            field,
            "flattened SQLx field must use a named FromRow type",
        ));
        return;
    };
    if !row_names.contains(name) {
        diagnostics.push(error_on_field(
            field,
            format!(
                "flattened SQLx field `{}` must reference an @FromRow class",
                field.name
            ),
        ));
    }
}

/// Validates mutually exclusive row field options.
fn validate_conflicting_options(
    field: &FieldIr,
    config: &SqlxConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if config.json && config.flatten {
        diagnostics.push(error_on_field(
            field,
            "SQLx field cannot use both `json` and `flatten`",
        ));
    }
    if config.try_from_source.is_some() && config.flatten {
        diagnostics.push(error_on_field(
            field,
            "SQLx field cannot use both `tryFrom` and `flatten`",
        ));
    }
}

/// Returns true when a type can be read directly from a SQL row.
fn is_supported_row_type(ty: &TypeIr) -> bool {
    is_supported_scalar_type(ty) || ty.is_nullable() && is_supported_scalar_type(ty)
}

/// Reports duplicate SQL columns on a row class.
fn push_duplicate_column(
    class: &ClassIr,
    field: &FieldIr,
    existing: &FieldIr,
    column: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(
        Diagnostic::error(format!(
            "duplicate SQL column `{column}` on FromRow class `{}`",
            class.name
        ))
        .with_label(SourceLabel::new(
            field.span.file_id,
            field.span.range,
            format!("field `{}` maps to `{column}`", field.name),
        ))
        .with_label(SourceLabel::new(
            existing.span.file_id,
            existing.span.range,
            format!("field `{}` already maps to `{column}`", existing.name),
        )),
    );
}

/// Builds a row-field diagnostic with a common label.
fn error_on_field(field: &FieldIr, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(message.into()).with_label(SourceLabel::new(
        field.span.file_id,
        field.span.range,
        "unsupported SQLx row mapping",
    ))
}

/// Returns true when a field-formal constructor parameter has a default.
fn constructor_param_has_default(class: &ClassIr, field_name: &str) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor
            .params
            .iter()
            .any(|param| param.name == field_name && param.has_default)
    })
}
