use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use dust_dart_emit::{DART_LIST, DART_VOID, render_template};
use dust_ir::DartFileIr;
use dust_ir::ParamKind;
use dust_ir::TypeIr;
use dust_plugin_api::GENERATED_HEADER;
use dust_workspace::{package_import_uri, rewrite_library_import_uri};
use serde::Serialize;

use crate::plugin::emit::fixture::FixtureCatalog;
use crate::plugin::emit::path::{PathSegment, path_segments};
use crate::plugin::emit::test_support::{
    SampleValue, fallback_sample, sample_body_assertion, sample_response_data,
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec, RequestMode};
use crate::plugin::util::{escape_single_quoted, is_response_body_type};

/// Template context for a generated HTTP test file.
#[derive(Serialize)]
struct TestFileContext {
    /// Generated file header.
    header: &'static str,
    /// Imports required by the generated test.
    imports: String,
    /// Rendered test groups.
    groups: String,
}

/// Template context for one generated client test group.
#[derive(Serialize)]
struct TestGroupContext<'a> {
    /// Source HTTP client class name.
    class_name: &'a str,
    /// Rendered endpoint tests in the group.
    tests: String,
}

/// Template context for one generated endpoint test.
#[derive(Serialize)]
struct EndpointTestContext<'a> {
    /// HTTP verb expected by the fake adapter.
    verb: &'a str,
    /// Source method name under test.
    method_name: &'a str,
    /// Fake response data expression.
    response_data: String,
    /// Source HTTP client class name.
    class_name: &'a str,
    /// Rendered API invocation.
    invocation: String,
    /// Expected request path after placeholder substitution.
    expected_path: String,
    /// Rendered request assertions.
    assertions: String,
}

