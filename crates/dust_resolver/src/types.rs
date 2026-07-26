use dust_ir::{BuiltinType, LoweringOutcome, TypeIr};
use dust_parser_dart::{ParsedTypeKind, ParsedTypeSurface};
use dust_text::TextRange;

/// Lowers parser-owned Dart type facts, or a raw fallback source, into `TypeIr`.
pub fn lower_type_ir(
    parsed: Option<&ParsedTypeSurface>,
    source: Option<&str>,
) -> LoweringOutcome<TypeIr> {
    if let Some(parsed) = parsed {
        return LoweringOutcome::new(type_from_parsed_surface(parsed));
    }

    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return LoweringOutcome::new(TypeIr::unknown());
    };

    let span = TextRange::new(0_u32, source.len() as u32);
    let parsed = ParsedTypeSurface::parse(source, span);
    LoweringOutcome::new(
        parsed
            .as_ref()
            .map(type_from_parsed_surface)
            .unwrap_or_else(TypeIr::unknown),
    )
}

/// Converts structured parser type output into type IR.
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

/// Returns the source form without a trailing nullable marker.
fn non_nullable_source(parsed: &ParsedTypeSurface) -> &str {
    parsed.source.strip_suffix('?').unwrap_or(&parsed.source)
}

/// Parses supported builtin Dart type names.
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
