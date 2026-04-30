use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsSnakeCase, AsTrainCase};

use dust_ir::{SerdeRenameRuleIr, TypeIr};

pub(crate) fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{}{}", kind.as_str(), nullable)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } => {
            let args = if args.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    args.iter().map(render_type).collect::<Vec<_>>().join(", ")
                )
            };
            let nullable = if *nullable { "?" } else { "" };
            format!("{name}{args}{nullable}")
        }
        TypeIr::Function {
            signature,
            nullable,
        } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{signature}{nullable}")
        }
        TypeIr::Record { shape, nullable } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{shape}{nullable}")
        }
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "Object?".to_owned(),
    }
}

pub(crate) fn non_nullable(ty: &TypeIr) -> TypeIr {
    match ty {
        TypeIr::Builtin { kind, .. } => TypeIr::builtin(*kind),
        TypeIr::Named { name, args, .. } => TypeIr::generic(name.as_ref(), args.to_vec()),
        TypeIr::Function { signature, .. } => TypeIr::function(signature.as_ref()),
        TypeIr::Record { shape, .. } => TypeIr::record(shape.as_ref()),
        TypeIr::Dynamic => TypeIr::dynamic(),
        TypeIr::Unknown => TypeIr::unknown(),
    }
}

pub(crate) fn non_null_encode_expr(expr: &str) -> String {
    format!("({expr}!)")
}

pub(crate) fn access_receiver(source: &str) -> String {
    if is_simple_receiver(source) {
        source.to_owned()
    } else {
        format!("({source})")
    }
}

pub(crate) fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    match rule {
        SerdeRenameRuleIr::LowerCase => source.to_lowercase(),
        SerdeRenameRuleIr::UpperCase => source.to_uppercase(),
        SerdeRenameRuleIr::PascalCase => AsPascalCase(source).to_string(),
        SerdeRenameRuleIr::CamelCase => AsLowerCamelCase(source).to_string(),
        SerdeRenameRuleIr::SnakeCase => AsSnakeCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingSnakeCase => AsSnakeCase(source).to_string().to_uppercase(),
        SerdeRenameRuleIr::KebabCase => AsKebabCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingKebabCase => AsTrainCase(source).to_string().to_uppercase(),
    }
}

fn is_simple_receiver(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
}
