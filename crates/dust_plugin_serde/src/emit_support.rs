pub(crate) fn format_prefixed_expr(
    indent: usize,
    prefix: &str,
    expr: &str,
    suffix: &str,
) -> String {
    let pad = " ".repeat(indent);
    let continuation = " ".repeat(indent + prefix.len());
    let mut lines = expr.lines();
    let Some(first) = lines.next() else {
        return format!("{pad}{prefix}{suffix}");
    };

    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        return format!("{pad}{prefix}{first}{suffix}");
    }

    let mut rendered = Vec::with_capacity(rest.len() + 1);
    rendered.push(format!("{pad}{prefix}{first}"));
    for (index, line) in rest.iter().enumerate() {
        let tail = if index + 1 == rest.len() { suffix } else { "" };
        rendered.push(format!("{continuation}{line}{tail}"));
    }
    rendered.join("\n")
}

pub(crate) fn render_deserialize_helpers() -> &'static str {
    r#"Never _jsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) => value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_jsonAs<String>(value, key, 'String')) ?? _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) => _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _jsonAsDateTime(Object? value, String key) => _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) => _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) => _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
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
