use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::emit::types::uses_direct_body_value;
use crate::plugin::model::{EndpointParam, ReturnSpec};
use crate::plugin::util::is_response_body_type;

pub(super) struct SampleValue {
    pub(super) expression: String,
    pub(super) assertion_expression: String,
    pub(super) path_value: Option<String>,
}

impl SampleValue {
    pub(super) fn new(expression: &str, path_value: Option<&str>) -> Self {
        Self {
            expression: expression.to_owned(),
            assertion_expression: expression.to_owned(),
            path_value: path_value.map(str::to_owned),
        }
    }
}

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
        TypeIr::Named { name, .. } if name.as_ref() == "void" => "null",
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" && args.len() == 1 => {
            "const <dynamic>[]"
        }
        TypeIr::Named { .. } if is_response_body_type(&return_spec.ty) => {
            "ResponseBody.fromString('{}', 200)"
        }
        TypeIr::Named { .. } => "const <String, dynamic>{}",
        TypeIr::Function { .. } | TypeIr::Record { .. } => "null",
    }
}

pub(super) fn fallback_sample(binding: &EndpointParam<'_>) -> Option<SampleValue> {
    Some(match binding {
        EndpointParam::Body { .. } => return None,
        EndpointParam::Queries { .. }
        | EndpointParam::HeaderMap { .. }
        | EndpointParam::Part { .. } => {
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

pub(super) fn sample_body_assertion(ty: &TypeIr, sample: &SampleValue) -> String {
    if uses_direct_body_value(ty) {
        sample.assertion_expression.clone()
    } else {
        format!("{}.toJson()", sample.expression)
    }
}
