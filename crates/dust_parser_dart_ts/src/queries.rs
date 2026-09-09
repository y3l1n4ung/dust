use dust_dart_syntax::{parse_static_dart_string_literal, split_top_level_items};
use dust_parser_dart::{ParsedQueryCallSurface, ParsedQueryFunction};
use dust_text::{SourceText, TextRange};
use tree_sitter::Node;

use crate::syntax::{node_text, text_range};

/// Lowering a Dart method chain into one query call.
mod chains;
/// Reading arguments and names off a call node.
mod selectors;

use self::{chains::*, selectors::*};

/// Extracts supported Database query helper calls from a tree-sitter Dart tree.
pub(crate) fn extract_query_calls(
    root: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedQueryCallSurface> {
    // Fast coarse gate for the common case: most Dart files do not contain DB
    // helper names. Actual query discovery below still uses tree-sitter nodes.
    if !might_contain_query_helper(source.as_str()) {
        return Vec::new();
    }

    let mut calls = Vec::new();
    collect_calls(root, source, &Enclosing::default(), &mut calls);
    calls.sort_by_key(|call| call.span.start());
    calls
}

/// Fast source check for query helper names.
fn might_contain_query_helper(source: &str) -> bool {
    source.contains("queryAs")
        || source.contains("queryScalar")
        || source.contains("queryExecute")
        || source.contains(".unsafe")
}

/// The declaration a query call sits inside, for diagnostics.
///
/// A bare helper name says nothing in a file holding several queries, and the
/// span alone is not what a reader searches for.
#[derive(Debug, Default, Clone)]
struct Enclosing {
    /// Nearest enclosing class, when the call is inside one.
    class: Option<String>,
    /// Nearest enclosing named function or method.
    function: Option<String>,
}

impl Enclosing {
    /// Renders the name a diagnostic should use, if there is one.
    ///
    /// A method reads as `Class.method`; a top-level function as its own name.
    /// A call in a field initializer has neither and keeps the helper name.
    fn name(&self) -> Option<String> {
        let function = self.function.as_deref()?;
        Some(match self.class.as_deref() {
            Some(class) => format!("{class}.{function}"),
            None => function.to_owned(),
        })
    }
}

/// Recursively collects query helper calls.
fn collect_calls(
    node: Node<'_>,
    source: &SourceText,
    enclosing: &Enclosing,
    out: &mut Vec<ParsedQueryCallSurface>,
) {
    // A closure's body is a `function_expression_body`, not a `function_body`,
    // so a query inside one keeps the name of the function containing it rather
    // than reporting an anonymous closure.
    let enclosing = match node.kind() {
        "class_declaration" => &Enclosing {
            class: declaration_name(node, source),
            function: None,
        },
        "function_body" => &Enclosing {
            class: enclosing.class.clone(),
            function: signature_name_before(node, source).or_else(|| enclosing.function.clone()),
        },
        _ => enclosing,
    };
    // One place stamps the enclosing name, so every lowering path reports the
    // same way.
    let lowered = if node.kind() == "call_expression" {
        lower_query_call(node, source)
    } else {
        lower_selector_query_chain(node, source).or_else(|| lower_unsafe_sql_chain(node, source))
    };
    if let Some(mut call) = lowered {
        call.enclosing_name = enclosing.name();
        out.push(call);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        collect_calls(child, source, enclosing, out);
    }
}

/// Returns the first `identifier` child of [node].
fn declaration_name(node: Node<'_>, source: &SourceText) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == "identifier")
        .map(|child| node_text(child, source).to_owned())
}

/// Returns the name of the signature a function body belongs to.
///
/// A body is a sibling of its signature rather than a child of it, and a method
/// wraps that signature in a `method_signature`.
fn signature_name_before(node: Node<'_>, source: &SourceText) -> Option<String> {
    let signature = node.prev_named_sibling()?;
    let signature = match signature.kind() {
        "function_signature" => signature,
        "method_signature" => {
            let mut cursor = signature.walk();
            signature
                .children(&mut cursor)
                .find(|child| child.kind() == "function_signature")?
        }
        _ => return None,
    };
    declaration_name(signature, source)
}

#[cfg(test)]
#[path = "queries/tests.rs"]
mod tests;
