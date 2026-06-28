/// Query-match lowering for static i18n API calls.
mod lower;

use dust_diagnostics::Diagnostic;
use dust_text::{SourceText, TextRange};
use tree_sitter::{Query, QueryCursor, StreamingIterator};

use self::lower::lower_match;
use crate::PARSER;

/// Result of scanning one Dart source file for static i18n usage.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct I18nScanResult {
    /// Static translation usages discovered in source order.
    pub entries: Vec<I18nTranslationUse>,
    /// Warnings found while scanning static i18n APIs.
    pub diagnostics: Vec<Diagnostic>,
}

/// One static translation use found in Dart source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nTranslationUse {
    /// Static translation key.
    pub key: String,
    /// Namespace inferred from the first key segment.
    pub namespace: String,
    /// Optional fallback text supplied by the call.
    pub default_text: Option<String>,
    /// Placeholder argument names supplied by the call.
    pub args: Vec<String>,
    /// API shape that produced this entry.
    pub kind: I18nTranslationKind,
    /// Full source span of the scanned call.
    pub span: TextRange,
}

/// Public i18n API call shape recognized by the scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I18nTranslationKind {
    /// `TranslatedText("key")`.
    TranslatedText,
    /// `context.tr("key")`.
    ContextTr,
}

/// Scans one Dart source for static i18n keys.
pub fn scan_i18n_source(source: &SourceText) -> I18nScanResult {
    let mut result = I18nScanResult::default();
    let Some(tree) = parse_tree(source, &mut result.diagnostics) else {
        return result;
    };
    let Some(query) = i18n_query(&mut result.diagnostics) else {
        return result;
    };

    let mut cursor = QueryCursor::new();
    let bytes = source.as_str().as_bytes();
    let mut matches = cursor.matches(&query, tree.root_node(), bytes);
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };
        if let Some(entry) = lower_match(source, &query, query_match, &mut result.diagnostics) {
            result.entries.push(entry);
        }
    }

    result.entries.sort_by_key(|entry| entry.span.start());
    result
        .diagnostics
        .sort_by_key(|diagnostic| diagnostic.labels.first().map(|label| label.range.start()));
    result
}

/// Tree-sitter query for Dart call shapes relevant to i18n.
const I18N_QUERY: &str = r#"
(const_object_expression
  type: (type_identifier) @constructor
  arguments: (arguments) @arguments) @call

(selector
  (argument_part
    (arguments) @arguments)) @call
"#;

/// Parses source into a tree-sitter tree.
fn parse_tree(source: &SourceText, diagnostics: &mut Vec<Diagnostic>) -> Option<tree_sitter::Tree> {
    PARSER
        .with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            parser.parse(source.as_str(), None)
        })
        .or_else(|| {
            diagnostics.push(Diagnostic::error("tree-sitter failed to parse source"));
            None
        })
}

/// Compiles the i18n query.
fn i18n_query(diagnostics: &mut Vec<Diagnostic>) -> Option<Query> {
    Query::new(&tree_sitter_dart::LANGUAGE.into(), I18N_QUERY)
        .map_err(|error| {
            diagnostics.push(Diagnostic::error(format!(
                "failed to compile i18n tree-sitter query: {}",
                error.message
            )));
        })
        .ok()
}

#[cfg(test)]
mod tests;
