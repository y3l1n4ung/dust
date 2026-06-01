use std::{collections::HashSet, fs, path::Path};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, ConstructorIr, LibraryIr, MethodIr, ParamKind, TypeIr};
use dust_plugin_api::PluginContribution;

use super::{
    DbPluginOptions,
    model::{DaoClass, DaoMethod, DatabaseClass, DbDriver, RowField},
    parse::{
        dao_classes, database_classes, effective_column_name, imported_row_names, row_classes,
        sqlx_config,
    },
};

pub(crate) fn emit_db_library(library: &LibraryIr, options: DbPluginOptions) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let mut sections = Vec::new();

    let rows = row_classes(library);
    if options.databases {
        for row in &rows {
            sections.push(render_from_row_extension(library, row.class, &row.config));
        }
        for db in database_classes(library) {
            sections.push(render_database_class(library, &db));
        }
        let imported_rows = imported_row_names(library);
        let row_names = rows
            .iter()
            .map(|row| row.class.name.as_str())
            .chain(imported_rows.iter().map(String::as_str))
            .collect::<HashSet<_>>();
        for dao in dao_classes(library) {
            sections.push(render_dao_class(&dao, &row_names));
        }
    } else {
        for row in &rows {
            sections.push(render_from_row_extension(library, row.class, &row.config));
        }
    }
    if !sections.is_empty() {
        contribution.support_types.push(sections.join("\n\n"));
    }

    contribution
}

fn render_dao_class(dao: &DaoClass<'_>, row_names: &HashSet<&str>) -> String {
    let class_name = &dao.class.name;
    let generated_name = dao
        .class
        .constructors
        .iter()
        .find_map(|constructor| constructor.redirected_target_name.as_deref())
        .filter(|target| target.starts_with("_$"))
        .map_or_else(|| format!("_${class_name}"), str::to_owned);
    let methods = dao
        .methods
        .iter()
        .map(|method| render_dao_method(method, row_names))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "final class {generated_name} implements {class_name} {{\n  const {generated_name}(this._db);\n\n  final SqlxDriver _db;\n\n{methods}\n}}"
    )
}

fn render_dao_method(method: &DaoMethod<'_>, row_names: &HashSet<&str>) -> String {
    let method_ir = method.method;
    let return_type = DYNAMIC_TYPES.render(&method_ir.return_type);
    let params = render_method_params(method_ir);
    let args = method_ir
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = render_sql_literal(&method.sql);
    let body = render_dao_method_body(method, row_names, &sql, &args);
    format!(
        "  @override\n  {return_type} {}({params}) {{\n{body}\n  }}",
        method_ir.name
    )
}

fn render_method_params(method: &MethodIr) -> String {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for param in &method.params {
        let rendered = format!("{} {}", DYNAMIC_TYPES.render(&param.ty), param.name);
        match param.kind {
            ParamKind::Positional => positional.push(rendered),
            ParamKind::Named => named.push(rendered),
        }
    }
    if named.is_empty() {
        positional.join(", ")
    } else {
        let mut parts = positional;
        parts.push(format!("{{{}}}", named.join(", ")));
        parts.join(", ")
    }
}

fn render_dao_method_body(
    method: &DaoMethod<'_>,
    row_names: &HashSet<&str>,
    sql: &str,
    args: &str,
) -> String {
    let Some(ok_type) = method.return_ok_type.as_ref() else {
        return format!(
            "    return Err<dynamic, SqlxError>(\n      SqlxError.decode('DAO method `{}` must return Future<Result<T, SqlxError>>.'),\n    );",
            method.method.name
        );
    };
    if ok_type.is_named("ExecResult") {
        return format!("    return _db.execute(\n      {sql},\n      [{args}],\n    );");
    }
    if ok_type.is_named("Unit") {
        return format!(
            "    return _db.execute(\n      {sql},\n      [{args}],\n    ).then(\n      (result) => result.andThen<Unit>((_) => const Ok<Unit, SqlxError>(unit)),\n    );"
        );
    }
    if is_scalar_type(ok_type) {
        return format!(
            "    return _db.fetchScalar<{}>(\n      {sql},\n      [{args}],\n    );",
            DYNAMIC_TYPES.render(ok_type)
        );
    }
    if ok_type.is_named("List") {
        let Some(item) = ok_type.args().first() else {
            return format!("    return _db.raw.fetch(\n      {sql},\n      [{args}],\n    );");
        };
        if item.is_named("Row") {
            return format!("    return _db.raw.fetch(\n      {sql},\n      [{args}],\n    );");
        }
        let item_name = item.name().unwrap_or("Object");
        if row_names.contains(item_name) {
            return format!(
                "    return _db.fetchAll<{item_name}>(\n      {sql},\n      [{args}],\n      {item_name}FromRow.fromRow,\n    );"
            );
        }
        return format!(
            "    return Err<{}, SqlxError>(\n      SqlxError.decode('Unsupported DAO list item type.'),\n    );",
            DYNAMIC_TYPES.render(ok_type)
        );
    }
    let Some(row_name) = ok_type.name() else {
        return format!(
            "      return Err<{}, SqlxError>(\n        SqlxError.decode('Unsupported DAO return type.'),\n      );",
            DYNAMIC_TYPES.render(ok_type)
        );
    };
    if ok_type.is_nullable() {
        return format!(
            "    return _db.fetchOptional<{row_name}>(\n      {sql},\n      [{args}],\n      {row_name}FromRow.fromRow,\n    );"
        );
    }
    format!(
        "    return _db.fetchOne<{row_name}>(\n      {sql},\n      [{args}],\n      {row_name}FromRow.fromRow,\n    );"
    )
}

