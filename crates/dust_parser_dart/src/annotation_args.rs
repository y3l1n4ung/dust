use crate::surface::ParsedAnnotation;
use dust_dart_syntax::{
    normalized_args, parse_bool_literal, parse_string_list, parse_string_literal, parse_string_map,
    parse_type_list, parse_type_name, split_top_level_items, split_top_level_once,
};

impl ParsedAnnotation {
    /// Returns the annotation argument list without the outer parentheses.
    pub fn normalized_arguments(&self) -> Option<&str> {
        normalized_args(self.arguments_source.as_deref()?)
    }

    /// Returns one top-level positional argument source by index.
    pub fn positional_argument_source(&self, index: usize) -> Option<&str> {
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
