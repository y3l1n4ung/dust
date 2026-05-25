use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, ConstructorIr, LibraryIr, ParamKind, TypeIr};
use dust_plugin_api::PluginContribution;

use super::{
    model::{FetchKind, QuerySpec, RowField},
    parse::{dust_db_classes, effective_column_name, row_classes, sqlx_config},
};

pub(crate) fn emit_db_library(library: &LibraryIr) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let mut sections = Vec::new();

    for row in row_classes(library) {
        sections.push(render_from_row_extension(library, row.class, &row.config));
    }
    for db in dust_db_classes(library) {
        sections.push(render_repository_class(db.class, &db.queries));
    }
    if !sections.is_empty() {
        contribution.support_types.push(sections.join("\n\n"));
    }

    contribution
}

fn render_repository_class(class: &ClassIr, queries: &[QuerySpec<'_>]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "final class _${} implements {} {{\n",
        class.name, class.name
    ));
    out.push_str("  final dynamic _db;\n\n");
    out.push_str(&format!("  _${}(this._db);\n", class.name));
    for query in queries {
        out.push('\n');
        out.push_str(&render_query_method(query));
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn render_query_method(query: &QuerySpec<'_>) -> String {
    let method = query.method;
    let params = method
        .params
        .iter()
        .map(|param| {
            let ty = DYNAMIC_TYPES.render(&param.ty);
            match param.kind {
                ParamKind::Named => format!("required {ty} {}", param.name),
                ParamKind::Positional => format!("{ty} {}", param.name),
            }
        })
        .collect::<Vec<_>>();
    let signature_params = if method
        .params
        .iter()
        .any(|param| matches!(param.kind, ParamKind::Named))
    {
        format!("{{{}}}", params.join(", "))
    } else {
        params.join(", ")
    };
    let return_type = DYNAMIC_TYPES.render(&method.return_type);
    let args = render_args(&query.args);
    let sql = escape_dart_string(&query.sql);
    let body = render_query_body(query, &sql, &args);

    format!(
        "  @override\n  {return_type} {name}({signature_params}) async {{\n{body}\n  }}",
        name = method.name,
    )
}

fn render_query_body(query: &QuerySpec<'_>, sql: &str, args: &str) -> String {
    if query.transaction {
        let inner = render_query_body_inner(query, "txn", sql, args)
            .lines()
            .map(|line| format!("      {}", line.strip_prefix("    ").unwrap_or(line)))
            .collect::<Vec<_>>()
            .join("\n");
        return format!("    return _db.transaction((txn) async {{\n{inner}\n    }});");
    }
    render_query_body_inner(query, "_db", sql, args)
}

fn render_query_body_inner(query: &QuerySpec<'_>, db: &str, sql: &str, args: &str) -> String {
    match query.fetch {
        FetchKind::One => {
            let item_type = future_item_type(&query.method.return_type).unwrap_or("Object");
            format!(
                "    final rows = await {db}.rawQuery(\n      '{sql}',\n      {args},\n    );\n    if (rows.isEmpty) return null;\n    return {item_type}FromRow.fromRow(rows.first);"
            )
        }
        FetchKind::All => {
            let item_type = list_future_item_type(&query.method.return_type).unwrap_or("Object");
            format!(
                "    final rows = await {db}.rawQuery(\n      '{sql}',\n      {args},\n    );\n    return rows.map({item_type}FromRow.fromRow).toList();"
            )
        }
        FetchKind::Scalar => {
            let scalar_type = future_item_type(&query.method.return_type).unwrap_or("Object");
            format!(
                "    final rows = await {db}.rawQuery(\n      '{sql}',\n      {args},\n    );\n    return rows.first.values.first as {scalar_type};"
            )
        }
        FetchKind::InsertOne => {
            let item_type = future_item_type(&query.method.return_type).unwrap_or("Object");
            format!(
                "    final id = await {db}.rawInsert(\n      '{sql}',\n      {args},\n    );\n    final rows = await {db}.rawQuery(\n      'SELECT * FROM {table} WHERE id = ?',\n      <Object?>[id],\n    );\n    return {item_type}FromRow.fromRow(rows.first);",
                table = infer_insert_table(&query.sql).unwrap_or("row")
            )
        }
        FetchKind::Execute => {
            format!("    await {db}.execute(\n      '{sql}',\n      {args},\n    );")
        }
        FetchKind::Stream => {
            let item_type = stream_item_type(&query.method.return_type).unwrap_or("Object");
            format!(
                "    return Stream.fromFuture(\n      {db}.rawQuery(\n        '{sql}',\n        {args},\n      ),\n    ).asyncExpand(\n      (rows) => Stream.fromIterable(rows.map({item_type}FromRow.fromRow)),\n    );"
            )
        }
    }
}

fn render_from_row_extension(
    library: &LibraryIr,
    class: &ClassIr,
    class_config: &super::model::SqlxConfig,
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
        return format!(
            "extension {0}FromRow on {0} {{\n  static {0} fromRow(Map<String, Object?> row) => throw StateError('No usable constructor found for {0}');\n}}",
            class.name
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
    format!(
        "extension {0}FromRow on {0} {{\n  static {0} fromRow(Map<String, Object?> row) => {call};\n}}",
        class.name,
    )
}

fn render_row_value(
    library: &LibraryIr,
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
    let raw = format!("row['{}']", escape_dart_string(&field.column));
    let decoded = if field.config.json {
        let ty = DYNAMIC_TYPES.render_non_nullable(&field.field.ty);
        format!("{ty}.fromJson(\n      jsonDecode({raw} as String) as Map<String, Object?>,\n    )")
    } else if let Some(try_from) = &field.config.try_from_source {
        let value = try_from_decode_type(library, try_from)
            .filter(|ty| ty != "Object?" && ty != "dynamic")
            .map_or_else(|| raw.clone(), |ty| format!("{raw} as {ty}"));
        format!("{try_from}.decode({value})")
    } else {
        render_builtin_decode(&field.field.ty, &raw)
    };
    match default_source {
        Some(default) => format!(
            "row.containsKey('{}') ? {decoded} : {default}",
            escape_dart_string(&field.column)
        ),
        None => decoded,
    }
}

fn render_builtin_decode(ty: &TypeIr, raw: &str) -> String {
    let nullable = ty.is_nullable();
    match ty.name() {
        Some("bool") if nullable => format!("{raw} == null ? null : ({raw} as int) == 1"),
        Some("bool") => format!("({raw} as int) == 1"),
        Some("DateTime") if nullable => {
            format!("{raw} == null ? null : DateTime.parse({raw} as String)")
        }
        Some("DateTime") => format!("DateTime.parse({raw} as String)"),
        Some(name @ ("int" | "double" | "num" | "String")) => {
            let suffix = if nullable { "?" } else { "" };
            format!("{raw} as {name}{suffix}")
        }
        Some(name) => {
            let suffix = if nullable { "?" } else { "" };
            format!("{raw} as {name}{suffix}")
        }
        None => raw.to_owned(),
    }
}

fn try_from_decode_type(library: &LibraryIr, source: &str) -> Option<String> {
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

fn infer_try_from_type_suffix(converter_name: &str) -> Option<String> {
    [
        ("FromInt", "int"),
        ("FromString", "String"),
        ("FromBool", "bool"),
        ("FromDouble", "double"),
        ("FromNum", "num"),
    ]
    .into_iter()
    .find_map(|(suffix, ty)| converter_name.ends_with(suffix).then(|| ty.to_owned()))
}

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
        format!("{name}(\n    {},\n  )", args.join(",\n    "))
    }
}

fn find_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

fn render_args(args: &[String]) -> String {
    if args.is_empty() {
        "const <Object?>[]".to_owned()
    } else {
        format!("<Object?>[{}]", args.join(", "))
    }
}

fn future_item_type(ty: &TypeIr) -> Option<&str> {
    if ty.is_named("Future") && ty.args().len() == 1 {
        return ty.args()[0].name();
    }
    None
}

fn list_future_item_type(ty: &TypeIr) -> Option<&str> {
    if ty.is_named("Future") && ty.args().len() == 1 {
        let list = &ty.args()[0];
        if list.is_named("List") && list.args().len() == 1 {
            return list.args()[0].name();
        }
    }
    None
}

fn stream_item_type(ty: &TypeIr) -> Option<&str> {
    if ty.is_named("Stream") && ty.args().len() == 1 {
        return ty.args()[0].name();
    }
    None
}

fn infer_insert_table(sql: &str) -> Option<&str> {
    let mut words = sql.split_whitespace();
    if !words.next()?.eq_ignore_ascii_case("insert") {
        return None;
    }
    if !words.next()?.eq_ignore_ascii_case("into") {
        return None;
    }
    words
        .next()
        .map(|table| table.trim_matches(|ch: char| ch == '`' || ch == '"'))
}

fn escape_dart_string(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
}

#[cfg(test)]
mod tests {
    use dust_ir::{
        ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, ParamKind, SpanIr, TypeIr,
    };
    use dust_text::{FileId, TextRange};

    use super::*;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    #[test]
    fn emits_basic_from_row_extension() {
        let class = ClassIr {
            kind: ClassKindIr::Class,
            name: "UserRow".to_owned(),
            is_abstract: false,
            is_interface: false,
            superclass_name: None,
            span: span(),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::int(),
                span: span(),
                has_default: false,
                serde: None,
                configs: Vec::new(),
            }],
            constructors: vec![ConstructorIr {
                name: None,
                is_factory: false,
                redirected_target_source: None,
                redirected_target_name: None,
                span: span(),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::int(),
                    span: span(),
                    kind: ParamKind::Named,
                    has_default: false,
                    default_value_source: None,
                }],
            }],
            methods: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
            serde: None,
        };
        let library = LibraryIr {
            package_root: String::new(),
            package_name: String::new(),
            source_path: "user.dart".to_owned(),
            output_path: "user.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(),
            classes: vec![class.clone()],
            enums: Vec::new(),
        };

        assert_eq!(
            render_from_row_extension(&library, &class, &Default::default()),
            "extension UserRowFromRow on UserRow {\n  static UserRow fromRow(Map<String, Object?> row) => UserRow(id: row['id'] as int);\n}"
        );
    }
}
