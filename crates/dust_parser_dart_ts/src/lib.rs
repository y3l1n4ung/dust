#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Concrete tree-sitter backend for the Dust Dart parser contract."]

mod annotations;
mod classes;
mod diagnostics;
mod directives;
mod enums;
mod syntax;

use dust_diagnostics::Diagnostic;
use dust_parser_dart::{ParseBackend, ParseOptions, ParseResult, ParsedLibrarySurface};
use dust_text::SourceText;
use tree_sitter::Parser;

use self::{
    classes::extract_classes, diagnostics::extract_diagnostics, directives::extract_directives,
    enums::extract_enums, syntax::text_range,
};

/// A `tree-sitter-dart` implementation of Dust's parser backend contract.
///
/// This type owns no source state. It can be reused across parse calls by
/// creating one value and calling [`ParseBackend::parse_file`] repeatedly.
pub struct TreeSitterDartBackend;

impl TreeSitterDartBackend {
    /// Creates a new tree-sitter Dart backend.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for TreeSitterDartBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseBackend for TreeSitterDartBackend {
    fn parse_file(&self, source: &SourceText, options: ParseOptions) -> ParseResult {
        let mut parser = Parser::new();
        if let Err(error) = parser.set_language(&tree_sitter_dart::LANGUAGE.into()) {
            return ParseResult {
                library: empty_library(source),
                diagnostics: vec![Diagnostic::error(format!(
                    "failed to load tree-sitter Dart grammar: {error}"
                ))],
                options,
            };
        }

        let Some(tree) = parser.parse(source.as_str(), None) else {
            return ParseResult {
                library: empty_library(source),
                diagnostics: vec![Diagnostic::error("tree-sitter failed to parse source")],
                options,
            };
        };

        let root = tree.root_node();
        ParseResult {
            library: ParsedLibrarySurface {
                span: text_range(root),
                directives: extract_directives(root, source),
                classes: extract_classes(root, source),
                enums: extract_enums(root, source),
            },
            diagnostics: extract_diagnostics(&tree, source),
            options,
        }
    }
}

fn empty_library(source: &SourceText) -> ParsedLibrarySurface {
    ParsedLibrarySurface {
        span: source.full_range(),
        directives: Vec::new(),
        classes: Vec::new(),
        enums: Vec::new(),
    }
}