/// Renders an auxiliary generated test file for clients with `generateTest`.
pub(crate) fn render_test_file(library: &DartFileIr, specs: &[ClientSpec<'_>]) -> Option<String> {
    let package_root = Path::new(&library.package_root);
    let source_path = Path::new(&library.source_path);
    let source_import =
        package_import_uri(&library.package_name, package_root, source_path).ok()?;
    let fixtures = FixtureCatalog::from_library(library);
    let client_groups = specs
        .iter()
        .filter_map(|spec| render_client_group(spec, &fixtures))
        .collect::<Vec<_>>();
    if client_groups.is_empty() {
        return None;
    }

    Some(format!(
        "{}\n",
        render_template(
            "test_file",
            include_str!("templates/test_file.jinja"),
            TestFileContext {
                header: GENERATED_HEADER,
                imports: render_imports(library, package_root, source_path, &source_import),
                groups: format!("{}\n", client_groups.join("\n")),
            },
        )
    ))
}

/// Renders imports copied from the source library with generated-test exclusions.
fn render_imports(
    library: &DartFileIr,
    package_root: &Path,
    source_path: &Path,
    source_import: &str,
) -> String {
    let imports = rendered_imports(library, package_root, source_path)
        .into_iter()
        .filter(|import| {
            !matches!(
                import.as_str(),
                "package:dio/dio.dart" | "package:test/test.dart" | "package:dust_dart/http.dart"
            )
        })
        .chain([
            "package:dio/dio.dart".to_owned(),
            "package:test/test.dart".to_owned(),
            source_import.to_owned(),
        ])
        .collect::<BTreeSet<_>>();

    let dart = imports
        .iter()
        .filter(|import| import.starts_with("dart:"))
        .map(|import| format!("import '{import}';\n"))
        .collect::<String>();
    let package = imports
        .iter()
        .filter(|import| import.starts_with("package:"))
        .map(|import| format!("import '{import}';\n"))
        .collect::<String>();
    let other = imports
        .iter()
        .filter(|import| !import.starts_with("dart:") && !import.starts_with("package:"))
        .map(|import| format!("import '{import}';\n"))
        .collect::<String>();

    [dart, package, other]
        .into_iter()
        .filter(|group| !group.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

/// Rewrites source imports so generated tests can import the same dependencies.
fn rendered_imports(
    library: &DartFileIr,
    package_root: &Path,
    source_path: &Path,
) -> BTreeSet<String> {
    library
        .imports
        .iter()
        .map(|import| {
            rewrite_library_import_uri(&library.package_name, package_root, source_path, import)
                .unwrap_or_else(|_| import.clone())
        })
        .collect()
}

/// Renders one generated test group for a client spec.
fn render_client_group(spec: &ClientSpec<'_>, fixtures: &FixtureCatalog<'_>) -> Option<String> {
    let endpoint_tests = spec
        .endpoints
        .iter()
        .filter_map(|endpoint| render_endpoint_test(spec, endpoint, fixtures))
        .collect::<Vec<_>>();
    if endpoint_tests.is_empty() {
        return None;
    }

    Some(render_template(
        "test_group",
        include_str!("templates/test_group.jinja"),
        TestGroupContext {
            class_name: spec.class_name,
            tests: format!("{}\n", endpoint_tests.join("\n")),
        },
    ))
}

/// Renders one generated endpoint test, including fake response and assertions.
fn render_endpoint_test(
    spec: &ClientSpec<'_>,
    endpoint: &EndpointSpec<'_>,
    fixtures: &FixtureCatalog<'_>,
) -> Option<String> {
    let invocation = build_invocation(endpoint, fixtures)?;

    let invocation_expr = render_call_expression(&endpoint.method.name, &invocation.arguments);
    let response = response_data(endpoint, fixtures);
    let call_line = if endpoint.return_spec.is_stream() {
        format!("      await {invocation_expr}.drain<void>();")
    } else if response.completes {
        format!("      await {invocation_expr};")
    } else {
        render_expected_throw(&invocation_expr)
    };

    Some(render_template(
        "endpoint_test",
        include_str!("templates/endpoint_test.jinja"),
        EndpointTestContext {
            verb: endpoint.verb.as_str(),
            method_name: &endpoint.method.name,
            response_data: response.source,
            class_name: spec.class_name,
            invocation: call_line,
            expected_path: escape_single_quoted(&render_expected_path(
                endpoint,
                &invocation.path_values,
            )),
            assertions: invocation
                .assertions
                .into_iter()
                .map(|assertion| indent_block(&assertion, "      "))
                .collect(),
        },
    ))
}

/// Rendered fake response data plus whether the generated call should complete.
struct ResponseFixture {
    /// Dart expression passed to the fake response.
    source: String,
    /// Whether the endpoint can decode the fake response successfully.
    completes: bool,
}

/// Builds fake response data for a generated request-mapping test.
fn response_data(endpoint: &EndpointSpec<'_>, fixtures: &FixtureCatalog<'_>) -> ResponseFixture {
    if endpoint.return_spec.is_stream() {
        return ResponseFixture {
            source: sample_response_data(&endpoint.return_spec).to_owned(),
            completes: true,
        };
    }

    if let Some(value) = fixtures.json_value(&endpoint.return_spec.ty) {
        return ResponseFixture {
            source: value,
            completes: true,
        };
    }

    ResponseFixture {
        source: sample_response_data(&endpoint.return_spec).to_owned(),
        completes: fallback_response_completes(&endpoint.return_spec.ty),
    }
}

/// Returns true when fallback response data is enough for the generated decoder.
fn fallback_response_completes(ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown | TypeIr::Builtin { .. } => true,
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST && args.len() == 1 => true,
        TypeIr::Named { name, .. } if name.as_ref() == DART_VOID => true,
        TypeIr::Named { .. } if is_response_body_type(ty) => true,
        _ => false,
    }
}

/// Builds sample call arguments and request assertions for an endpoint.
fn build_invocation(
    endpoint: &EndpointSpec<'_>,
    fixtures: &FixtureCatalog<'_>,
) -> Option<Invocation> {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    let mut query_entries = Vec::new();
    let mut header_entries = Vec::new();
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
                    header_entries.push(map_entry(key, &value_expr));
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

    for (key, value) in &endpoint.headers {
        header_entries.push(map_entry(
            key,
            &format!("'{}'", escape_single_quoted(value)),
        ));
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
fn render_call_expression(method_name: &str, arguments: &[String]) -> String {
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
fn render_expected_throw(invocation_expr: &str) -> String {
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
fn sample_is_null(sample: &SampleValue) -> bool {
    sample.assertion_expression == "null"
}

/// Renders one Dart map entry for an exact generated request assertion.
fn map_entry(key: &str, value: &str) -> String {
    format!("'{}': {value}", escape_single_quoted(key))
}

/// Renders a readable exact map assertion for generated request tests.
fn map_assertion(target: &str, entries: &[String]) -> String {
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
fn indent_block(block: &str, indent: &str) -> String {
    block
        .lines()
        .map(|line| format!("{indent}{line}\n"))
        .collect()
}

/// Renders the expected request path for a generated endpoint test.
fn render_expected_path(
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
struct Invocation {
    /// Dart call arguments in declaration order.
    arguments: Vec<String>,
    /// Request assertions generated for the fake adapter callback.
    assertions: Vec<String>,
    /// Path placeholder values captured from argument samples.
    path_values: Vec<(String, Option<String>)>,
}
