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
    r#"Never _dustJsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _dustJsonAs<T>(Object? value, String key, String expected) => value is T ? value : _dustJsonTypeError(value, key, expected);
T _dustJsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_dustJsonAs<String>(value, key, 'String')) ?? _dustJsonTypeError(value, key, expected);
List<Object?> _dustJsonAsList(Object? value, String key) => _dustJsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _dustJsonAsMap(Object? value, String key) {
  final map = _dustJsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _dustJsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _dustJsonAsDateTime(Object? value, String key) => _dustJsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _dustJsonAsUri(Object? value, String key) => _dustJsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _dustJsonAsBigInt(Object? value, String key) => _dustJsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _dustJsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
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
