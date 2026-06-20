use dust_dart_emit::{DART_LIST, DART_VOID};
use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::emit::types::uses_direct_body_value;
use crate::plugin::model::{EndpointParam, ReturnSpec};
use crate::plugin::util::{is_response_body_type, type_name_is};

/// Generated test sample value and optional path-substitution value.
pub(super) struct SampleValue {
    /// Dart expression used at the call site.
    pub(super) expression: String,
    /// Dart expression used in generated request assertions.
    pub(super) assertion_expression: String,
    /// Optional URI-safe value expected in a rendered path.
    pub(super) path_value: Option<String>,
}

impl SampleValue {
    /// Creates a sample whose assertion expression matches the call expression.
    pub(super) fn new(expression: &str, path_value: Option<&str>) -> Self {
        Self {
            expression: expression.to_owned(),
            assertion_expression: expression.to_owned(),
            path_value: path_value.map(str::to_owned),
        }
    }
}

/// Returns generated response data for a fake Dio adapter response.
pub(super) fn sample_response_data(return_spec: &ReturnSpec) -> &'static str {
    if return_spec.is_stream() {
        return "ResponseBody.fromString('{}', 200)";
    }
    match &return_spec.ty {
        TypeIr::Dynamic | TypeIr::Unknown => "const <String, dynamic>{}",
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => "'ok'",
            BuiltinType::Int => "1",
            BuiltinType::Bool => "true",
            BuiltinType::Double | BuiltinType::Num => "1",
            BuiltinType::Object => "const <String, dynamic>{}",
        },
        TypeIr::Named { name, .. } if name.as_ref() == DART_VOID => "null",
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST && args.len() == 1 => {
            "const <dynamic>[]"
        }
        TypeIr::Named { .. } if is_response_body_type(&return_spec.ty) => {
            "ResponseBody.fromString('{}', 200)"
        }
        TypeIr::Named { .. } => "const <String, dynamic>{}",
        TypeIr::Function { .. } | TypeIr::Record { .. } => "null",
    }
}

/// Returns fallback samples for HTTP bindings that are not ordinary model values.
pub(super) fn fallback_sample(binding: &EndpointParam<'_>) -> Option<SampleValue> {
    Some(match binding {
        EndpointParam::Body { .. } => return None,
        EndpointParam::Queries { .. } | EndpointParam::HeaderMap { .. } => {
            SampleValue::new("const <String, dynamic>{'value': 'dust'} as dynamic", None)
        }
        EndpointParam::Part { param, .. } if type_name_is(&param.ty, "MultipartFile") => {
            SampleValue::new(
                "MultipartFile.fromBytes(<int>[1, 2, 3], filename: 'dust.txt')",
                None,
            )
        }
        EndpointParam::Part { .. } => {
            SampleValue::new("const <String, dynamic>{'value': 'dust'} as dynamic", None)
        }
        EndpointParam::CancelToken { .. } => SampleValue::new("CancelToken()", None),
        EndpointParam::Options { .. } => SampleValue::new("Options()", None),
        EndpointParam::OnSendProgress { .. } | EndpointParam::OnReceiveProgress { .. } => {
            SampleValue::new("(_, __) {}", None)
        }
        _ => SampleValue::new("'dust-id' as dynamic", Some("dust-id")),
    })
}

/// Renders the expected body assertion expression for a generated sample.
pub(super) fn sample_body_assertion(ty: &TypeIr, sample: &SampleValue) -> String {
    if uses_direct_body_value(ty) {
        sample.assertion_expression.clone()
    } else {
        format!("{}.toJson()", sample.expression)
    }
}
