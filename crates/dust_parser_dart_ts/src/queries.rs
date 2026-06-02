use dust_dart_emit::{
    balanced_parenthesized, parse_static_dart_string_literal, split_top_level_items,
};
use dust_parser_dart::{ParsedQueryCallSurface, ParsedQueryFunction};
use dust_text::{SourceText, TextRange};

/// Extracts supported Dust DB query helper calls from Dart source.
pub(crate) fn extract_query_calls(source: &SourceText) -> Vec<ParsedQueryCallSurface> {
    let mut calls = Vec::new();
    for (name, function) in [
        ("queryAs", ParsedQueryFunction::As),
        ("queryScalar", ParsedQueryFunction::Scalar),
        ("queryRaw", ParsedQueryFunction::Raw),
        ("queryExecute", ParsedQueryFunction::Execute),
    ] {
        collect_calls(source.as_str(), name, function, &mut calls);
    }
    calls.sort_by_key(|call| call.span.start());
    calls
}

fn collect_calls(
    source: &str,
    name: &str,
    function: ParsedQueryFunction,
    out: &mut Vec<ParsedQueryCallSurface>,
) {
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(name) {
        let start = offset + relative;
        if !is_code_position(source, start) || !is_identifier_boundary(source, start, name.len()) {
            offset = start + name.len();
            continue;
        }
        let Some((type_arg, after_type)) = parse_optional_type_arg(source, start + name.len())
        else {
            offset = start + name.len();
            continue;
        };
        let after_type = skip_ws(source, after_type);
        if !source[after_type..].starts_with('(') {
            offset = start + name.len();
            continue;
        }
        let Some(call) = balanced_parenthesized(&source[after_type..]) else {
            offset = start + name.len();
            continue;
        };
        let call_end = after_type + call.len();
        let args = call
            .strip_prefix('(')
            .and_then(|inner| inner.strip_suffix(')'))
            .map(split_top_level_items)
            .unwrap_or_default();
        let (sql, sql_source_static) = args
            .first()
            .and_then(|arg| parse_static_dart_string_literal(arg).map(|sql| (sql, true)))
            .unwrap_or_else(|| (String::new(), false));
        let params_source = args.get(1).copied().unwrap_or("const <Object?>[]");
        let (params_source_is_list, parameter_count) = parse_list_argument_count(params_source);

        out.push(ParsedQueryCallSurface {
            function,
            type_arg_source: type_arg,
            sql,
            sql_source_static,
            parameter_count,
            params_source_is_list,
            fetch_method: parse_fetch_method(&source[call_end..]),
            span: TextRange::new(start as u32, call_end as u32),
        });
        offset = call_end;
    }
}

fn parse_fetch_method(after_call: &str) -> Option<String> {
    let after = after_call.trim_start();
    ["fetchOptional", "fetchOne", "fetchAll", "fetch", "execute"]
        .into_iter()
        .find(|method| after.starts_with(&format!(".{method}")))
        .map(str::to_owned)
}

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

fn is_code_position(source: &str, target: usize) -> bool {
    let mut chars = source[..target].chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    let mut block_comment = false;
    let mut line_comment = false;
    while let Some(ch) = chars.next() {
        if line_comment {
            line_comment = ch != '\n';
            continue;
        }
        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                block_comment = false;
            }
            continue;
        }
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                line_comment = true;
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                block_comment = true;
            }
            _ => {}
        }
    }
    quote.is_none() && !block_comment && !line_comment
}

fn is_identifier_boundary(source: &str, start: usize, len: usize) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[start + len..].chars().next();
    !before.is_some_and(is_identifier_char) && !after.is_some_and(is_identifier_char)
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

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
