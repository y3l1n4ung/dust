use dust_dart_syntax::{
    normalized_args, parse_member_ref, parse_string_literal, split_top_level_items,
    split_top_level_once,
};

/// A structured constant annotation value.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationValue {
    /// A boolean literal.
    Bool(bool),
    /// A string literal with raw/quoted delimiters removed.
    String(String),
    /// A numeric literal kept as source for exact int/double interpretation.
    Number(String),
    /// A list literal.
    List(Vec<AnnotationValue>),
    /// A named record literal.
    Record(Vec<(String, AnnotationValue)>),
    /// A constant constructor invocation with named arguments.
    Constructor {
        /// Constructor/type name.
        name: String,
        /// Named argument values.
        named: Vec<(String, AnnotationValue)>,
    },
    /// A member, type, or function reference expression.
    Member(String),
    /// Any expression shape Dust should preserve but not semantically parse.
    Expression(String),
}

/// Parses top-level named annotation arguments into structured values.
pub fn parse_annotation_named_values(source: &str) -> Option<Vec<(String, AnnotationValue)>> {
    let inner = normalized_args(source)?;
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect()
}

fn parse_annotation_value(source: &str) -> AnnotationValue {
    let source = source.trim();
    if source == "true" {
        return AnnotationValue::Bool(true);
    }
    if source == "false" {
        return AnnotationValue::Bool(false);
    }
    if let Some(value) = parse_string_literal(source) {
        return AnnotationValue::String(value);
    }
    if is_number_literal(source) {
        return AnnotationValue::Number(source.to_owned());
    }
    if let Some(list) = parse_list(source) {
        return AnnotationValue::List(list);
    }
    if let Some(record) = parse_record(source) {
        return AnnotationValue::Record(record);
    }
    if let Some((name, named)) = parse_constructor(source) {
        return AnnotationValue::Constructor { name, named };
    }
    if let Some(member) = parse_member_ref(source) {
        return AnnotationValue::Member(member);
    }
    AnnotationValue::Expression(source.to_owned())
}

fn parse_list(source: &str) -> Option<Vec<AnnotationValue>> {
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        split_top_level_items(inner)
            .into_iter()
            .map(parse_annotation_value)
            .collect(),
    )
}

fn parse_record(source: &str) -> Option<Vec<(String, AnnotationValue)>> {
    let inner = source.strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect()
}

fn parse_constructor(source: &str) -> Option<(String, Vec<(String, AnnotationValue)>)> {
    let source = source.strip_prefix("const ").unwrap_or(source).trim();
    let open = source.find('(')?;
    let name = source[..open].trim();
    parse_member_ref(name)?;
    let inner = source[open + 1..].strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some((name.to_owned(), Vec::new()));
    }
    let named = split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect::<Option<Vec<_>>>()?;
    Some((name.to_owned(), named))
}

fn is_number_literal(source: &str) -> bool {
    let source = source.strip_prefix('-').unwrap_or(source);
    let mut saw_digit = false;
    let mut saw_dot = false;
    for ch in source.chars() {
        match ch {
            '0'..='9' => saw_digit = true,
            '.' if !saw_dot => saw_dot = true,
            _ => return false,
        }
    }
    saw_digit
}
