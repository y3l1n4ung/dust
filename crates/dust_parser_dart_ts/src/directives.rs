use dust_parser_dart::ParsedDirective;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::extract_member_annotations,
    syntax::{
        direct_named_child, find_first_descendant_text, has_descendant_kind, node_text, text_range,
        unquote,
    },
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
                    let (show, hide) = import_combinators(import, source);
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
                        show,
                        hide,
                        is_deferred: has_descendant_kind(import, "deferred"),
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

fn import_combinators(import: Node<'_>, source: &SourceText) -> (Vec<String>, Vec<String>) {
    let mut show = Vec::new();
    let mut hide = Vec::new();
    collect_import_combinators(import, source, &mut show, &mut hide);

    (show, hide)
}

fn collect_import_combinators(
    node: Node<'_>,
    source: &SourceText,
    show: &mut Vec<String>,
    hide: &mut Vec<String>,
) {
    if node.is_named() && node.kind() == "combinator" {
        collect_import_combinator(node, source, show, hide);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        collect_import_combinators(child, source, show, hide);
    }
}

fn collect_import_combinator(
    combinator: Node<'_>,
    source: &SourceText,
    show: &mut Vec<String>,
    hide: &mut Vec<String>,
) {
    let combinator_source = node_text(combinator, source);
    let target = if combinator_source.trim_start().starts_with("show") {
        show
    } else if combinator_source.trim_start().starts_with("hide") {
        hide
    } else {
        return;
    };

    let mut cursor = combinator.walk();
    target.extend(
        combinator
            .children(&mut cursor)
            .filter(|node| node.is_named() && node.kind() == "identifier")
            .map(|identifier| node_text(identifier, source)),
    );
}
