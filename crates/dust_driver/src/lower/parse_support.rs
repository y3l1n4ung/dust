#[derive(Default)]
struct DelimiterState {
    depth_angle: u32,
    depth_paren: u32,
    depth_brace: u32,
    depth_bracket: u32,
    quote: Option<char>,
    escape: bool,
}

impl DelimiterState {
    fn is_top_level(&self) -> bool {
        self.quote.is_none()
            && self.depth_angle == 0
            && self.depth_paren == 0
            && self.depth_brace == 0
            && self.depth_bracket == 0
    }

    fn advance(&mut self, ch: char) {
        if let Some(active_quote) = self.quote {
            if self.escape {
                self.escape = false;
                return;
            }

            if ch == '\\' {
                self.escape = true;
                return;
            }

            if ch == active_quote {
                self.quote = None;
            }
            return;
        }

        match ch {
            '\'' | '"' => self.quote = Some(ch),
            '<' => self.depth_angle += 1,
            '>' => self.depth_angle = self.depth_angle.saturating_sub(1),
            '(' => self.depth_paren += 1,
            ')' => self.depth_paren = self.depth_paren.saturating_sub(1),
            '{' => self.depth_brace += 1,
            '}' => self.depth_brace = self.depth_brace.saturating_sub(1),
            '[' => self.depth_bracket += 1,
            ']' => self.depth_bracket = self.depth_bracket.saturating_sub(1),
            _ => {}
        }
    }
}

pub(crate) fn split_top_level_items(source: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut state = DelimiterState::default();
    let mut start = 0_usize;

    for (index, ch) in source.char_indices() {
        if state.is_top_level() && ch == ',' {
            items.push(source[start..index].trim());
            start = index + 1;
        }
        state.advance(ch);
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        items.push(tail);
    }

    items
}

pub(crate) fn split_top_level_args(source: &str) -> Vec<&str> {
    split_top_level_items(source)
}

pub(crate) fn split_top_level_once(source: &str, target: char) -> Option<(&str, &str)> {
    let mut state = DelimiterState::default();

    for (index, ch) in source.char_indices() {
        if state.is_top_level() && ch == target {
            return Some((&source[..index], &source[index + ch.len_utf8()..]));
        }
        state.advance(ch);
    }

    None
}

pub(crate) fn has_top_level_char(source: &str, target: char) -> bool {
    find_top_level_char(source, |_, ch| ch == target).is_some()
}

pub(crate) fn find_top_level_char(
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
