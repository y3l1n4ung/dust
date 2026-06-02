use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, ConstructorIr, LibraryIr, ParamKind, TypeIr};

use crate::plugin::{
    model::{RowField, SqlxConfig},
    parse::{effective_column_name, sqlx_config},
};

use super::shared::{escape_dart_string, lower_first};

pub(super) fn render_from_row_extension(
    library: &LibraryIr,
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

#[cfg(test)]
mod tests {
    use dust_ir::{
        ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
        SpanIr, TypeIr,
    };
    use dust_text::{FileId, TextRange};

    use super::*;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    fn library(classes: Vec<ClassIr>) -> LibraryIr {
        LibraryIr {
            package_root: String::new(),
            package_name: String::new(),
            source_path: "user.dart".to_owned(),
            output_path: "user.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(),
            classes,
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }

    fn class(name: &str) -> ClassIr {
        ClassIr {
            kind: ClassKindIr::Class,
            name: name.to_owned(),
            is_abstract: false,
            is_interface: false,
            superclass_name: None,
            span: span(),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
            serde: None,
        }
    }

    #[test]
    fn emits_basic_from_row_extension() {
        let mut class = class("UserRow");
        class.fields.push(FieldIr {
            name: "id".to_owned(),
            ty: TypeIr::int(),
            span: span(),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        });
        class.constructors.push(ConstructorIr {
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
        });
        let library = library(vec![class.clone()]);

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

    #[test]
    fn emits_no_constructor_from_row_failure_body() {
        let mut class = class("NoCtorRow");
        class.fields.push(FieldIr {
            name: "id".to_owned(),
            ty: TypeIr::int(),
            span: span(),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        });
        let library = library(vec![class.clone()]);

        assert_eq!(
            render_from_row_extension(&library, &class, &Default::default()),
            r#"extension NoCtorRowFromRow on NoCtorRow {
  static NoCtorRow fromRow(Row row) {
    throw StateError('No usable constructor found for NoCtorRow');
  }
}"#
        );
    }
}
