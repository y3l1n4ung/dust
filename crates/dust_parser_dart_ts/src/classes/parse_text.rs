use dust_parser_dart::ParameterKind;
use dust_text::SourceText;
use tree_sitter::Node;

use crate::syntax::node_text;

pub(super) fn determine_parameter_kind(node: Node<'_>, source: &SourceText) -> ParameterKind {
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

pub(super) fn extract_type_prefix(declaration_text: &str, type_end: usize) -> Option<String> {
    let prefix = declaration_text.get(..type_end)?.trim();
    let stripped = strip_prefix_modifiers(prefix);
    if stripped.is_empty() || stripped == "var" {
        None
    } else {
        Some(stripped.to_owned())
    }
}

pub(super) fn extract_parameter_type(text: &str, name: &str) -> Option<String> {
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

pub(super) fn extract_default_value_source(text: &str) -> Option<String> {
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

pub(super) fn trailing_default_value_source(node: Node<'_>, source: &SourceText) -> Option<String> {
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

pub(super) fn extract_redirect_target(text: &str) -> Option<String> {
    let (_, rhs) = text.split_once('=')?;
    let target = rhs.trim().trim_end_matches(';').trim();
    if target.is_empty() {
        None
    } else {
        Some(target.to_owned())
    }
}

pub(super) fn extract_redirect_target_name(text: &str) -> Option<String> {
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
