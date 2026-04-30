#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Concrete tree-sitter backend for the Dust Dart parser contract."]

use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_parser_dart::{
    ParameterKind, ParseBackend, ParseOptions, ParseResult, ParsedAnnotation, ParsedClassKind,
    ParsedClassSurface, ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedDirective,
    ParsedEnumSurface, ParsedEnumVariantSurface, ParsedFieldSurface, ParsedLibrarySurface,
};
use dust_text::{SourceText, TextRange, TextSize};
use tree_sitter::{Node, Parser, Tree};

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

fn extract_enums(root: Node<'_>, source: &SourceText) -> Vec<ParsedEnumSurface> {
    let mut enums: Vec<ParsedEnumSurface> = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "enum_declaration" {
            enums.push(extract_enum(child, source));
        }
    }
    enums
}

fn extract_enum(node: Node<'_>, source: &SourceText) -> ParsedEnumSurface {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))
        .unwrap_or_default();

    let mut annotations: Vec<ParsedAnnotation> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }

    let mut variants: Vec<ParsedEnumVariantSurface> = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut body_cursor = body.walk();
        for member in body
            .children(&mut body_cursor)
            .filter(|child| child.is_named())
        {
            if member.kind() == "enum_constant" {
                variants.push(extract_enum_variant(member, source));
            }
        }
    }

    ParsedEnumSurface {
        name,
        annotations,
        variants,
        span: text_range(node),
    }
}

fn extract_enum_variant(node: Node<'_>, source: &SourceText) -> ParsedEnumVariantSurface {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))
        .unwrap_or_default();
    let mut annotations: Vec<ParsedAnnotation> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    ParsedEnumVariantSurface {
        name,
        annotations,
        span: text_range(node),
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

fn extract_directives(root: Node<'_>, source: &SourceText) -> Vec<ParsedDirective> {
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

fn extract_classes(root: Node<'_>, source: &SourceText) -> Vec<ParsedClassSurface> {
    let mut classes = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "class_declaration" {
            classes.push(extract_class(child, source));
        }
    }

    classes
}

fn extract_class(node: Node<'_>, source: &SourceText) -> ParsedClassSurface {
    let header = class_header_text(node, source);
    let kind = if header.contains("mixin class") {
        ParsedClassKind::MixinClass
    } else {
        ParsedClassKind::Class
    };
    let is_abstract = header.split_whitespace().any(|word| word == "abstract");
    let class_name = node
        .child_by_field_name("name")
        .map(|name| node_text(name, source))
        .unwrap_or_default();
    let superclass_name = node
        .child_by_field_name("superclass")
        .and_then(|superclass| {
            find_first_descendant_text(superclass, source, &["type_identifier"])
        });
    let mut annotations = Vec::new();
    let mut fields = Vec::new();
    let mut constructors = Vec::new();
    let class_body = node.child_by_field_name("body");

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }

    if let Some(body) = class_body {
        let mut body_cursor = body.walk();
        for member in body
            .children(&mut body_cursor)
            .filter(|child| child.is_named() && child.kind() == "class_member")
        {
            let member_annotations = extract_member_annotations(member, source);
            if let Some(declaration) = first_non_annotation_named_child(member) {
                if has_descendant_kind(declaration, "constant_constructor_signature")
                    || has_descendant_kind(declaration, "constructor_signature")
                {
                    constructors.push(extract_constructor(declaration, source));
                } else if has_descendant_kind(declaration, "initialized_identifier_list") {
                    fields.extend(extract_fields(declaration, &member_annotations, source));
                }
            }
        }
    }

    ParsedClassSurface {
        kind,
        name: class_name,
        is_abstract,
        superclass_name,
        annotations,
        fields,
        constructors,
        span: text_range(node),
    }
}

fn class_header_text(node: Node<'_>, source: &SourceText) -> String {
    let body_start = node
        .child_by_field_name("body")
        .map(|body| body.start_byte())
        .unwrap_or_else(|| node.end_byte());
    let range = TextRange::new(
        TextSize::new(node.start_byte() as u32),
        TextSize::new(body_start as u32),
    );
    source.slice(range).unwrap_or_default().to_owned()
}

fn extract_annotation(node: Node<'_>, source: &SourceText) -> ParsedAnnotation {
    let mut name = String::new();
    let mut arguments_source = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        match child.kind() {
            "identifier" if name.is_empty() => name = node_text(child, source),
            "annotation_arguments" => arguments_source = Some(node_text(child, source)),
            _ => {}
        }
    }

    ParsedAnnotation {
        name,
        arguments_source,
        span: text_range(node),
    }
}

fn extract_member_annotations(node: Node<'_>, source: &SourceText) -> Vec<ParsedAnnotation> {
    let mut annotations = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }
    annotations
}

