use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use dust_dart_emit::render_template;
use dust_ir::DartFileIr;
use dust_ir::ParamKind;
use dust_plugin_api::GENERATED_HEADER;
use dust_workspace::{package_import_uri, rewrite_library_import_uri};
use serde::Serialize;

use crate::plugin::emit::fixture::FixtureCatalog;
use crate::plugin::emit::path::{PathSegment, path_segments};
use crate::plugin::emit::test_support::{
    fallback_sample, sample_body_assertion, sample_response_data,
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec, RequestMode};
use crate::plugin::util::escape_single_quoted;

/// Template context for a generated HTTP test file.
#[derive(Serialize)]
struct TestFileContext {
    /// Generated file header.
    header: &'static str,
    /// Imports required by the generated test.
    imports: String,
    /// Import URI for the source library under test.
    source_import: String,
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
                imports: render_imports(library, package_root, source_path),
                source_import,
                groups: format!("{}\n", client_groups.join("\n")),
            },
        )
    ))
}

/// Renders imports copied from the source library with generated-test exclusions.
fn render_imports(library: &DartFileIr, package_root: &Path, source_path: &Path) -> String {
    rendered_imports(library, package_root, source_path)
        .into_iter()
        .filter(|import| {
            !matches!(
                import.as_str(),
                "package:dio/dio.dart" | "package:test/test.dart" | "package:dust_dart/http.dart"
            )
        })
        .map(|import| format!("import '{import}';\n"))
        .collect::<String>()
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

    let invocation_expr = format!(
        "api.{}({})",
        endpoint.method.name, invocation.call_arguments
    );
    let call_line = if endpoint.return_spec.is_stream() {
        format!("        await {invocation_expr}.drain<void>();\n")
    } else {
        format!("        await {invocation_expr};\n")
    };

    Some(render_template(
        "endpoint_test",
        include_str!("templates/endpoint_test.jinja"),
        EndpointTestContext {
            verb: endpoint.verb.as_str(),
            method_name: &endpoint.method.name,
            response_data: sample_response_data(&endpoint.return_spec).to_owned(),
            class_name: spec.class_name,
            invocation: call_line,
            expected_path: escape_single_quoted(&render_expected_path(
                endpoint,
                &invocation.path_values,
            )),
            assertions: invocation
                .assertions
                .into_iter()
                .map(|assertion| format!("      {assertion}\n"))
                .collect(),
        },
    ))
}

/// Builds sample call arguments and request assertions for an endpoint.
fn build_invocation(
    endpoint: &EndpointSpec<'_>,
    fixtures: &FixtureCatalog<'_>,
) -> Option<Invocation> {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    let mut assertions = vec![
        "expect(request.queryParameters, isA<Map<String, dynamic>>());".to_owned(),
        "expect(request.headers, isA<Map<String, dynamic>>());".to_owned(),
    ];
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
                EndpointParam::Query { key, .. } => assertions.push(format!(
                    "expect(request.queryParameters['{}'], equals({value_expr}));",
                    escape_single_quoted(key)
                )),
                EndpointParam::Header { key, .. } => assertions.push(format!(
                    "expect(request.headers['{}'], equals({value_expr}));",
                    escape_single_quoted(key)
                )),
                EndpointParam::Body { .. } if endpoint.request_mode == RequestMode::Standard => {
                    assertions.push(format!(
                        "expect(request.data, equals({}));",
                        sample_body_assertion(&param.ty, &sample)
                    ));
                }
                EndpointParam::Field { key, .. }
                    if endpoint.request_mode == RequestMode::FormUrlEncoded =>
                {
                    assertions.push(format!(
                        "expect((request.data as Map<String, dynamic>)['{}'], equals({value_expr}));",
                        escape_single_quoted(key)
                    ));
                }
                _ => {}
            }
        }
    }

    for (key, value) in &endpoint.headers {
        assertions.push(format!(
            "expect(request.headers['{}'], equals('{}'));",
            escape_single_quoted(key),
            escape_single_quoted(value)
        ));
    }

    let call_arguments = match (positional.is_empty(), named.is_empty()) {
        (true, true) => String::new(),
        (false, true) => positional.join(", "),
        (true, false) => named.join(", "),
        (false, false) => format!("{}, {}", positional.join(", "), named.join(", ")),
    };

    Some(Invocation {
        call_arguments,
        assertions,
        path_values,
    })
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
    /// Comma-separated Dart call arguments.
    call_arguments: String,
    /// Request assertions generated for the fake adapter callback.
    assertions: Vec<String>,
    /// Path placeholder values captured from argument samples.
    path_values: Vec<(String, Option<String>)>,
}
