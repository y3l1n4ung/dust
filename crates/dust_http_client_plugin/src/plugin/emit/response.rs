use crate::plugin::emit::types::{
    is_void_type, needs_isolate_helper, render_decode_expr, render_decode_expr_nonnull,
    render_non_nullable_type, render_type,
};
use crate::plugin::model::{ClientSpec, EndpointSpec, HttpTargetMode, ParseThreadMode};
use crate::plugin::util::is_response_body_type;

/// Renders the endpoint completion statement after `_dio.fetch`.
pub(super) fn render_response_return(spec: &ClientSpec<'_>, endpoint: &EndpointSpec<'_>) -> String {
    let ty = &endpoint.return_spec.ty;
    if is_void_type(ty) {
        if endpoint.return_spec.raw_response {
            return "    return _buildResponse<void>(_result, null);\n".to_owned();
        }
        return "    return;\n".to_owned();
    }
    if is_response_body_type(ty) {
        if endpoint.return_spec.raw_response {
            return "    return _result;\n".to_owned();
        }
        if ty.is_nullable() {
            return "    return _result.data;\n".to_owned();
        }
        return "    return _result.data!;\n".to_owned();
    }

    let decode_value =
        if endpoint.parse_thread == ParseThreadMode::Isolate && needs_isolate_helper(ty) {
            let helper_name = isolate_helper_name(spec.class_name, &endpoint.method.name);
            if ty.is_nullable() {
                format!(
                    "_result.data == null ? null : {}",
                    render_background_decode(spec.target, &helper_name, "_result.data")
                )
            } else {
                render_background_decode(spec.target, &helper_name, "_result.data!")
            }
        } else {
            render_decode_expr("_result.data", ty)
        };

    if endpoint.return_spec.raw_response {
        format!(
            "    final _value = {};\n    return _buildResponse<{}>(_result, _value);\n",
            decode_value,
            render_type(ty)
        )
    } else {
        format!("    return {};\n", decode_value)
    }
}

/// Renders isolate decode helpers required by a client spec.
pub(crate) fn render_isolate_helpers(spec: &ClientSpec<'_>) -> Vec<String> {
    spec.endpoints
        .iter()
        .filter(|endpoint| {
            endpoint.parse_thread == ParseThreadMode::Isolate
                && needs_isolate_helper(&endpoint.return_spec.ty)
        })
        .map(|endpoint| render_isolate_helper(spec.class_name, endpoint))
        .collect()
}

/// Renders shared helpers used by generated HTTP clients.
pub(crate) fn render_shared_helpers() -> Vec<String> {
    vec![
        [
            render_set_stream_type_helper(),
            render_combine_base_urls_helper(),
            render_response_wrapper_helper(),
        ]
        .join("\n\n"),
    ]
}

/// Renders one top-level isolate decode helper for an endpoint.
fn render_isolate_helper(class_name: &str, endpoint: &EndpointSpec<'_>) -> String {
    format!(
        "{} {}(dynamic json) {{\n  return {};\n}}\n",
        render_non_nullable_type(&endpoint.return_spec.ty),
        isolate_helper_name(class_name, &endpoint.method.name),
        render_decode_expr_nonnull("json", &endpoint.return_spec.ty)
    )
}

/// Builds the deterministic generated isolate helper name.
fn isolate_helper_name(class_name: &str, method_name: &str) -> String {
    format!("_${}_{}_Decode", class_name, method_name)
}

/// Renders a target-specific background decode expression.
fn render_background_decode(target: HttpTargetMode, helper_name: &str, data_expr: &str) -> String {
    match target {
        HttpTargetMode::Dart => format!("await Isolate.run(() => {helper_name}({data_expr}))"),
        HttpTargetMode::Flutter => format!("await compute({helper_name}, {data_expr})"),
    }
}

/// Renders the Dio response-type adjustment helper.
fn render_set_stream_type_helper() -> &'static str {
    r#"RequestOptions _setStreamType<T>(RequestOptions requestOptions) {
  if (T != dynamic &&
      requestOptions.responseType != ResponseType.bytes &&
      requestOptions.responseType != ResponseType.stream) {
    if (T == ResponseBody) {
      requestOptions.responseType = ResponseType.stream;
    } else if (T == String) {
      requestOptions.responseType = ResponseType.plain;
    } else {
      requestOptions.responseType = ResponseType.json;
    }
  }
  return requestOptions;
}"#
}

/// Renders the helper that combines Dio and annotation base URLs.
fn render_combine_base_urls_helper() -> &'static str {
    r#"String _combineBaseUrls(String dioBaseUrl, String? baseUrl) {
  if (baseUrl == null || baseUrl.trim().isEmpty) {
    return dioBaseUrl;
  }
  final url = Uri.parse(baseUrl);
  if (url.isAbsolute) {
    return url.toString();
  }
  return Uri.parse(dioBaseUrl).resolveUri(url).toString();
}"#
}

/// Renders the helper that rebuilds `Response<T>` around decoded data.
fn render_response_wrapper_helper() -> &'static str {
    r#"Response<T> _buildResponse<T>(Response<dynamic> response, T data) {
  return Response<T>(
    data: data,
    headers: response.headers,
    isRedirect: response.isRedirect,
    redirects: response.redirects,
    requestOptions: response.requestOptions,
    statusCode: response.statusCode,
    statusMessage: response.statusMessage,
    extra: response.extra,
  );
}"#
}
