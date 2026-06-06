/// A structured constant annotation value.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationValue {
    /// A boolean literal.
    Bool(bool),
    /// A string literal with raw/quoted delimiters removed.
    String(String),
    /// A numeric literal kept as source for exact int/double interpretation.
    Number(String),
    /// A list literal.
    List(Vec<AnnotationValue>),
    /// A named record literal.
    Record(Vec<(String, AnnotationValue)>),
    /// A constant constructor invocation with named arguments.
    Constructor {
        /// Constructor/type name.
        name: String,
        /// Named argument values.
        named: Vec<(String, AnnotationValue)>,
    },
    /// A member, type, or function reference expression.
    Member(String),
    /// Any expression shape Dust should preserve but not semantically parse.
    Expression(String),
}

/// Parses top-level named annotation arguments into structured values.
pub fn parse_annotation_named_values(source: &str) -> Option<Vec<(String, AnnotationValue)>> {
    let inner = normalized_arguments(source)?;
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect()
}

fn parse_annotation_value(source: &str) -> AnnotationValue {
    let source = source.trim();
    if source == "true" {
        return AnnotationValue::Bool(true);
    }
    if source == "false" {
        return AnnotationValue::Bool(false);
    }
    if let Some(value) = parse_string_literal(source) {
        return AnnotationValue::String(value);
    }
    if is_number_literal(source) {
        return AnnotationValue::Number(source.to_owned());
    }
    if let Some(list) = parse_list(source) {
        return AnnotationValue::List(list);
    }
    if let Some(record) = parse_record(source) {
        return AnnotationValue::Record(record);
    }
    if let Some((name, named)) = parse_constructor(source) {
        return AnnotationValue::Constructor { name, named };
    }
    if is_member_ref(source) {
        return AnnotationValue::Member(source.to_owned());
    }
    AnnotationValue::Expression(source.to_owned())
}

fn parse_list(source: &str) -> Option<Vec<AnnotationValue>> {
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        split_top_level_items(inner)
            .into_iter()
            .map(parse_annotation_value)
            .collect(),
    )
}

fn parse_record(source: &str) -> Option<Vec<(String, AnnotationValue)>> {
    let inner = source.strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect()
}

fn parse_constructor(source: &str) -> Option<(String, Vec<(String, AnnotationValue)>)> {
    let source = source.strip_prefix("const ").unwrap_or(source).trim();
    let open = source.find('(')?;
    let name = source[..open].trim();
    if !is_member_ref(name) {
        return None;
    }
    let inner = source[open + 1..].strip_suffix(')')?.trim();
    if inner.is_empty() {
        return Some((name.to_owned(), Vec::new()));
    }
    let named = split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((key.trim().to_owned(), parse_annotation_value(value.trim())))
        })
        .collect::<Option<Vec<_>>>()?;
    Some((name.to_owned(), named))
}

fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let source = source
        .strip_prefix('r')
        .or_else(|| source.strip_prefix('R'))
        .unwrap_or(source);
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }
    Some(source[1..source.len() - 1].to_owned())
}

fn is_number_literal(source: &str) -> bool {
    let source = source.strip_prefix('-').unwrap_or(source);
    let mut saw_digit = false;
    let mut saw_dot = false;
    for ch in source.chars() {
        match ch {
            '0'..='9' => saw_digit = true,
            '.' if !saw_dot => saw_dot = true,
            _ => return false,
        }
    }
    saw_digit
}

fn is_member_ref(source: &str) -> bool {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '.' || ch.is_ascii_alphanumeric())
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
