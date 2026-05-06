use crate::plugin::emit::path::{PathSegment, path_segments};
use crate::plugin::emit::types::render_body_value;
use crate::plugin::model::{EndpointParam, EndpointSpec, RequestMode};
use crate::plugin::util::escape_single_quoted;

pub(super) fn render_request_data(endpoint: &EndpointSpec<'_>) -> String {
    match endpoint.request_mode {
        RequestMode::Standard => endpoint
            .params
            .iter()
            .find_map(|param| match param {
                EndpointParam::Body { param } => Some(param),
                _ => None,
            })
            .map(|param| format!("    final Object? _data = {};\n", render_body_value(param)))
            .unwrap_or_else(|| "    final Object? _data = null;\n".to_owned()),
        RequestMode::FormUrlEncoded => render_keyed_body(endpoint, false),
        RequestMode::MultiPart => render_keyed_body(endpoint, true),
    }
}

pub(super) fn render_path_expression(endpoint: &EndpointSpec<'_>) -> String {
    let pieces = path_segments(endpoint)
        .into_iter()
        .map(|segment| match segment {
            PathSegment::Literal(value) => format!("'{}'", escape_single_quoted(value)),
            PathSegment::Binding { param_name, .. } => {
                format!("Uri.encodeComponent({param_name}.toString())")
            }
        })
        .collect::<Vec<_>>();

    match pieces.len() {
        0 => "''".to_owned(),
        1 => pieces[0].clone(),
        _ => pieces.join(" + "),
    }
}

fn render_keyed_body(endpoint: &EndpointSpec<'_>, multipart: bool) -> String {
    let mut out = if multipart {
        String::from("    final _data = FormData.fromMap(<String, dynamic>{\n")
    } else {
        String::from("    final _data = <String, dynamic>{\n")
    };

    for param in &endpoint.params {
        match param {
            EndpointParam::Field { param, key } if !multipart => {
                push_keyed_field(&mut out, &param.name, key, param.ty.is_nullable());
            }
            EndpointParam::Part { param, key } if multipart => {
                push_keyed_field(&mut out, &param.name, key, param.ty.is_nullable());
            }
            _ => {}
        }
    }

    out.push_str(if multipart { "    });\n" } else { "    };\n" });
    out
}

fn push_keyed_field(out: &mut String, name: &str, key: &str, nullable: bool) {
    let key = escape_single_quoted(key);
    if nullable {
        out.push_str(&format!("      if ({name} != null) '{key}': {name},\n"));
    } else {
        out.push_str(&format!("      '{key}': {name},\n"));
    }
}
