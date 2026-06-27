use std::collections::BTreeMap;

use crate::{AnnotationValueIr, SpanIr};
use dust_dart_syntax::{
    normalized_args, parse_bool_literal, parse_string_list, parse_string_literal, parse_string_map,
    parse_type_list, parse_type_name, split_top_level_items, split_top_level_once,
};

/// A stable semantic identifier for a trait or config symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId(pub String);

impl SymbolId {
    /// Creates a symbol identifier from a fully qualified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// One resolved trait application on a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitApplicationIr {
    /// The resolved trait symbol.
    pub symbol: SymbolId,
    /// The source span of the trait annotation.
    pub span: SpanIr,
}

/// One resolved configuration application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigApplicationIr {
    /// The resolved config symbol.
    pub symbol: SymbolId,
    /// The raw annotation argument source, if the config was written with
    /// parentheses and arguments.
    pub arguments_source: Option<String>,
    /// Positional annotation argument values preserved from parser-owned facts.
    pub positional_args: Vec<AnnotationValueIr>,
    /// Named annotation argument values preserved from parser-owned facts.
    pub named_args: BTreeMap<String, AnnotationValueIr>,
    /// The source span of the config annotation.
    pub span: SpanIr,
}

impl ConfigApplicationIr {
    /// Creates a config application with compatibility raw argument source only.
    pub fn new(symbol: SymbolId, arguments_source: Option<String>, span: SpanIr) -> Self {
        Self {
            symbol,
            arguments_source,
            positional_args: Vec::new(),
            named_args: BTreeMap::new(),
            span,
        }
    }

    /// Creates a config application with parser-owned structured arguments.
    pub fn with_arguments(
        symbol: SymbolId,
        arguments_source: Option<String>,
        positional_args: Vec<AnnotationValueIr>,
        named_args: BTreeMap<String, AnnotationValueIr>,
        span: SpanIr,
    ) -> Self {
        Self {
            symbol,
            arguments_source,
            positional_args,
            named_args,
            span,
        }
    }

    /// Returns one positional argument value by index.
    pub fn positional_argument_value(&self, index: usize) -> Option<&AnnotationValueIr> {
        self.positional_args.get(index)
    }

    /// Returns one named argument value by name.
    pub fn named_argument_value(&self, name: &str) -> Option<&AnnotationValueIr> {
        self.named_args.get(name)
    }

    /// Returns the annotation argument list without the outer parentheses.
    pub fn normalized_arguments(&self) -> Option<&str> {
        normalized_args(self.arguments_source.as_deref()?)
    }

    /// Returns one top-level positional argument source by index.
    pub fn positional_argument_source(&self, index: usize) -> Option<&str> {
        if let Some(source) = self
            .positional_argument_value(index)
            .and_then(annotation_value_source)
        {
            return Some(source);
        }

        self.argument_items()
            .into_iter()
            .filter(|item| split_top_level_once(item, ':').is_none())
            .nth(index)
    }

    /// Returns one top-level positional string literal by index.
    pub fn positional_string(&self, index: usize) -> Option<String> {
        parse_string_literal(self.positional_argument_source(index)?)
    }

    /// Returns one top-level positional string map literal by index.
    pub fn positional_string_map(&self, index: usize) -> Option<Vec<(String, String)>> {
        parse_string_map(self.positional_argument_source(index)?)
    }

    /// Returns one top-level named argument source by name.
    pub fn named_argument_source(&self, name: &str) -> Option<&str> {
        if let Some(source) = self
            .named_argument_value(name)
            .and_then(annotation_value_source)
        {
            return Some(source);
        }

        self.named_arguments()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }

    /// Returns one top-level named argument as expression source.
    pub fn named_expression_source(&self, name: &str) -> Option<String> {
        self.named_argument_source(name)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    /// Returns one top-level named string literal.
    pub fn named_string(&self, name: &str) -> Option<String> {
        parse_string_literal(self.named_argument_source(name)?)
    }

    /// Returns one top-level named list of string literals.
    pub fn named_string_list(&self, name: &str) -> Option<Vec<String>> {
        parse_string_list(self.named_argument_source(name)?)
    }

    /// Returns one top-level named boolean literal.
    pub fn named_bool(&self, name: &str) -> Option<bool> {
        parse_bool_literal(self.named_argument_source(name)?)
    }

    /// Returns one top-level named member or type reference.
    pub fn named_member(&self, name: &str) -> Option<String> {
        parse_type_name(self.named_argument_source(name)?)
    }

    /// Returns one top-level named type reference.
    pub fn named_type(&self, name: &str) -> Option<String> {
        self.named_member(name)
    }

    /// Returns one top-level positional type reference.
    pub fn positional_type(&self, index: usize) -> Option<String> {
        parse_type_name(self.positional_argument_source(index)?)
    }

    /// Returns one top-level named list of type references.
    pub fn named_type_list(&self, name: &str) -> Option<Vec<String>> {
        parse_type_list(self.named_argument_source(name)?)
    }

    /// Returns one top-level named string map literal.
    pub fn named_string_map(&self, name: &str) -> Option<Vec<(String, String)>> {
        parse_string_map(self.named_argument_source(name)?)
    }

    /// Returns whether a top-level named argument is present.
    pub fn has_named_argument(&self, name: &str) -> bool {
        self.named_argument_source(name).is_some()
    }

    /// Returns all top-level annotation argument items.
    pub fn argument_items(&self) -> Vec<&str> {
        self.normalized_arguments()
            .map(split_top_level_items)
            .unwrap_or_default()
    }

    /// Returns all top-level named annotation arguments as key/source pairs.
    pub fn named_arguments(&self) -> Vec<(&str, &str)> {
        self.argument_items()
            .into_iter()
            .filter_map(|item| {
                let (key, value) = split_top_level_once(item, ':')?;
                Some((key.trim(), value.trim()))
            })
            .collect()
    }
}

/// Returns source text for annotation values that are backed by raw source.
fn annotation_value_source(value: &AnnotationValueIr) -> Option<&str> {
    match value {
        AnnotationValueIr::Number { source, .. } => Some(source),
        AnnotationValueIr::Member(name) => Some(name.source.as_str()),
        AnnotationValueIr::Expression(source) => Some(source.source.as_str()),
        AnnotationValueIr::Null
        | AnnotationValueIr::Bool(_)
        | AnnotationValueIr::String(_)
        | AnnotationValueIr::List(_)
        | AnnotationValueIr::Set(_)
        | AnnotationValueIr::Map(_)
        | AnnotationValueIr::Record(_)
        | AnnotationValueIr::Constructor { .. } => None,
    }
}
