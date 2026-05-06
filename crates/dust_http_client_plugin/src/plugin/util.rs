use dust_diagnostics::SourceLabel;
use dust_ir::{SpanIr, TypeIr};

pub(super) fn config_name(symbol: &str) -> &str {
    symbol.rsplit("::").next().unwrap_or(symbol)
}

pub(super) fn escape_single_quoted(source: &str) -> String {
    source.replace('\\', "\\\\").replace('\'', "\\'")
}

pub(super) fn label(span: SpanIr, message: impl Into<String>) -> SourceLabel {
    SourceLabel::new(span.file_id, span.range, message)
}

pub(super) fn extract_path_placeholders(path: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut cursor = 0_usize;
    while let Some(start) = path[cursor..].find('{') {
        let start = cursor + start;
        let Some(end) = path[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + end;
        placeholders.push(path[start + 1..end].to_owned());
        cursor = end + 1;
    }
    placeholders
}

pub(super) fn type_name_is(ty: &TypeIr, expected: &str) -> bool {
    ty.name()
        .map(|name| name == expected || name.rsplit('.').next() == Some(expected))
        .unwrap_or(false)
}

pub(super) fn is_string_keyed_map(ty: &TypeIr) -> bool {
    type_name_is(ty, "Map") && ty.args().len() == 2 && ty.args()[0].is_named("String")
}

pub(super) fn is_response_body_type(ty: &TypeIr) -> bool {
    type_name_is(ty, "ResponseBody")
}

pub(super) fn is_list_of_int_type(ty: &TypeIr) -> bool {
    type_name_is(ty, "List") && ty.args().len() == 1 && ty.args()[0].is_builtin(dust_ir::BuiltinType::Int)
}

pub(super) fn has_import(imports: &[String], uri: &str) -> bool {
    imports.iter().any(|import| import == uri)
}

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

pub(super) fn split_top_level_items(source: &str) -> Vec<&str> {
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

pub(super) fn split_top_level_once(source: &str, target: char) -> Option<(&str, &str)> {
    let mut state = DelimiterState::default();
    for (index, ch) in source.char_indices() {
        if state.is_top_level() && ch == target {
            return Some((&source[..index], &source[index + ch.len_utf8()..]));
        }
        state.advance(ch);
    }
    None
}
