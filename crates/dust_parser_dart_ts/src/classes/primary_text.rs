use dust_text::{TextRange, TextSize};

pub(super) fn find_keyword(text: &str, keyword: &str, from: usize) -> Option<usize> {
    let mut search_from = from;
    while let Some(relative) = text.get(search_from..)?.find(keyword) {
        let index = search_from + relative;
        if is_start_boundary(text, index) && is_end_boundary(text, index + keyword.len()) {
            return Some(index);
        }
        search_from = index + keyword.len();
    }
    None
}

fn is_start_boundary(text: &str, index: usize) -> bool {
    text.get(..index)
        .and_then(|head| head.chars().next_back())
        .is_none_or(|ch| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
}

fn is_end_boundary(text: &str, index: usize) -> bool {
    text.get(index..)
        .and_then(|tail| tail.chars().next())
        .is_none_or(|ch| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
}

pub(super) fn read_identifier(text: &str, index: &mut usize) -> Option<String> {
    let mut end = *index;
    for (relative, ch) in text.get(*index..)?.char_indices() {
        if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
            end = *index + relative + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > *index).then(|| {
        let value = text[*index..end].to_owned();
        *index = end;
        value
    })
}

pub(super) fn skip_type_arguments(text: &str, index: usize) -> usize {
    if text.get(index..).is_some_and(|tail| tail.starts_with('<')) {
        find_matching(text, index, '<', '>').map_or(index, |end| end + 1)
    } else {
        index
    }
}

pub(super) fn find_matching(
    text: &str,
    open_index: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0_u32;
    let mut quote = None;
    let mut escape = false;
    for (relative, ch) in text.get(open_index..)?.char_indices() {
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
            current if current == open => depth += 1,
            current if current == close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_index + relative);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn split_top_level(text: &str, delimiter: char) -> Vec<(usize, usize)> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0_i32;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            current if current == delimiter && depth == 0 => {
                parts.push((start, index));
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push((start, text.len()));
    parts
}

pub(super) fn split_default(text: &str) -> Option<(&str, &str)> {
    text.split_once('=')
}

pub(super) fn skip_ws(text: &str, mut index: usize) -> usize {
    while text
        .get(index..)
        .and_then(|tail| tail.chars().next())
        .is_some_and(char::is_whitespace)
    {
        index += text[index..].chars().next().map_or(0, char::len_utf8);
    }
    index
}

pub(super) fn has_word(text: &str, word: &str) -> bool {
    text.split_whitespace().any(|part| part == word)
}

pub(super) fn range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32))
}
