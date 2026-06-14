use dust_parser_dart::ParsedDirective;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::extract_member_annotations,
    syntax::{direct_named_child, find_first_descendant_text, node_text, text_range, unquote},
};

pub(crate) fn extract_directives(root: Node<'_>, source: &SourceText) -> Vec<ParsedDirective> {
    let mut directives = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        match child.kind() {
            "library_name" => directives.push(ParsedDirective::Library {
                name: direct_named_child(child, "dotted_identifier_list")
                    .map(|name| node_text(name, source)),
                annotations: extract_member_annotations(child, source),
                span: text_range(child),
            }),
            "import_or_export" => {
                if let Some(import) = direct_named_child(child, "library_import") {
                    let import_specification = direct_named_child(import, "import_specification");
                    directives.push(ParsedDirective::Import {
                        uri: find_first_descendant_text(import, source, &["uri", "string_literal"])
                            .map(unquote)
                            .unwrap_or_default(),
                        prefix: import_specification
                            .and_then(|specification| {
                                specification
                                    .child_by_field_name("alias")
                                    .or_else(|| direct_named_child(specification, "identifier"))
                            })
                            .map(|alias| node_text(alias, source))
                            .filter(|prefix| !prefix.is_empty()),
                        span: text_range(child),
                    });
                } else if let Some(export) = direct_named_child(child, "library_export") {
                    directives.push(ParsedDirective::Export {
                        uri: find_first_descendant_text(export, source, &["uri", "string_literal"])
                            .map(unquote)
                            .unwrap_or_default(),
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
                library_name: direct_named_child(child, "dotted_identifier_list")
                    .map(|name| node_text(name, source))
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
