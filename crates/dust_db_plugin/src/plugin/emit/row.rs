use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_DYNAMIC, DART_INT, DART_NUM, DART_OBJECT_NULLABLE,
    DART_STRING, DYNAMIC_TYPES, render_template,
};
use dust_ir::{ClassIr, ConstructorIr, DartFileIr, ParamKind, TypeIr};
use serde::Serialize;

use crate::plugin::{
    model::{RowField, SqlxConfig},
    parse::{effective_column_name, sqlx_config},
};

use super::shared::{escape_dart_string, lower_first};

/// Template context for a generated `FromRow` extension.
#[derive(Serialize)]
struct FromRowContext<'a> {
    /// Source row class name.
    class_name: &'a str,
    /// Rendered constructor call for decoded fields.
    call: String,
    /// Rendered row mapper registration statement.
    registration: String,
}

/// Renders generated `FromRow` support for a row class.
pub(super) fn render_from_row_extension(
    library: &DartFileIr,
    class: &ClassIr,
    class_config: &SqlxConfig,
) -> String {
    let fields = class
        .fields
        .iter()
        .map(|field| {
            let config = sqlx_config(&field.configs);
            let column = effective_column_name(class_config, &field.name, &config);
            RowField {
                field,
                config,
                column,
            }
        })
        .collect::<Vec<_>>();
    let Some(constructor) = find_constructor(class) else {
        return render_template(
            "from_row_missing_constructor",
            include_str!("templates/from_row_missing_constructor.jinja"),
            FromRowContext {
                class_name: &class.name,
                call: String::new(),
                registration: String::new(),
            },
        );
    };
    let args = constructor
        .params
        .iter()
        .filter_map(|param| {
            let field = fields.iter().find(|field| field.field.name == param.name)?;
            if field.config.skip && param.has_default && field.config.default_value_source.is_none()
            {
                return None;
            }
            let default_source = field
                .config
                .default_value_source
                .as_deref()
                .or(param.default_value_source.as_deref());
            let value = render_row_value(library, field, default_source);
            Some(match param.kind {
                ParamKind::Named => format!("{}: {value}", param.name),
                ParamKind::Positional => value,
            })
        })
        .collect::<Vec<_>>();
    let call = render_constructor_call(&class.name, constructor, &args);
    let registration = format!(
        "final bool {} = registerRowMapper<{}>({}FromRow.fromRow);",
        row_registration_name(&class.name),
        class.name,
        class.name
    );

    render_template(
        "from_row_extension",
        include_str!("templates/from_row_extension.jinja"),
        FromRowContext {
            class_name: &class.name,
            call,
            registration,
        },
    )
}

/// Returns the generated top-level row mapper registration variable name.
fn row_registration_name(class_name: &str) -> String {
    format!("_${}FromRowRegistered", lower_first(class_name))
}

/// Renders the Dart expression that reads one field from a row.
fn render_row_value(
    library: &DartFileIr,
    field: &RowField<'_>,
    default_source: Option<&str>,
) -> String {
    if field.config.skip {
        return default_source.unwrap_or("null").to_owned();
    }
    if field.config.flatten {
        let ty = DYNAMIC_TYPES.render_non_nullable(&field.field.ty);
        return format!("{ty}FromRow.fromRow(row)");
    }
    let column = escape_dart_string(&field.column);
    let decoded = if field.config.json {
        let ty = DYNAMIC_TYPES.render_non_nullable(&field.field.ty);
        format!("{ty}.fromJson(decodeJsonObject(row.read<String>('{column}')))")
    } else if let Some(try_from) = &field.config.try_from_source {
        let value = try_from_decode_type(library, try_from)
            .filter(|ty| ty != DART_OBJECT_NULLABLE && ty != DART_DYNAMIC)
            .map_or_else(
                || format!("row.read<Object?>('{column}')"),
                |ty| format!("row.read<{ty}>('{column}')"),
            );
        format!("{try_from}.decode({value})")
    } else {
        render_builtin_decode(&field.field.ty, &column)
    };
    match default_source {
        Some(default) => {
            format!("row.readNullable<Object?>('{column}') == null ? {default} : {decoded}")
        }
        None => decoded,
    }
}

/// Renders builtin row-read calls for directly supported Dart types.
fn render_builtin_decode(ty: &TypeIr, column: &str) -> String {
    let nullable = ty.is_nullable();
    match ty.name() {
        Some(DART_BOOL) if nullable => format!("row.readBoolNullable('{column}')"),
        Some(DART_BOOL) => format!("row.readBool('{column}')"),
        Some(DART_DATE_TIME) if nullable => format!("row.readDateTimeNullable('{column}')"),
        Some(DART_DATE_TIME) => format!("row.readDateTime('{column}')"),
        Some(name) if nullable => format!("row.readNullable<{name}>('{column}')"),
        Some(name) => format!("row.read<{name}>('{column}')"),
        None => format!("row.read<Object?>('{column}')"),
    }
}

/// Resolves the input type accepted by a `tryFrom` converter.
fn try_from_decode_type(library: &DartFileIr, source: &str) -> Option<String> {
    let converter_name = try_from_converter_name(source)?;
    library
        .classes
        .iter()
        .find(|class| class.name == converter_name)
        .and_then(|converter| {
            converter
                .methods
                .iter()
                .find(|method| method.name == "decode")
        })
        .and_then(|decode| decode.params.first())
        .map(|param| DYNAMIC_TYPES.render(&param.ty))
        .or_else(|| infer_try_from_type_suffix(converter_name))
}

/// Extracts the converter class name from a `tryFrom` expression.
fn try_from_converter_name(source: &str) -> Option<&str> {
    let source = source
        .trim()
        .strip_prefix("const ")
        .unwrap_or(source.trim());
    let before_args = source
        .split_once('(')
        .map_or(source, |(name, _)| name)
        .trim();
    before_args
        .rsplit('.')
        .next()
        .filter(|name| !name.is_empty())
}

/// Infers a converter input type from common converter name suffixes.
fn infer_try_from_type_suffix(converter_name: &str) -> Option<String> {
    [
        ("FromInt", DART_INT),
        ("FromString", DART_STRING),
        ("FromBool", DART_BOOL),
        ("FromDouble", DART_DOUBLE),
        ("FromNum", DART_NUM),
    ]
    .into_iter()
    .find_map(|(suffix, ty)| converter_name.ends_with(suffix).then(|| ty.to_owned()))
}

/// Renders a row constructor call, wrapping long argument lists.
fn render_constructor_call(
    class_name: &str,
    constructor: &ConstructorIr,
    args: &[String],
) -> String {
    let name = constructor.name.as_ref().map_or_else(
        || class_name.to_owned(),
        |name| format!("{class_name}.{name}"),
    );
    if args.is_empty() {
        format!("{name}()")
    } else if args.join(", ").len() + name.len() <= 56 {
        format!("{name}({})", args.join(", "))
    } else {
        format!("{name}(\n      {},\n    )", args.join(",\n      "))
    }
}

/// Finds a constructor that can initialize every row field.
fn find_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

#[cfg(test)]
/// Unit tests for generated row mapping output.
mod tests;
