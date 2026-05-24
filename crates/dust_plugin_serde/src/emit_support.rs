pub(crate) fn format_prefixed_expr(
    indent: usize,
    prefix: &str,
    expr: &str,
    suffix: &str,
) -> String {
    let pad = " ".repeat(indent);
    let continuation = " ".repeat(indent + 4);
    let mut lines = expr.lines();
    let Some(first) = lines.next() else {
        return format!("{pad}{prefix}{suffix}");
    };

    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        let candidate = format!("{pad}{prefix}{first}{suffix}");
        if candidate.len() <= 80 {
            return candidate;
        }
        if let Some(wrapped) = wrap_call_expr(first, indent, prefix, suffix) {
            return wrapped;
        }
        return candidate;
    }

    let common_indent = rest
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| *ch == ' ').count())
        .min()
        .unwrap_or(0);
    let mut rendered = Vec::with_capacity(rest.len() + 1);
    rendered.push(format!("{pad}{prefix}{first}"));
    for (index, line) in rest.iter().enumerate() {
        let tail = if index + 1 == rest.len() { suffix } else { "" };
        let stripped = line
            .get(common_indent..)
            .unwrap_or_else(|| line.trim_start());
        rendered.push(format!("{continuation}{stripped}{tail}"));
    }
    rendered.join("\n")
}

fn wrap_call_expr(expr: &str, indent: usize, prefix: &str, suffix: &str) -> Option<String> {
    let open = expr.find('(')?;
    let close = expr.rfind(')')?;
    if close + 1 != expr.len() {
        return None;
    }
    let callee = &expr[..open];
    let args = split_top_level_args(&expr[open + 1..close])?;
    if args.len() <= 1 {
        return None;
    }

    let pad = " ".repeat(indent);
    let arg_pad = " ".repeat(indent + 2);
    let mut rendered = vec![format!("{pad}{prefix}{callee}(")];
    rendered.extend(args.into_iter().map(|arg| format!("{arg_pad}{arg},")));
    rendered.push(format!("{pad}){suffix}"));
    Some(rendered.join("\n"))
}

fn split_top_level_args(source: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
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
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
        if depth < 0 {
            return None;
        }
    }

    if quote.is_some() || depth != 0 {
        return None;
    }
    args.push(source[start..].trim());
    Some(args.into_iter().filter(|arg| !arg.is_empty()).collect())
}

pub(crate) fn render_deserialize_helpers() -> &'static str {
    r#"Never _jsonTypeError(Object? value, String key, String expected) =>
    throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) =>
    value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(
  Object? value,
  String key,
  String expected,
  T? Function(String value) parse,
) =>
    parse(_jsonAs<String>(value, key, 'String')) ??
    _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) =>
    _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}

DateTime _jsonAsDateTime(Object? value, String key) =>
    _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) =>
    _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) =>
    _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _jsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}"#
}
