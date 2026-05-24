use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use dust_ir::LibraryIr;
use dust_ir::ParamKind;
use dust_plugin_api::GENERATED_HEADER;
use dust_workspace::{package_import_uri, rewrite_library_import_uri};

use crate::plugin::emit::fixture::FixtureCatalog;
use crate::plugin::emit::path::{PathSegment, path_segments};
use crate::plugin::emit::test_support::{
    fallback_sample, sample_body_assertion, sample_response_data,
};
use crate::plugin::model::{ClientSpec, EndpointParam, EndpointSpec, RequestMode};
use crate::plugin::util::escape_single_quoted;

pub(crate) fn render_test_file(library: &LibraryIr, specs: &[ClientSpec<'_>]) -> Option<String> {
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

    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push_str("import 'package:dio/dio.dart';\n");
    out.push_str("import 'package:test/test.dart';\n");
    for import in rendered_imports(library, package_root, source_path) {
        if matches!(
            import.as_str(),
            "package:dio/dio.dart"
                | "package:test/test.dart"
                | "package:dust_http_client_annotation/dust_http_client_annotation.dart"
        ) {
            continue;
        }
        out.push_str(&format!("import '{}';\n", import));
    }
    out.push_str(&format!("import '{}';\n\n", source_import));
    out.push_str("void main() {\n");
    for group in client_groups {
        out.push_str(&group);
    }
    out.push_str("}\n");
    Some(out)
}

fn rendered_imports(
    library: &LibraryIr,
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

fn render_client_group(spec: &ClientSpec<'_>, fixtures: &FixtureCatalog<'_>) -> Option<String> {
    let endpoint_tests = spec
        .endpoints
        .iter()
        .filter_map(|endpoint| render_endpoint_test(spec, endpoint, fixtures))
        .collect::<Vec<_>>();
    if endpoint_tests.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str(&format!(
        "  group('{} request mapping', () {{\n",
        spec.class_name
    ));
    for endpoint in endpoint_tests {
        out.push_str(&endpoint);
    }
    out.push_str("  });\n");
    Some(out)
}

fn render_endpoint_test(
    spec: &ClientSpec<'_>,
    endpoint: &EndpointSpec<'_>,
    fixtures: &FixtureCatalog<'_>,
) -> Option<String> {
    let invocation = build_invocation(endpoint, fixtures)?;

    let mut out = String::new();
    out.push_str(&format!(
        "    test('{} {}', () async {{\n",
        endpoint.verb.as_str(),
        endpoint.method.name
    ));
    out.push_str("      RequestOptions? captured;\n");
    out.push_str("      final dio = Dio();\n");
    out.push_str("      dio.interceptors.add(\n");
    out.push_str("        InterceptorsWrapper(\n");
    out.push_str("          onRequest: (options, handler) {\n");
    out.push_str("            captured = options;\n");
    out.push_str(&format!(
        "            handler.resolve(Response<dynamic>(requestOptions: options, data: {}));\n",
        sample_response_data(&endpoint.return_spec)
    ));
    out.push_str("          },\n");
    out.push_str("        ),\n");
    out.push_str("      );\n");
    out.push_str(&format!("      final api = {}(dio);\n", spec.class_name));
    out.push_str("      try {\n");
    let invocation_expr = format!(
        "api.{}({})",
        endpoint.method.name, invocation.call_arguments
    );
    if endpoint.return_spec.is_stream() {
        out.push_str(&format!("        await {invocation_expr}.drain<void>();\n"));
    } else {
        out.push_str(&format!("        await {invocation_expr};\n"));
    }
    out.push_str("      } catch (_) {}\n");
    out.push_str("      expect(captured, isNotNull);\n");
    out.push_str("      final request = captured!;\n");
    out.push_str(&format!(
        "      expect(request.method, '{}');\n",
        endpoint.verb.as_str()
    ));
    out.push_str(&format!(
        "      expect(request.path, '{}');\n",
        escape_single_quoted(&render_expected_path(endpoint, &invocation.path_values))
    ));
    for assertion in invocation.assertions {
        out.push_str("      ");
        out.push_str(&assertion);
        out.push('\n');
    }
    out.push_str("    });\n");
    Some(out)
}

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

struct Invocation {
    call_arguments: String,
    assertions: Vec<String>,
    path_values: Vec<(String, Option<String>)>,
}
