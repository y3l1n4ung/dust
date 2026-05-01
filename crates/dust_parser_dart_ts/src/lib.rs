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
use std::cell::RefCell;
use tree_sitter::Parser;

use self::{
    classes::extract_classes, diagnostics::extract_diagnostics, directives::extract_directives,
    enums::extract_enums, syntax::text_range,
};

thread_local! {
    static PARSER: RefCell<Parser> = {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_dart::LANGUAGE.into())
            .expect("failed to load tree-sitter Dart grammar");
        RefCell::new(parser)
    };
}

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
        PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
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
        })
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
