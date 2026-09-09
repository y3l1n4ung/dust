//! Lowering a Dart method chain into one query call.
//!
//! A query reaches the parser as `db.query<Row>(...).fetchAll()` or as an
//! `unsafe` chain, so the shape has to be walked before anything can be read
//! off it.

use super::*;

/// Lowers a regular call expression query helper.
pub(super) fn lower_query_call(
    node: Node<'_>,
    source: &SourceText,
) -> Option<ParsedQueryCallSurface> {
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
pub(super) fn lower_selector_query_chain(
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

/// Lowers `<facade>.unsafe.fetch(...)`, `.fetchAs(...)`, or `.execute(...)`.
///
/// The escape hatch is a member chain rather than a helper function, so it
/// parses as a selector chain and not as a call expression with a named
/// callee. Collecting it here is what lets one warning per use be reported
/// from the same place every other query call is validated.
pub(super) fn lower_unsafe_sql_chain(
    node: Node<'_>,
    source: &SourceText,
) -> Option<ParsedQueryCallSurface> {
    if node.child_count() < 3 {
        return None;
    }

    let children = named_children(node);
    for (index, child) in children.iter().enumerate() {
        if child.kind() != "selector"
            || selector_property_name(*child, source).as_deref() != Some("unsafe")
        {
            continue;
        }

        let terminal = children.get(index + 1).filter(|s| s.kind() == "selector")?;
        let method = selector_property_name(*terminal, source)
            .filter(|name| matches!(name.as_str(), "fetch" | "fetchAs" | "execute"))?;

        // `fetchAs<T>` carries its type argument in a selector of its own.
        let mut args_index = index + 2;
        let type_arg_source = children
            .get(args_index)
            .filter(|selector| selector.kind() == "selector")
            .and_then(|selector| selector_type_arguments_source(*selector, source));
        if type_arg_source.is_some() {
            args_index += 1;
        }

        let args_selector = children
            .get(args_index)
            .filter(|s| s.kind() == "selector")?;
        let args = selector_argument_sources(*args_selector, source)?;
        let span = TextRange::new(child.start_byte() as u32, args_selector.end_byte() as u32);

        let mut surface = query_call_surface(
            ParsedQueryFunction::Unsafe,
            type_arg_source,
            args,
            Some(method),
            span,
        );
        surface.unsafe_sql_allowed = unsafe_sql_allowed_at(source, child.start_byte());
        return Some(surface);
    }

    None
}

/// Marker comment that silences the warning for one unchecked SQL call.
pub(super) const UNSAFE_SQL_ALLOW_MARKER: &str = "dust:allow-unsafe-sql";

/// Returns whether a marker comment covers the call starting at `offset`.
///
/// The marker is accepted on the call's own line or on the line above it, which
/// are the two places a reader looks. Anything wider would let one marker cover
/// a whole file, and the point is that each use reads as a deliberate line in a
/// diff.
pub(super) fn unsafe_sql_allowed_at(source: &SourceText, offset: usize) -> bool {
    let text = source.as_str();
    let line_start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let line_end = text[offset..]
        .find('\n')
        .map_or(text.len(), |at| offset + at);
    if text[line_start..line_end].contains(UNSAFE_SQL_ALLOW_MARKER) {
        return true;
    }
    let previous_start = text[..line_start.saturating_sub(1)]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    text[previous_start..line_start].contains(UNSAFE_SQL_ALLOW_MARKER)
}

/// Returns whether a node has a direct query helper identifier child.
pub(super) fn has_direct_query_identifier(node: Node<'_>, source: &SourceText) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.is_named()
            && child.kind() == "identifier"
            && query_function_node(child, source).is_some()
    })
}
