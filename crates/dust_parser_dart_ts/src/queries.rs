use dust_dart_syntax::{parse_static_dart_string_literal, split_top_level_items};
use dust_parser_dart::{ParsedQueryCallSurface, ParsedQueryFunction};
use dust_text::{SourceText, TextRange};
use tree_sitter::Node;

use crate::syntax::{node_text, text_range};

/// Extracts supported Dust DB query helper calls from a tree-sitter Dart tree.
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
    collect_calls(root, source, &mut calls);
    calls.sort_by_key(|call| call.span.start());
    calls
}

/// Fast source check for query helper names.
fn might_contain_query_helper(source: &str) -> bool {
    source.contains("queryAs")
        || source.contains("queryScalar")
        || source.contains("queryRaw")
        || source.contains("queryExecute")
}

/// Recursively collects query helper calls.
fn collect_calls(node: Node<'_>, source: &SourceText, out: &mut Vec<ParsedQueryCallSurface>) {
    if node.kind() == "call_expression"
        && let Some(call) = lower_query_call(node, source)
    {
        out.push(call);
    }
    if node.kind() != "call_expression"
        && let Some(call) = lower_selector_query_chain(node, source)
    {
        out.push(call);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor).filter(|child| child.is_named()) {
        collect_calls(child, source, out);
    }
}

/// Lowers a regular call expression query helper.
fn lower_query_call(node: Node<'_>, source: &SourceText) -> Option<ParsedQueryCallSurface> {
    let function_node = node.child_by_field_name("function")?;
    let (function, type_arg_source) = query_function(function_node, source)?;
    let args = argument_sources(node.child_by_field_name("arguments")?, source);
    let span = text_range(node);
    let fetch_method = fetch_method_for_query_call(node, source);

    Some(query_call_surface(
        function,
        type_arg_source,
        args,
        fetch_method,
        span,
    ))
}

/// Lowers a selector-chain query helper call.
fn lower_selector_query_chain(
    node: Node<'_>,
    source: &SourceText,
) -> Option<ParsedQueryCallSurface> {
    if node.child_count() < 2 || !has_direct_query_identifier(node, source) {
        return None;
    }

    let children = named_children(node);
    for (index, child) in children.iter().enumerate() {
        if child.kind() != "identifier" {
            continue;
        }
        let Some(function) = query_function_node(*child, source) else {
            continue;
        };

        let mut selector_index = index + 1;
        let type_arg_source = children
            .get(selector_index)
            .filter(|selector| selector.kind() == "selector")
            .and_then(|selector| selector_type_arguments_source(*selector, source));
        if type_arg_source.is_some() {
            selector_index += 1;
        }

        let query_args_selector = children
            .get(selector_index)
            .filter(|selector| selector.kind() == "selector")?;
        let args = selector_argument_sources(*query_args_selector, source)?;
        let fetch_method = children
            .get(selector_index + 1)
            .filter(|selector| selector.kind() == "selector")
            .and_then(|selector| selector_property_name(*selector, source))
            .filter(|method| is_fetch_method(method));
        let span = TextRange::new(
            child.start_byte() as u32,
            query_args_selector.end_byte() as u32,
        );

        return Some(query_call_surface(
            function,
            type_arg_source,
            args,
            fetch_method,
            span,
        ));
    }

    None
}

/// Returns whether a node has a direct query helper identifier child.
fn has_direct_query_identifier(node: Node<'_>, source: &SourceText) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.is_named()
            && child.kind() == "identifier"
            && query_function_node(child, source).is_some()
    })
}

/// Builds a parsed query call surface from extracted parts.
fn query_call_surface(
    function: ParsedQueryFunction,
    type_arg_source: Option<String>,
    args: Vec<String>,
    fetch_method: Option<String>,
    span: TextRange,
) -> ParsedQueryCallSurface {
    let (sql, sql_source_static) = args
        .first()
        .and_then(|arg| parse_static_dart_string_literal(arg).map(|sql| (sql, true)))
        .unwrap_or_else(|| (String::new(), false));
    let params_source = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("const <Object?>[]");
    let (params_source_is_list, parameter_count) = parse_list_argument_count(params_source);

    ParsedQueryCallSurface {
        function,
        type_arg_source,
        sql,
        sql_source_static,
        parameter_count,
        params_source_is_list,
        fetch_method,
        span,
    }
}

/// Extracts query function metadata from a call target node.
fn query_function(
    function_node: Node<'_>,
    source: &SourceText,
) -> Option<(ParsedQueryFunction, Option<String>)> {
    match function_node.kind() {
        "identifier" => query_function_node(function_node, source).map(|function| {
            let type_arg_source = None;
            (function, type_arg_source)
        }),
        "instantiation_expression" => {
            let identifier = function_node.child_by_field_name("function")?;
            let function = query_function_node(identifier, source)?;
            let type_arg_source = function_node
                .child_by_field_name("type_arguments")
                .and_then(|type_args| type_arguments_source(type_args, source));
            Some((function, type_arg_source))
        }
        _ => None,
    }
}

/// Converts an identifier node into a query function.
fn query_function_node(node: Node<'_>, source: &SourceText) -> Option<ParsedQueryFunction> {
    let name = source.as_str().get(node.start_byte()..node.end_byte())?;
    query_function_name(name)
}