fn render_database_class(library: &LibraryIr, db: &DatabaseClass<'_>) -> String {
    let class_name = &db.class.name;
    let generated_name = format!("_${class_name}");
    let migrations_name = format!("_${}Migrations", lower_first(class_name));
    let open_expr = match db.driver {
        DbDriver::Sqlite3 => {
            format!("Sqlite3Driver.open(\n      path,\n      migrations: {migrations_name},\n    )")
        }
        DbDriver::Postgres => {
            "throw UnsupportedError('Driver.postgres is not supported in Dust DB v1')".to_owned()
        }
    };
    let migrations = render_migrations_map(library, &db.migrations, &migrations_name);
    format!(
        "final class {generated_name} implements {class_name} {{\n  {generated_name}._(this.pool);\n\n  factory {generated_name}.open(String path) {{\n    final pool = {open_expr};\n    return {generated_name}._(pool);\n  }}\n\n  @override\n  final Pool pool;\n}}\n\n{migrations}"
    )
}

fn render_migrations_map(library: &LibraryIr, migrations: &str, name: &str) -> String {
    let path = Path::new(&library.package_root).join(migrations);
    let mut files = fs::read_dir(&path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    files.sort();

    let entries = files
        .iter()
        .filter_map(|file| {
            let source = fs::read_to_string(file).ok()?;
            let key = file.file_name()?.to_str()?;
            Some(format!(
                "  '{}': '{}',",
                escape_dart_string(key),
                escape_dart_string(&source)
            ))
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return format!("const Map<String, String> {name} = <String, String>{{}};");
    }
    format!(
        "const Map<String, String> {name} = <String, String>{{\n{}\n}};",
        entries.join("\n")
    )
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
            "extension {0}FromRow on {0} {{\n  static {0} fromRow(Row row) {{\n    throw StateError('No usable constructor found for {0}');\n  }}\n}}",
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
    let registration = format!(
        "final bool {} = registerRowMapper<{}>({}FromRow.fromRow);",
        row_registration_name(&class.name),
        class.name,
        class.name
    );
    format!(
        "extension {0}FromRow on {0} {{\n  static {0} fromRow(Row row) {{\n    return {call};\n  }}\n}}\n\n{registration}",
        class.name,
    )
}

fn row_registration_name(class_name: &str) -> String {
    format!("_${}FromRowRegistered", lower_first(class_name))
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
    let column = escape_dart_string(&field.column);
    let decoded = if field.config.json {
        let ty = DYNAMIC_TYPES.render_non_nullable(&field.field.ty);
        format!("{ty}.fromJson(decodeJsonObject(row.read<String>('{column}')))")
    } else if let Some(try_from) = &field.config.try_from_source {
        let value = try_from_decode_type(library, try_from)
            .filter(|ty| ty != "Object?" && ty != "dynamic")
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
            format!("row.readOrNull<Object?>('{column}') == null ? {default} : {decoded}")
        }
        None => decoded,
    }
}

fn render_builtin_decode(ty: &TypeIr, column: &str) -> String {
    let nullable = ty.is_nullable();
    match ty.name() {
        Some("bool") if nullable => format!("row.readBoolOrNull('{column}')"),
        Some("bool") => format!("row.readBool('{column}')"),
        Some("DateTime") if nullable => format!("row.readDateTimeOrNull('{column}')"),
        Some("DateTime") => format!("row.readDateTime('{column}')"),
        Some(name) if nullable => format!("row.readOrNull<{name}>('{column}')"),
        Some(name) => format!("row.read<{name}>('{column}')"),
        None => format!("row.read<Object?>('{column}')"),
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
        format!("{name}(\n      {},\n    )", args.join(",\n      "))
    }
}

fn find_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        chars.collect::<String>()
    )
}

fn escape_dart_string(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn render_sql_literal(source: &str) -> String {
    if !source.contains("'''") {
        return format!("r'''{source}'''");
    }
    format!("'{}'", escape_dart_string(source))
}

fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
    )
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
            r#"extension UserRowFromRow on UserRow {
  static UserRow fromRow(Row row) {
    return UserRow(id: row.read<int>('id'));
  }
}

final bool _$userRowFromRowRegistered = registerRowMapper<UserRow>(UserRowFromRow.fromRow);"#
        );
    }
}
