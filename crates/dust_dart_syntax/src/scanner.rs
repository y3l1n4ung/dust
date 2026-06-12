/// Splits a Dart argument/list source string at top-level commas.
pub fn split_top_level_items(source: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut state = DelimiterState::default();
    let mut start = 0_usize;
    for (index, ch) in source.char_indices() {
        if state.is_top_level() && ch == ',' {
            let item = source[start..index].trim();
            if !item.is_empty() {
                items.push(item);
            }
            start = index + ch.len_utf8();
        }
        state.advance(ch);
    }
    let tail = source[start..].trim();
    if !tail.is_empty() {
        items.push(tail);
    }
    items
}

/// Splits a Dart source string once at a top-level target character.
pub fn split_top_level_once(source: &str, target: char) -> Option<(&str, &str)> {
    let index = find_top_level_char(source, |_, ch| ch == target)?;
    Some((&source[..index], &source[index + target.len_utf8()..]))
}

/// Returns `true` if `source` has `target` at top level.
pub fn has_top_level_char(source: &str, target: char) -> bool {
    find_top_level_char(source, |_, ch| ch == target).is_some()
}

/// Finds the first top-level character matching `predicate`.
pub fn find_top_level_char(
    source: &str,
    mut predicate: impl FnMut(usize, char) -> bool,
) -> Option<usize> {
    let mut state = DelimiterState::default();
    for (index, ch) in source.char_indices() {
        if state.is_top_level() && predicate(index, ch) {
            return Some(index);
        }
        state.advance(ch);
    }
    None
}

/// Returns the first balanced parenthesized source segment from `source`.
pub fn balanced_parenthesized(source: &str) -> Option<&str> {
    let mut depth = 0_u32;
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
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
                    return Some(&source[..=index]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parses parenthesized named arguments into key/source pairs.
pub fn parse_named_arguments(source: Option<&str>) -> Vec<(&str, &str)> {
    let Some(inner) = source.and_then(normalized_args) else {
        return Vec::new();
    };
    split_top_level_items(inner)
        .into_iter()
        .filter_map(|item| split_top_level_once(item, ':'))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect()
}

/// Strips one parenthesized argument list.
pub fn normalized_args(source: &str) -> Option<&str> {
    source
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')
        .map(str::trim)
}

#[derive(Default)]
struct DelimiterState {
    paren: u32,
    bracket: u32,
    brace: u32,
    angle: u32,
    quote: Option<char>,
    escaped: bool,
}

impl DelimiterState {
    fn is_top_level(&self) -> bool {
        self.paren == 0
            && self.bracket == 0
            && self.brace == 0
            && self.angle == 0
            && self.quote.is_none()
    }

    fn advance(&mut self, ch: char) {
        if let Some(active) = self.quote {
            if self.escaped {
                self.escaped = false;
            } else if ch == '\\' {
                self.escaped = true;
            } else if ch == active {
                self.quote = None;
            }
            return;
        }
        match ch {
            '\'' | '"' => self.quote = Some(ch),
            '(' => self.paren += 1,
            ')' => self.paren = self.paren.saturating_sub(1),
            '[' => self.bracket += 1,
            ']' => self.bracket = self.bracket.saturating_sub(1),
            '{' => self.brace += 1,
            '}' => self.brace = self.brace.saturating_sub(1),
            '<' => self.angle += 1,
            '>' => self.angle = self.angle.saturating_sub(1),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{balanced_parenthesized, parse_named_arguments, split_top_level_items};

    #[test]
    fn splits_nested_items() {
        assert_eq!(
            split_top_level_items("a, const Codec<int>(), ['x,y'], ({String name})"),
            vec!["a", "const Codec<int>()", "['x,y']", "({String name})"]
        );
    }

    #[test]
    fn parses_named_arguments() {
        assert_eq!(
            parse_named_arguments(Some("(rename: 'x', tryFrom: const Codec())")),
            vec![("rename", "'x'"), ("tryFrom", "const Codec()")]
        );
    }

    #[test]
    fn finds_balanced_parentheses() {
        assert_eq!(balanced_parenthesized("('a,b'), tail"), Some("('a,b')"));
    }
}
