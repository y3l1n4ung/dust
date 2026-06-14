use dust_ir::{BuiltinType, LoweringOutcome, TypeIr};
use dust_parser_dart::{ParsedTypeKind, ParsedTypeSurface};

use super::parse_support::{find_top_level_char, has_top_level_char};
pub(crate) use dust_dart_syntax::split_top_level_items as split_top_level_args;

pub(crate) fn lower_type(
    parsed: Option<&ParsedTypeSurface>,
    source: Option<&str>,
) -> LoweringOutcome<TypeIr> {
    if let Some(parsed) = parsed {
        return LoweringOutcome::new(type_from_parsed_surface(parsed));
    }

    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return LoweringOutcome::new(TypeIr::unknown());
    };

    LoweringOutcome::new(parse_type(source))
}

fn type_from_parsed_surface(parsed: &ParsedTypeSurface) -> TypeIr {
    let ty = match &parsed.kind {
        ParsedTypeKind::Builtin(name) => parse_builtin(name)
            .map(TypeIr::builtin)
            .unwrap_or_else(|| TypeIr::named(name.as_str())),
        ParsedTypeKind::Named { name, args } => {
            let args = args
                .iter()
                .map(type_from_parsed_surface)
                .collect::<Vec<_>>();
            if args.is_empty() {
                TypeIr::named(name.as_str())
            } else {
                TypeIr::generic(name.as_str(), args)
            }
        }
        ParsedTypeKind::Function => TypeIr::function(non_nullable_source(parsed)),
        ParsedTypeKind::Record => TypeIr::record(non_nullable_source(parsed)),
        ParsedTypeKind::Dynamic => TypeIr::dynamic(),
        ParsedTypeKind::Unknown => return TypeIr::unknown(),
    };

    if parsed.nullable { ty.nullable() } else { ty }
}

fn non_nullable_source(parsed: &ParsedTypeSurface) -> &str {
    parsed.source.strip_suffix('?').unwrap_or(&parsed.source)
}

pub(crate) fn parse_type(source: &str) -> TypeIr {
    let (base, nullable) = strip_nullable(source);
    let base = base.trim();

    let type_ir = if base == "dynamic" {
        TypeIr::dynamic()
    } else if looks_like_function_type(base) {
        TypeIr::function(base)
    } else if looks_like_record_type(base) {
        TypeIr::record(base)
    } else if let Some(builtin) = parse_builtin(base) {
        TypeIr::builtin(builtin)
    } else if let Some((name, args)) = split_generic(base) {
        TypeIr::generic(
            name.trim(),
            split_top_level_args(args)
                .into_iter()
                .map(|arg| parse_type(arg.trim()))
                .collect(),
        )
    } else {
        TypeIr::named(base)
    };

    if nullable {
        type_ir.nullable()
    } else {
        type_ir
    }
}

fn strip_nullable(source: &str) -> (&str, bool) {
    if let Some(stripped) = source.strip_suffix('?') {
        (stripped, true)
    } else {
        (source, false)
    }
}

fn parse_builtin(source: &str) -> Option<BuiltinType> {
    match source {
        "String" => Some(BuiltinType::String),
        "int" => Some(BuiltinType::Int),
        "bool" => Some(BuiltinType::Bool),
        "double" => Some(BuiltinType::Double),
        "num" => Some(BuiltinType::Num),
        "Object" => Some(BuiltinType::Object),
        _ => None,
    }
}

fn split_generic(source: &str) -> Option<(&str, &str)> {
    let start = source.find('<')?;
    let end = source.rfind('>')?;
    if end <= start {
        return None;
    }
    Some((&source[..start], &source[start + 1..end]))
}

fn looks_like_record_type(source: &str) -> bool {
    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        return false;
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return false;
    }

    inner.starts_with('{') || has_top_level_char(inner, ',')
}

fn looks_like_function_type(source: &str) -> bool {
    find_top_level_char(source, |index, ch| {
        if ch != 'F' || index == 0 {
            return false;
        }

        let tail = &source[index..];
        let Some(stripped) = tail.strip_prefix("Function") else {
            return false;
        };

        let prev = source[..index].chars().next_back().unwrap_or_default();
        let after = stripped.trim_start();
        prev.is_whitespace() && after.starts_with('(')
    })
    .is_some()
}