fn extract_fields(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Vec<ParsedFieldSurface> {
    let Some(identifier_list) = find_first_descendant(node, "initialized_identifier_list") else {
        return Vec::new();
    };

    let declaration_text = node_text(node, source);
    let relative_type_end = identifier_list
        .start_byte()
        .saturating_sub(node.start_byte());
    let type_source = extract_type_prefix(&declaration_text, relative_type_end);

    let mut fields = Vec::new();
    let mut cursor = identifier_list.walk();
    for initialized in identifier_list
        .children(&mut cursor)
        .filter(|child| child.is_named() && child.kind() == "initialized_identifier")
    {
        let name =
            find_last_descendant_text(initialized, source, &["identifier"]).unwrap_or_default();
        fields.push(ParsedFieldSurface {
            name,
            annotations: annotations.to_vec(),
            type_source: type_source.clone(),
            has_default: node_text(initialized, source).contains('='),
            span: text_range(initialized),
        });
    }

    fields
}

fn extract_constructor(node: Node<'_>, source: &SourceText) -> ParsedConstructorSurface {
    let Some(signature) = find_first_descendant_by(node, |candidate| {
        matches!(
            candidate.kind(),
            "constant_constructor_signature" | "constructor_signature"
        )
    }) else {
        return ParsedConstructorSurface {
            name: None,
            params: Vec::new(),
            span: text_range(node),
        };
    };

    let mut identifiers = Vec::new();
    let mut cursor = signature.walk();
    for child in signature
        .children(&mut cursor)
        .filter(|child| child.is_named())
    {
        if child.kind() == "identifier" {
            identifiers.push(node_text(child, source));
        }
    }

    let name = if identifiers.len() > 1 {
        identifiers.get(1).cloned()
    } else {
        None
    };

    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|list| extract_constructor_params(list, source))
        .unwrap_or_default();

    ParsedConstructorSurface {
        name,
        params,
        span: text_range(signature),
    }
}

fn extract_constructor_params(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<ParsedConstructorParamSurface> {
    let mut params = Vec::new();
    collect_formal_parameters(node, source, &mut params);
    params
}

fn collect_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<ParsedConstructorParamSurface>,
) {
    if node.is_named() && node.kind() == "formal_parameter" {
        out.push(extract_formal_parameter(node, source));
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_formal_parameters(child, source, out);
    }
}

fn extract_formal_parameter(node: Node<'_>, source: &SourceText) -> ParsedConstructorParamSurface {
    let text = node_text(node, source);
    let name = find_last_descendant_text(node, source, &["identifier"]).unwrap_or_default();
    let type_source = extract_parameter_type(&text, &name);

    ParsedConstructorParamSurface {
        name,
        type_source,
        kind: determine_parameter_kind(node, source),
        has_default: text.contains('='),
        span: text_range(node),
    }
}

fn determine_parameter_kind(node: Node<'_>, source: &SourceText) -> ParameterKind {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "optional_formal_parameters" {
            let text = node_text(parent, source);
            return if text.trim_start().starts_with('{') {
                ParameterKind::Named
            } else {
                ParameterKind::Positional
            };
        }
        current = parent.parent();
    }

    ParameterKind::Positional
}

fn extract_type_prefix(declaration_text: &str, type_end: usize) -> Option<String> {
    let prefix = declaration_text.get(..type_end)?.trim();
    let stripped = strip_prefix_modifiers(prefix);
    if stripped.is_empty() || stripped == "var" {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn extract_parameter_type(text: &str, name: &str) -> Option<String> {
    if name.is_empty() || text.contains("this.") || text.contains("super.") {
        return None;
    }

    let end = text.rfind(name)?;
    let prefix = text.get(..end)?.trim();
    let stripped = strip_prefix_modifiers(prefix);
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn strip_prefix_modifiers(text: &str) -> &str {
    let mut remaining = text.trim();
    let modifiers = [
        "external",
        "static",
        "covariant",
        "late",
        "final",
        "const",
        "required",
        "factory",
        "var",
    ];

    loop {
        let mut matched = false;
        for modifier in modifiers {
            if let Some(rest) = remaining.strip_prefix(modifier) {
                remaining = rest.trim_start();
                matched = true;
                break;
            }
        }

        if !matched {
            return remaining.trim();
        }
    }
}

fn extract_diagnostics(tree: &Tree, source: &SourceText) -> Vec<Diagnostic> {
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

fn text_range(node: Node<'_>) -> TextRange {
    TextRange::new(
        TextSize::new(node.start_byte() as u32),
        TextSize::new(node.end_byte() as u32),
    )
}

fn node_text(node: Node<'_>, source: &SourceText) -> String {
    source
        .slice(text_range(node))
        .unwrap_or_default()
        .to_owned()
}

fn first_non_annotation_named_child<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.is_named() && child.kind() != "annotation")
}

fn find_first_descendant<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    find_first_descendant_by(node, |candidate| candidate.kind() == kind)
}

fn find_first_descendant_by<'tree>(
    node: Node<'tree>,
    predicate: impl Copy + Fn(Node<'tree>) -> bool,
) -> Option<Node<'tree>> {
    if predicate(node) {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_first_descendant_by(child, predicate) {
            return Some(found);
        }
    }

    None
}

fn has_descendant_kind(node: Node<'_>, kind: &str) -> bool {
    find_first_descendant(node, kind).is_some()
}

fn find_first_descendant_text(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
) -> Option<String> {
    if node.is_named() && kinds.iter().any(|kind| node.kind() == *kind) {
        return Some(node_text(node, source));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_first_descendant_text(child, source, kinds) {
            return Some(found);
        }
    }

    None
}

fn find_last_descendant_text(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
) -> Option<String> {
    let mut values = Vec::new();
    collect_descendant_texts(node, source, kinds, &mut values);
    values.pop()
}

fn collect_descendant_texts(
    node: Node<'_>,
    source: &SourceText,
    kinds: &[&str],
    out: &mut Vec<String>,
) {
    if node.is_named() && kinds.iter().any(|kind| node.kind() == *kind) {
        out.push(node_text(node, source));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_descendant_texts(child, source, kinds, out);
    }
}

fn unquote(text: String) -> String {
    text.trim().trim_matches('\'').trim_matches('"').to_owned()
}
