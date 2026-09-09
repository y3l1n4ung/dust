//! Building the call a generated test makes, and the assertions around it.

use super::*;

/// Builds sample call arguments and request assertions for an endpoint.
pub(super) fn build_invocation(
    endpoint: &EndpointSpec<'_>,
    fixtures: &FixtureCatalog<'_>,
) -> Option<Invocation> {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    let mut query_entries = Vec::new();
    let mut header_entries = endpoint
        .headers
        .iter()
        .map(|(key, value)| map_entry(key, &format!("'{}'", escape_single_quoted(value))))
        .collect::<Vec<_>>();
    let mut extra_entries = Vec::new();
    let mut body_entries = Vec::new();
    let mut data_assertion = "expect(request.data, isNull);".to_owned();
    let mut path_values = Vec::new();

    for param in &endpoint.method.params {
        let binding = endpoint.binding_for_param(&param.name);
        let sample = fixtures
            .sample_value(&param.ty)
            .or_else(|| binding.and_then(fallback_sample))?;
        let value_expr = sample.assertion_expression.clone();
        if param.kind == ParamKind::Named {
            named.push(format!("{}: {}", param.name, sample.expression));
        } else {
            positional.push(sample.expression.clone());
        }

        if let Some(binding) = binding {
            match binding {
                EndpointParam::Path { key, .. } => {
                    path_values.push((key.clone(), sample.path_value))
                }
                EndpointParam::Query { key, .. } if !sample_is_null(&sample) => {
                    query_entries.push(map_entry(key, &value_expr));
                }
                EndpointParam::Queries { .. } if !sample_is_null(&sample) => {
                    query_entries.push(format!("...{}", sample.assertion_expression));
                }
                EndpointParam::Header { key, .. } if !sample_is_null(&sample) => {
                    header_entries
                        .push(map_entry(key, &format!("{}.toString()", sample.expression)));
                }
                EndpointParam::HeaderMap { .. } if !sample_is_null(&sample) => {
                    header_entries.push(format!("...{}", sample.assertion_expression));
                }
                EndpointParam::Extra { key, .. } if !sample_is_null(&sample) => {
                    extra_entries.push(map_entry(key, &value_expr));
                }
                EndpointParam::Body { .. } if endpoint.request_mode == RequestMode::Standard => {
                    data_assertion = format!(
                        "expect(request.data, equals({}));",
                        sample_body_assertion(&param.ty, &sample)
                    );
                }
                EndpointParam::Field { key, .. }
                    if endpoint.request_mode == RequestMode::FormUrlEncoded
                        && !sample_is_null(&sample) =>
                {
                    body_entries.push(map_entry(key, &value_expr));
                }
                EndpointParam::Part { .. } if endpoint.request_mode == RequestMode::MultiPart => {
                    data_assertion = "expect(request.data, isA<FormData>());".to_owned();
                }
                _ => {}
            }
        }
    }

    if endpoint.request_mode == RequestMode::FormUrlEncoded {
        data_assertion = map_assertion("request.data", &body_entries);
    }

    let mut assertions = vec![
        map_assertion("request.queryParameters", &query_entries),
        map_assertion(
            "Map<String, dynamic>.from(request.headers)..remove('content-type')",
            &header_entries,
        ),
        map_assertion("request.extra", &extra_entries),
        data_assertion,
    ];
    match endpoint.request_mode {
        RequestMode::Standard => {}
        RequestMode::FormUrlEncoded => assertions
            .push("expect(request.contentType, Headers.formUrlEncodedContentType);".to_owned()),
        RequestMode::MultiPart => assertions
            .push("expect(request.contentType, Headers.multipartFormDataContentType);".to_owned()),
    }

    let mut arguments = positional;
    arguments.extend(named);

    Some(Invocation {
        arguments,
        assertions,
        path_values,
    })
}

/// Renders an API call expression, wrapping long argument lists.
pub(super) fn render_call_expression(method_name: &str, arguments: &[String]) -> String {
    let one_line = format!("api.{method_name}({})", arguments.join(", "));
    if one_line.len() <= 72 {
        return one_line;
    }

    let args = arguments
        .iter()
        .map(|argument| format!("        {argument},\n"))
        .collect::<String>();
    format!("api.{method_name}(\n{args}      )")
}

/// Renders an expected throwing API call with stable wrapping.
pub(super) fn render_expected_throw(invocation_expr: &str) -> String {
    let one_line = format!("      await expectLater({invocation_expr}, throwsA(anything));");
    if one_line.len() <= 88 && !invocation_expr.contains('\n') {
        return one_line;
    }

    let lines = invocation_expr.lines().collect::<Vec<_>>();
    let invocation = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let indent = if index == 0 { "        " } else { "  " };
            let suffix = if index + 1 == lines.len() { "," } else { "" };
            format!("{indent}{line}{suffix}\n")
        })
        .collect::<String>();
    format!("      await expectLater(\n{invocation}        throwsA(anything),\n      );")
}

/// Returns true when a generated sample represents a nullable omitted value.
pub(super) fn sample_is_null(sample: &SampleValue) -> bool {
    sample.assertion_expression == "null"
}

/// Renders one Dart map entry for an exact generated request assertion.
pub(super) fn map_entry(key: &str, value: &str) -> String {
    format!("'{}': {value}", escape_single_quoted(key))
}

/// Renders a readable exact map assertion for generated request tests.
pub(super) fn map_assertion(target: &str, entries: &[String]) -> String {
    if entries.is_empty() {
        return format!("expect({target}, equals(const <String, dynamic>{{}}));");
    }

    let body = entries
        .iter()
        .map(|entry| format!("    {entry},\n"))
        .collect::<String>();
    format!("expect(\n  {target},\n  equals(<String, dynamic>{{\n{body}  }}),\n);")
}

/// Applies the template indentation to each line in a rendered block.
pub(super) fn indent_block(block: &str, indent: &str) -> String {
    block
        .lines()
        .map(|line| format!("{indent}{line}\n"))
        .collect()
}

/// Renders the expected request path for a generated endpoint test.
pub(super) fn render_expected_path(
    endpoint: &EndpointSpec<'_>,
    values: &[(String, Option<String>)],
) -> String {
    let path_values = values
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_deref()))
        .collect::<BTreeMap<_, _>>();
    let mut path = String::new();

    for segment in path_segments(endpoint) {
        match segment {
            PathSegment::Literal(value) => path.push_str(value),
            PathSegment::Binding { key, .. } => match path_values.get(key).copied().flatten() {
                Some(value) => path.push_str(value),
                None => {
                    path.push('{');
                    path.push_str(key);
                    path.push('}');
                }
            },
        }
    }

    path
}

/// Generated endpoint invocation data used by test rendering.
pub(super) struct Invocation {
    /// Dart call arguments in declaration order.
    pub(super) arguments: Vec<String>,
    /// Request assertions generated for the fake adapter callback.
    pub(super) assertions: Vec<String>,
    /// Path placeholder values captured from argument samples.
    pub(super) path_values: Vec<(String, Option<String>)>,
}
