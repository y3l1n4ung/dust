use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::model::{EndpointParam, ReturnSpec};
use crate::plugin::util::{is_response_body_type, is_string_keyed_map};

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

pub(super) fn sample_value(ty: &TypeIr) -> Option<SampleValue> {
    match ty {
        TypeIr::Builtin { kind, .. } => Some(match kind {
            BuiltinType::String => SampleValue::new("'dust-id'", Some("dust-id")),
            BuiltinType::Int => SampleValue::new("42", Some("42")),
            BuiltinType::Bool => SampleValue::new("true", Some("true")),
            BuiltinType::Double => SampleValue::new("3.14", Some("3.14")),
            BuiltinType::Num => SampleValue::new("7", Some("7")),
            BuiltinType::Object => {
                SampleValue::new("const <String, dynamic>{'value': 'dust'}", None)
            }
        }),
        TypeIr::Dynamic => Some(SampleValue::new(
            "const <String, dynamic>{'value': 'dust'}",
            None,
        )),
        TypeIr::Named { name, .. } if name.as_ref() == "String" => {
            Some(SampleValue::new("'dust-id'", Some("dust-id")))
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" && args.len() == 1 => {
            Some(SampleValue::new("const <dynamic>['dust']", None))
        }
        TypeIr::Named { args, .. } if is_string_keyed_map(ty) => map_sample_value(&args[1]),
        TypeIr::Named { nullable: true, .. } => Some(SampleValue::new("null", Some("null"))),
        _ => None,
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

fn map_sample_value(value_ty: &TypeIr) -> Option<SampleValue> {
    match value_ty {
        TypeIr::Dynamic | TypeIr::Unknown => Some(SampleValue::new(
            "const <String, dynamic>{'value': 'dust'}",
            None,
        )),
        TypeIr::Builtin { kind, .. } => Some(match kind {
            BuiltinType::String => {
                SampleValue::new("const <String, String>{'value': 'dust'}", None)
            }
            BuiltinType::Int => SampleValue::new("const <String, int>{'value': 42}", None),
            BuiltinType::Bool => SampleValue::new("const <String, bool>{'value': true}", None),
            BuiltinType::Double => SampleValue::new("const <String, double>{'value': 3.14}", None),
            BuiltinType::Num => SampleValue::new("const <String, num>{'value': 3.14}", None),
            BuiltinType::Object => {
                SampleValue::new("const <String, Object>{'value': 'dust'}", None)
            }
        }),
        TypeIr::Named { name, .. } if name.as_ref() == "String" => Some(SampleValue::new(
            "const <String, String>{'value': 'dust'}",
            None,
        )),
        TypeIr::Named { nullable: true, .. } => Some(SampleValue::new(
            "const <String, dynamic>{'value': null}",
            None,
        )),
        _ => Some(SampleValue::new(
            "const <String, dynamic>{'value': 'dust'}",
            None,
        )),
    }
}
