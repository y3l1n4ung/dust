use crate::surface::ParsedAnnotation;
use dust_dart_syntax::{
    normalized_args, parse_bool_literal, parse_constructor_list, parse_constructor_name,
    parse_string_list, parse_string_literal, parse_string_map, parse_type_list, parse_type_name,
    split_top_level_items, split_top_level_once,
};

impl ParsedAnnotation {
    /// Returns the annotation argument list without the outer parentheses.
    pub fn normalized_arguments(&self) -> Option<&str> {
        normalized_args(self.arguments_source.as_deref()?)
    }

    /// Returns one top-level positional argument source by index.
    pub fn positional_argument_source(&self, index: usize) -> Option<&str> {
        if let Some(arguments) = &self.parsed_arguments {
            return arguments
                .positional
                .get(index)
                .map(|argument| argument.source.as_str());
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
        if let Some(arguments) = &self.parsed_arguments {
            return arguments.named.iter().find_map(|argument| {
                (argument.name == name).then_some(argument.value_source.as_str())
            });
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

    /// Returns constructor names from positional annotation argument expressions.
    pub fn positional_constructor_names(&self) -> Vec<String> {
        self.positional_argument_sources()
            .into_iter()
            .flat_map(constructor_names_from_argument)
            .collect()
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
        if let Some(arguments) = &self.parsed_arguments {
            return arguments
                .positional
                .iter()
                .map(|argument| argument.source.as_str())
                .chain(
                    arguments
                        .named
                        .iter()
                        .map(|argument| argument.source.as_str()),
                )
                .collect();
        }

        self.normalized_arguments()
            .map(split_top_level_items)
            .unwrap_or_default()
    }

    /// Returns all top-level named annotation arguments as key/source pairs.
    pub fn named_arguments(&self) -> Vec<(&str, &str)> {
        if let Some(arguments) = &self.parsed_arguments {
            return arguments
                .named
                .iter()
                .map(|argument| (argument.name.as_str(), argument.value_source.as_str()))
                .collect();
        }

        self.argument_items()
            .into_iter()
            .filter_map(|item| {
                let (key, value) = split_top_level_once(item, ':')?;
                Some((key.trim(), value.trim()))
            })
            .collect()
    }

    /// Returns positional argument source snippets.
    fn positional_argument_sources(&self) -> Vec<&str> {
        if let Some(arguments) = &self.parsed_arguments {
            return arguments
                .positional
                .iter()
                .map(|argument| argument.source.as_str())
                .collect();
        }

        self.argument_items()
            .into_iter()
            .filter(|item| split_top_level_once(item, ':').is_none())
            .collect()
    }
}

/// Extracts constructor names from one positional annotation argument.
fn constructor_names_from_argument(source: &str) -> Vec<String> {
    parse_constructor_list(source)
        .or_else(|| parse_constructor_name(source).map(|name| vec![name]))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ParsedAnnotationArgument, ParsedAnnotationArguments, ParsedAnnotationNamedArgument,
    };
    use dust_text::TextRange;

    #[test]
    fn constructor_names_prefer_structured_positional_arguments() {
        let annotation = ParsedAnnotation {
            name: "Derive".to_owned(),
            prefix: None,
            qualified_name: "Derive".to_owned(),
            arguments_source: Some("(ignored: Unknown())".to_owned()),
            parsed_arguments: Some(ParsedAnnotationArguments {
                positional: vec![ParsedAnnotationArgument {
                    source: "[ToString(), const prefix.CopyWith<User>()]".to_owned(),
                    span: TextRange::new(0_u32, 42_u32),
                }],
                named: vec![ParsedAnnotationNamedArgument {
                    name: "ignored".to_owned(),
                    source: "ignored: Unknown()".to_owned(),
                    value_source: "Unknown()".to_owned(),
                    span: TextRange::new(44_u32, 62_u32),
                    value_span: TextRange::new(53_u32, 62_u32),
                }],
            }),
            span: TextRange::new(0_u32, 64_u32),
        };

        assert_eq!(
            annotation.positional_constructor_names(),
            vec!["ToString".to_owned(), "CopyWith".to_owned()]
        );
    }

    #[test]
    fn constructor_names_fall_back_to_raw_positional_arguments() {
        let annotation = ParsedAnnotation {
            name: "Derive".to_owned(),
            prefix: None,
            qualified_name: "Derive".to_owned(),
            arguments_source: Some("([ToString(), Eq()])".to_owned()),
            parsed_arguments: None,
            span: TextRange::new(0_u32, 24_u32),
        };

        assert_eq!(
            annotation.positional_constructor_names(),
            vec!["ToString".to_owned(), "Eq".to_owned()]
        );
    }
}
