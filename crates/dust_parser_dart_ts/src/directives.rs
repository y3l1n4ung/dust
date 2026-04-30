use dust_parser_dart::ParsedDirective;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::{find_first_descendant_text, node_text, text_range, unquote};

pub(crate) fn extract_directives(root: Node<'_>, source: &SourceText) -> Vec<ParsedDirective> {
    let mut directives = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        match child.kind() {
            "import_or_export" => {
                let text = node_text(child, source);
                let uri = find_first_descendant_text(child, source, &["uri", "string_literal"])
                    .map(unquote)
                    .unwrap_or_default();

                if text.trim_start().starts_with("import") {
                    directives.push(ParsedDirective::Import {
                        uri,
                        span: text_range(child),
                    });
                } else if text.trim_start().starts_with("export") {
                    directives.push(ParsedDirective::Export {
                        uri,
                        span: text_range(child),
                    });
                }
            }
            "part_directive" => directives.push(ParsedDirective::Part {
                uri: find_first_descendant_text(child, source, &["uri", "string_literal"])
                    .map(unquote)
                    .unwrap_or_default(),
                span: text_range(child),
            }),
            "part_of_directive" => directives.push(ParsedDirective::PartOf {
                library_name: find_first_descendant_text(child, source, &["identifier"])
                    .filter(|name| !name.is_empty()),
                uri: find_first_descendant_text(child, source, &["uri", "string_literal"])
                    .map(unquote),
                span: text_range(child),
            }),
            _ => {}
        }
    }

    directives
}