/// Converts an identifier name into a query function.
fn query_function_name(name: &str) -> Option<ParsedQueryFunction> {
    match name {
        "queryAs" => Some(ParsedQueryFunction::As),
        "queryScalar" => Some(ParsedQueryFunction::Scalar),
        "queryRaw" => Some(ParsedQueryFunction::Raw),
        "queryExecute" => Some(ParsedQueryFunction::Execute),
        _ => None,
    }
}

/// Returns type argument source without surrounding angle brackets.
fn type_arguments_source(type_args: Node<'_>, source: &SourceText) -> Option<String> {
    let text = node_text(type_args, source);
    text.trim()
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .map(str::trim)
        .filter(|inner| !inner.is_empty())
        .map(str::to_owned)
}

/// Returns source snippets for argument nodes.
fn argument_sources(arguments: Node<'_>, source: &SourceText) -> Vec<String> {
    let mut cursor = arguments.walk();
    arguments
        .children(&mut cursor)
        .filter(|child| child.is_named())
        .filter_map(|child| argument_source(child, source))
        .collect()
}

/// Returns source for one argument value.
fn argument_source(argument: Node<'_>, source: &SourceText) -> Option<String> {
    if argument.kind() != "argument" {
        return Some(node_text(argument, source));
    }

    first_named_child(argument).map(|value| node_text(value, source))
}

/// Returns type argument source from a selector node.
fn selector_type_arguments_source(selector: Node<'_>, source: &SourceText) -> Option<String> {
    let type_args = named_children(selector)
        .into_iter()
        .find(|child| child.kind() == "type_arguments")?;
    type_arguments_source(type_args, source)
}

/// Returns query argument sources from a selector node.
fn selector_argument_sources(selector: Node<'_>, source: &SourceText) -> Option<Vec<String>> {
    let argument_part = named_children(selector)
        .into_iter()
        .find(|child| child.kind() == "argument_part")?;
    let arguments = named_children(argument_part)
        .into_iter()
        .find(|child| child.kind() == "arguments")?;
    Some(argument_sources(arguments, source))
}

/// Returns a selector property identifier.
fn selector_property_name(selector: Node<'_>, source: &SourceText) -> Option<String> {
    let assignable_selector = named_children(selector)
        .into_iter()
        .find(|child| child.kind() == "unconditional_assignable_selector")?;
    named_children(assignable_selector)
        .into_iter()
        .find(|child| child.kind() == "identifier")
        .map(|identifier| node_text(identifier, source))
}

/// Returns a chained fetch method following a query call.
fn fetch_method_for_query_call(query_call: Node<'_>, source: &SourceText) -> Option<String> {
    let member = query_call.parent()?;
    if member.kind() != "member_expression" {
        return None;
    }
    let object = member.child_by_field_name("object")?;
    if !same_node(object, query_call) {
        return None;
    }

    let method = node_text(member.child_by_field_name("property")?, source);
    if !is_fetch_method(&method) {
        return None;
    }

    Some(method)
}

/// Returns whether a method name is a supported fetch method.
fn is_fetch_method(method: &str) -> bool {
    matches!(
        method,
        "fetchOptional" | "fetchOne" | "fetchAll" | "fetch" | "execute"
    )
}

/// Returns whether two tree-sitter handles point at the same node span.
fn same_node(lhs: Node<'_>, rhs: Node<'_>) -> bool {
    lhs.kind() == rhs.kind()
        && lhs.start_byte() == rhs.start_byte()
        && lhs.end_byte() == rhs.end_byte()
}

/// Returns named direct children.
fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.is_named())
        .collect()
}

/// Returns the first named direct child.
fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|child| child.is_named())
}

/// Parses an optional generic type argument list from a source offset.
fn parse_optional_type_arg(source: &str, start: usize) -> Option<(Option<String>, usize)> {
    let start = skip_ws(source, start);
    if !source[start..].starts_with('<') {
        return Some((None, start));
    }
    let mut depth = 0_i32;
    for (relative, ch) in source[start..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + relative;
                    return Some((Some(source[start + 1..end].trim().to_owned()), end + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Returns whether source is a list literal and how many top-level items it has.
fn parse_list_argument_count(source: &str) -> (bool, usize) {
    let mut source = source.trim();
    source = source.strip_prefix("const ").unwrap_or(source).trim();
    if source.starts_with('<') {
        let Some((_, after_type)) = parse_optional_type_arg(source, 0) else {
            return (false, 0);
        };
        source = source[after_type..].trim();
    }
    let Some(inner) = source
        .strip_prefix('[')
        .and_then(|item| item.strip_suffix(']'))
    else {
        return (false, 0);
    };
    if inner.trim().is_empty() {
        return (true, 0);
    }
    (true, split_top_level_items(inner).len())
}

/// Skips whitespace from a byte offset.
fn skip_ws(source: &str, mut offset: usize) -> usize {
    while let Some(ch) = source[offset..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        offset += ch.len_utf8();
    }
    offset
}

#[cfg(test)]
#[path = "queries/tests.rs"]
mod tests;
