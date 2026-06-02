use crate::surface::ParsedAnnotation;

impl ParsedAnnotation {
    /// Returns the annotation argument list without the outer parentheses.
    pub fn normalized_arguments(&self) -> Option<&str> {
        normalized_arguments(self.arguments_source.as_deref()?)
    }

    /// Returns one top-level positional argument source by index.
    pub fn positional_argument_source(&self, index: usize) -> Option<&str> {
        self.argument_items()
            .into_iter()
            .filter(|item| split_top_level_once(item, ':').is_none())
            .nth(index)
    }

    /// Returns one top-level named argument source by name.
    pub fn named_argument_source(&self, name: &str) -> Option<&str> {
        self.named_arguments()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }

    /// Returns whether a top-level named argument is present.
    pub fn has_named_argument(&self, name: &str) -> bool {
        self.named_argument_source(name).is_some()
    }

    /// Returns all top-level annotation argument items.
    pub fn argument_items(&self) -> Vec<&str> {
        self.normalized_arguments()
            .map(split_top_level_items)
            .unwrap_or_default()
    }

    /// Returns all top-level named annotation arguments as key/source pairs.
    pub fn named_arguments(&self) -> Vec<(&str, &str)> {
        self.argument_items()
            .into_iter()
            .filter_map(|item| {
                let (key, value) = split_top_level_once(item, ':')?;
                Some((key.trim(), value.trim()))
            })
            .collect()
    }
}

fn normalized_arguments(source: &str) -> Option<&str> {
    source
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')
        .map(str::trim)
}

fn split_top_level_items(source: &str) -> Vec<&str> {
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

fn split_top_level_once(source: &str, target: char) -> Option<(&str, &str)> {
    let mut state = DelimiterState::default();
    for (index, ch) in source.char_indices() {
        if state.is_top_level() && ch == target {
            return Some((&source[..index], &source[index + ch.len_utf8()..]));
        }
        state.advance(ch);
    }
    None
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
            '<' => self.angle += 1,
            '>' => self.angle = self.angle.saturating_sub(1),
            '(' => self.paren += 1,
            ')' => self.paren = self.paren.saturating_sub(1),
            '{' => self.brace += 1,
            '}' => self.brace = self.brace.saturating_sub(1),
            '[' => self.bracket += 1,
            ']' => self.bracket = self.bracket.saturating_sub(1),
            _ => {}
        }
    }
}
