use dust_ir::{BuiltinType, LoweringOutcome, TypeIr};

pub(crate) fn lower_type(source: Option<&str>) -> LoweringOutcome<TypeIr> {
    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return LoweringOutcome::new(TypeIr::unknown());
    };

    LoweringOutcome::new(parse_type(source))
}

pub(crate) fn parse_type(source: &str) -> TypeIr {
    let (base, nullable) = strip_nullable(source);
    let base = base.trim();

    let type_ir = if base == "dynamic" {
        TypeIr::dynamic()
    } else if looks_like_function_type(base) {
        TypeIr::function(base)
    } else if looks_like_record_type(base) {
        TypeIr::record(base)
    } else if let Some(builtin) = parse_builtin(base) {
        TypeIr::builtin(builtin)
    } else if let Some((name, args)) = split_generic(base) {
        TypeIr::generic(
            name.trim(),
            split_top_level_args(args)
                .into_iter()
                .map(|arg| parse_type(arg.trim()))
                .collect(),
        )
    } else {
        TypeIr::named(base)
    };

    if nullable {
        type_ir.nullable()
    } else {
        type_ir
    }
}

pub(crate) fn split_top_level_items(source: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut quote = None;
    let mut escape = false;
    let mut start = 0_usize;

    for (index, ch) in source.char_indices() {
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
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                items.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        items.push(tail);
    }

    items
}

pub(crate) fn split_top_level_args(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut start = 0_usize;

    for (index, ch) in source.char_indices() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                args.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        args.push(tail);
    }

    args
}

fn strip_nullable(source: &str) -> (&str, bool) {
    if let Some(stripped) = source.strip_suffix('?') {
        (stripped, true)
    } else {
        (source, false)
    }
}

fn parse_builtin(source: &str) -> Option<BuiltinType> {
    match source {
        "String" => Some(BuiltinType::String),
        "int" => Some(BuiltinType::Int),
        "bool" => Some(BuiltinType::Bool),
        "double" => Some(BuiltinType::Double),
        "num" => Some(BuiltinType::Num),
        "Object" => Some(BuiltinType::Object),
        _ => None,
    }
}

fn split_generic(source: &str) -> Option<(&str, &str)> {
    let start = source.find('<')?;
    let end = source.rfind('>')?;
    if end <= start {
        return None;
    }
    Some((&source[..start], &source[start + 1..end]))
}

fn looks_like_record_type(source: &str) -> bool {
    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        return false;
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return false;
    }

    inner.starts_with('{') || has_top_level_char(inner, ',')
}

fn looks_like_function_type(source: &str) -> bool {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;

    for (index, ch) in source.char_indices() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            'F' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0
                && index > 0 =>
            {
                let tail = &source[index..];
                if let Some(stripped) = tail.strip_prefix("Function") {
                    let prev = source[..index].chars().next_back().unwrap_or_default();
                    let after = stripped.trim_start();
                    if prev.is_whitespace() && after.starts_with('(') {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

fn has_top_level_char(source: &str, target: char) -> bool {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;

    for ch in source.chars() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            _ if ch == target
                && depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                return true;
            }
            _ => {}
        }
    }

    false
}
