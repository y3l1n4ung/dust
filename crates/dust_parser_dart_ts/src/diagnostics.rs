use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_text::SourceText;
use tree_sitter::{Node, Tree};

use crate::syntax::text_range;

pub(crate) fn extract_diagnostics(tree: &Tree, source: &SourceText) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    collect_error_diagnostics(tree.root_node(), source, &mut diagnostics);
    diagnostics
}

fn collect_error_diagnostics(
    node: Node<'_>,
    source: &SourceText,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.is_error() || node.is_missing() {
        diagnostics.push(
            Diagnostic::error("tree-sitter reported a Dart syntax error").with_label(
                SourceLabel::new(source.file_id(), text_range(node), "invalid syntax"),
            ),
        );
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_error_diagnostics(child, source, diagnostics);
    }
}
