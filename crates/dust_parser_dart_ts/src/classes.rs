use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedFieldSurface,
};
use dust_text::SourceText;
use tree_sitter::Node;

use crate::{
    annotations::{extract_annotation, extract_descendant_annotations, extract_member_annotations},
    syntax::{
        class_header_text, find_first_descendant, find_first_descendant_by,
        find_first_descendant_text, find_last_descendant_text, first_non_annotation_named_child,
        has_descendant_kind, node_text, text_range,
    },
};

pub(crate) fn extract_classes(root: Node<'_>, source: &SourceText) -> Vec<ParsedClassSurface> {
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
    let is_interface = header.contains("interface class");
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
    let mut methods = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        if child.kind() == "annotation" {
            annotations.push(extract_annotation(child, source));
        }
    }

    if let Some(body) = node.child_by_field_name("body") {
        let mut body_cursor = body.walk();
        for member in body
            .children(&mut body_cursor)
            .filter(|child| child.is_named() && child.kind() == "class_member")
        {
            let member_annotations = extract_member_annotations(member, source);
            if let Some(declaration) = first_non_annotation_named_child(member) {
                if has_descendant_kind(declaration, "constant_constructor_signature")
                    || has_descendant_kind(declaration, "constructor_signature")
                    || has_descendant_kind(declaration, "factory_constructor_signature")
                    || has_descendant_kind(declaration, "redirecting_factory_constructor_signature")
                {
                    constructors.push(extract_constructor(declaration, source));
                } else if has_descendant_kind(declaration, "initialized_identifier_list") {
                    fields.extend(extract_fields(declaration, &member_annotations, source));
                } else if has_descendant_kind(declaration, "declaration")
                    || has_descendant_kind(declaration, "method_signature")
                    || has_descendant_kind(declaration, "function_signature")
                {
                    // Tree-sitter-dart uses different kinds for methods depending on if they have a body.
                    // We'll try to extract them if they look like methods.
                    if let Some(method) = extract_method(declaration, &member_annotations, source) {
                        methods.push(method);
                    }
                }
            }
        }
    }

    ParsedClassSurface {
        kind,
        name: class_name,
        is_abstract,
        is_interface,
        superclass_name,
        annotations,
        fields,
        constructors,
        methods,
        span: text_range(node),
    }
}

fn extract_method(
    node: Node<'_>,
    annotations: &[ParsedAnnotation],
    source: &SourceText,
) -> Option<dust_parser_dart::ParsedMethodSurface> {
    let signature = find_first_descendant(node, "method_signature")
        .or_else(|| find_first_descendant(node, "function_signature"))
        .or_else(|| find_first_descendant(node, "declaration"))?;

    let name_node = signature.child_by_field_name("name")?;
    let name = node_text(name_node, source);

    let header_text = node_text(signature, source);
    let declaration_text = node_text(node, source);
    let is_static = header_text.contains("static");
    let is_external = header_text.contains("external");

    let return_type_source = signature
        .child_by_field_name("type")
        .map(|t| node_text(t, source))
        .or_else(|| extract_parameter_type(&header_text, &name));

    let params = find_first_descendant(signature, "formal_parameter_list")
        .map(|list| extract_method_params(list, source))
        .unwrap_or_default();

    let params_node = find_first_descendant(signature, "formal_parameter_list");
    let has_body = if let Some(params) = params_node {
        let end_offset = params.end_byte().saturating_sub(node.start_byte());
        let after_params = &declaration_text[end_offset.min(declaration_text.len())..];
        after_params.contains('{') || after_params.contains("=>")
    } else {
        declaration_text.contains('{') || declaration_text.contains("=>")
    };

    Some(dust_parser_dart::ParsedMethodSurface {
        name,
        is_static,
        is_external,
        annotations: annotations.to_vec(),
        return_type_source,
        has_body,
        params,
        span: text_range(signature),
    })
}

fn extract_method_params(
    node: Node<'_>,
    source: &SourceText,
) -> Vec<dust_parser_dart::ParsedMethodParamSurface> {
    let mut params = Vec::new();
    collect_method_formal_parameters(node, source, &mut params, &mut Vec::new());
    params
}

fn collect_method_formal_parameters(
    node: Node<'_>,
    source: &SourceText,
    out: &mut Vec<dust_parser_dart::ParsedMethodParamSurface>,
    pending_annotations: &mut Vec<ParsedAnnotation>,
) {
    if !node.is_named() {
        return;
    }

    if node.kind() == "annotation" {
        pending_annotations.push(extract_annotation(node, source));
        return;
    }

    if node.is_named() && matches!(node.kind(), "formal_parameter" | "default_formal_parameter") {
        let mut param = extract_method_formal_parameter(node, source);
        if !pending_annotations.is_empty() {
            let mut annotations = std::mem::take(pending_annotations);
            annotations.extend(param.annotations);
            param.annotations = annotations;
        }
        out.push(param);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_method_formal_parameters(child, source, out, pending_annotations);
    }
}

