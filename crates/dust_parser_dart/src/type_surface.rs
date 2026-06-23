use dust_dart_syntax::{find_top_level_char, has_top_level_char, split_top_level_items};
use dust_text::TextRange;

/// A parser-owned normalized Dart type surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTypeSurface {
    /// The exact type source preserved from the parsed Dart syntax.
    pub source: String,
    /// The source span for the type expression.
    pub span: TextRange,
    /// The normalized type shape.
    pub kind: ParsedTypeKind,
    /// Whether the type source has a trailing nullable marker.
    pub nullable: bool,
}

/// Parser-owned Dart type shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedTypeKind {
    /// A common built-in type name.
    Builtin(String),
    /// A named type with optional generic arguments.
    Named {
        /// The base type name.
        name: String,
        /// Generic type arguments.
        args: Vec<ParsedTypeSurface>,
    },
    /// A function type preserved as source.
    Function,
    /// A record type preserved as source.
    Record,
    /// The explicit `dynamic` type.
    Dynamic,
    /// A shape the parser could not normalize.
    Unknown,
}

impl ParsedTypeSurface {
    /// Parses one Dart type source into parser-owned type facts.
    pub fn parse(source: impl Into<String>, span: TextRange) -> Option<Self> {
        let source = source.into();
        let source = source.trim();
        if source.is_empty() {
            return None;
        }

        let (base, nullable) = strip_nullable(source);
        let kind = parse_type_kind(base.trim(), span);
        Some(Self {
            source: source.to_owned(),
            span,
            kind,
            nullable,
        })
    }
}

/// Parses the non-nullable base type shape.
fn parse_type_kind(source: &str, span: TextRange) -> ParsedTypeKind {
    if source == "dynamic" {
        return ParsedTypeKind::Dynamic;
    }
    if looks_like_function_type(source) {
        return ParsedTypeKind::Function;
    }
    if looks_like_record_type(source) {
        return ParsedTypeKind::Record;
    }
    if is_builtin(source) {
        return ParsedTypeKind::Builtin(source.to_owned());
    }
    if let Some((name, args)) = split_generic(source) {
        return ParsedTypeKind::Named {
            name: name.trim().to_owned(),
            args: split_top_level_items(args)
                .into_iter()
                .filter_map(|arg| ParsedTypeSurface::parse(arg, span))
                .collect(),
        };
    }
    if source.is_empty() {
        ParsedTypeKind::Unknown
    } else {
        ParsedTypeKind::Named {
            name: source.to_owned(),
            args: Vec::new(),
        }
    }
}

/// Splits a trailing nullable marker from a type source.
fn strip_nullable(source: &str) -> (&str, bool) {
    if let Some(stripped) = source.strip_suffix('?') {
        (stripped, true)
    } else {
        (source, false)
    }
}

/// Returns whether the type name is a built-in Dart scalar.
fn is_builtin(source: &str) -> bool {
    matches!(
        source,
        "String" | "int" | "bool" | "double" | "num" | "Object"
    )
}

/// Splits a generic type into base name and argument source.
fn split_generic(source: &str) -> Option<(&str, &str)> {
    let start = source.find('<')?;
    let end = source.rfind('>')?;
    if end <= start {
        return None;
    }
    Some((&source[..start], &source[start + 1..end]))
}

/// Returns whether the source looks like a Dart record type.
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

/// Returns whether the source looks like a Dart function type.
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

#[cfg(test)]
mod tests {
    use super::{ParsedTypeKind, ParsedTypeSurface};
    use dust_text::TextRange;

    fn parsed(source: &str) -> ParsedTypeSurface {
        ParsedTypeSurface::parse(source, TextRange::new(0_u32, source.len() as u32)).unwrap()
    }

    #[test]
    fn parses_named_generic_and_nullable_types() {
        let ty = parsed("Map<String, List<int?>>");
        let ParsedTypeKind::Named { name, args } = ty.kind else {
            panic!("expected named type");
        };

        assert_eq!(name, "Map");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].kind, ParsedTypeKind::Builtin("String".to_owned()));
        assert!(args[1].source.ends_with(">"));
    }

    #[test]
    fn keeps_function_and_record_shapes_distinct() {
        assert_eq!(
            parsed("void Function(String)?").kind,
            ParsedTypeKind::Function
        );
        assert!(parsed("void Function(String)?").nullable);
        assert_eq!(
            parsed("({String name, int age})").kind,
            ParsedTypeKind::Record
        );
    }
}
