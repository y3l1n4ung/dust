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

    /// Returns one top-level positional string literal by index.
    pub fn positional_string(&self, index: usize) -> Option<String> {
        parse_string_literal(self.positional_argument_source(index)?)
    }

    /// Returns one top-level positional string map literal by index.
    pub fn positional_string_map(&self, index: usize) -> Option<Vec<(String, String)>> {
        parse_string_map(self.positional_argument_source(index)?)
    }

    /// Returns one top-level named argument source by name.
    pub fn named_argument_source(&self, name: &str) -> Option<&str> {
        self.named_arguments()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }

    /// Returns one top-level named argument as expression source.
    pub fn named_expression_source(&self, name: &str) -> Option<String> {
        self.named_argument_source(name)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    /// Returns one top-level named string literal.
    pub fn named_string(&self, name: &str) -> Option<String> {
        parse_string_literal(self.named_argument_source(name)?)
    }

    /// Returns one top-level named list of string literals.
    pub fn named_string_list(&self, name: &str) -> Option<Vec<String>> {
        parse_string_list(self.named_argument_source(name)?)
    }

    /// Returns one top-level named boolean literal.
    pub fn named_bool(&self, name: &str) -> Option<bool> {
        parse_bool_literal(self.named_argument_source(name)?)
    }

    /// Returns one top-level named member or type reference.
    pub fn named_member(&self, name: &str) -> Option<String> {
        parse_type_name(self.named_argument_source(name)?)
    }

    /// Returns one top-level named type reference.
    pub fn named_type(&self, name: &str) -> Option<String> {
        self.named_member(name)
    }

    /// Returns one top-level positional type reference.
    pub fn positional_type(&self, index: usize) -> Option<String> {
        parse_type_name(self.positional_argument_source(index)?)
    }

    /// Returns one top-level named list of type references.
    pub fn named_type_list(&self, name: &str) -> Option<Vec<String>> {
        parse_type_list(self.named_argument_source(name)?)
    }

    /// Returns one top-level named string map literal.
    pub fn named_string_map(&self, name: &str) -> Option<Vec<(String, String)>> {
        parse_string_map(self.named_argument_source(name)?)
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

fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }
    Some(source[1..source.len() - 1].to_owned())
}

fn parse_string_list(source: &str) -> Option<Vec<String>> {
    let inner = source.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(parse_string_literal)
        .collect()
}

fn parse_bool_literal(source: &str) -> Option<bool> {
    match source.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_type_name(source: &str) -> Option<String> {
    let ident = source
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
        .collect::<String>();
    (!ident.is_empty()).then_some(ident)
}

fn parse_type_list(source: &str) -> Option<Vec<String>> {
    let inner = source.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    Some(
        split_top_level_items(inner)
            .into_iter()
            .filter_map(parse_type_name)
            .collect(),
    )
}

fn parse_string_map(source: &str) -> Option<Vec<(String, String)>> {
    let inner = source.trim().strip_prefix('{')?.strip_suffix('}')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((
                parse_string_literal(key.trim())?,
                parse_string_literal(value.trim())?,
            ))
        })
        .collect()
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