fn extract_method_formal_parameter(
    node: Node<'_>,
    source: &SourceText,
) -> dust_parser_dart::ParsedMethodParamSurface {
    let text = node_text(node, source);
    let name = find_last_descendant_text(node, source, &["identifier"]).unwrap_or_default();
    let type_source = extract_parameter_type(&text, &name);

    dust_parser_dart::ParsedMethodParamSurface {
        name,
        annotations: extract_descendant_annotations(node, source),
        type_source,
        kind: determine_parameter_kind(node, source),
        has_default: text.contains('=') || trailing_default_value_source(node, source).is_some(),
        default_value_source: extract_default_value_source(&text)
            .or_else(|| trailing_default_value_source(node, source)),
        span: text_range(node),
    }
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
            "constant_constructor_signature"
                | "constructor_signature"
                | "factory_constructor_signature"
                | "redirecting_factory_constructor_signature"
        )
    }) else {
        return ParsedConstructorSurface {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            params: Vec::new(),
            span: text_range(node),
        };
    };
    let declaration_text = node_text(node, source);
    let is_factory = declaration_text
        .split_whitespace()
        .any(|word| word == "factory");

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
    let redirected_target_source = extract_redirect_target(&declaration_text);
    let redirected_target_name = redirected_target_source
        .as_deref()
        .and_then(extract_redirect_target_name);

    ParsedConstructorSurface {
        name,
        is_factory,
        redirected_target_source,
        redirected_target_name,
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
    if node.is_named() && matches!(node.kind(), "formal_parameter" | "default_formal_parameter") {
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
        has_default: text.contains('=') || trailing_default_value_source(node, source).is_some(),
        default_value_source: extract_default_value_source(&text)
            .or_else(|| trailing_default_value_source(node, source)),
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
    let stripped = strip_prefix_modifiers(strip_leading_annotations(prefix));
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn extract_default_value_source(text: &str) -> Option<String> {
    let equals_index = top_level_equals_index(text)?;
    let raw_default = text.get(equals_index + '='.len_utf8()..)?;
    let default_end = top_level_default_end(raw_default).unwrap_or(raw_default.len());
    let default = raw_default
        .get(..default_end)?
        .trim()
        .trim_end_matches(',')
        .trim();
    if default.is_empty() {
        None
    } else {
        Some(default.to_owned())
    }
}

fn top_level_default_end(text: &str) -> Option<usize> {
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;
    let mut brace_depth = 0_u32;
    let mut quote = None;
    let mut escape = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            ',' | '}' | ']' | ')' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
    }

    None
}

fn trailing_default_value_source(node: Node<'_>, source: &SourceText) -> Option<String> {
    let parent = node.parent()?;
    let tail = source.as_str().get(node.end_byte()..parent.end_byte())?;
    tail.trim_start()
        .starts_with('=')
        .then(|| extract_default_value_source(tail))
        .flatten()
}

fn top_level_equals_index(text: &str) -> Option<usize> {
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;
    let mut brace_depth = 0_u32;
    let mut quote = None;
    let mut escape = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '=' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
    }

    None
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

fn strip_leading_annotations(text: &str) -> &str {
    let mut remaining = text.trim_start();
    while remaining.starts_with('@') {
        remaining = &remaining[1..];
        let mut boundary = 0;
        for (index, ch) in remaining.char_indices() {
            if ch == '_' || ch == '$' || ch == '.' || ch.is_ascii_alphanumeric() {
                boundary = index + ch.len_utf8();
            } else {
                break;
            }
        }

        remaining = remaining.get(boundary..).unwrap_or("").trim_start();
        if remaining.starts_with('(') {
            let consumed = consume_parenthesized(remaining);
            remaining = remaining.get(consumed..).unwrap_or("").trim_start();
        }
    }
    remaining
}

fn consume_parenthesized(text: &str) -> usize {
    let mut depth = 0_u32;
    let mut quote = None;
    let mut escape = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index + ch.len_utf8();
                }
            }
            _ => {}
        }
    }

    text.len()
}

fn extract_redirect_target(text: &str) -> Option<String> {
    let (_, rhs) = text.split_once('=')?;
    let target = rhs.trim().trim_end_matches(';').trim();
    if target.is_empty() {
        None
    } else {
        Some(target.to_owned())
    }
}

fn extract_redirect_target_name(text: &str) -> Option<String> {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch == '_' || ch == '$' || ch.is_ascii_alphabetic() {
            break;
        }
        chars.next();
    }

    let mut name = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch == '_' || ch == '$' || ch == '.' || ch.is_ascii_alphanumeric() {
            name.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() { None } else { Some(name) }
}
