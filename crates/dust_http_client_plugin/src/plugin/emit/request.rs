use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::emit::path::{PathSegment, path_segments};
use crate::plugin::emit::types::render_body_value;
use crate::plugin::model::{EndpointParam, EndpointSpec, RequestMode};
use crate::plugin::util::escape_single_quoted;

/// Template context for standard request data generation.
#[derive(Serialize)]
struct StandardDataContext {
    /// Dart expression used as the request body.
    value: String,
}

/// Template context for form-url-encoded and multipart request bodies.
#[derive(Serialize)]
struct KeyedDataContext {
    /// Dart expression prefix before the generated map.
    prefix: &'static str,
    /// Rendered keyed body fields.
    fields: String,
    /// Dart expression suffix after the generated map.
    suffix: &'static str,
}

/// Template context for one keyed body entry.
#[derive(Serialize)]
struct KeyedFieldContext<'a> {
    /// Optional Dart null guard prefix for nullable parameters.
    nullable_prefix: String,
    /// Escaped body field key.
    key: String,
    /// Dart parameter name whose value is sent.
    name: &'a str,
}

/// Renders the Dio `data:` expression for an endpoint request.
pub(super) fn render_request_data(endpoint: &EndpointSpec<'_>) -> String {
    match endpoint.request_mode {
        RequestMode::Standard => endpoint
            .params
            .iter()
            .find_map(|param| match param {
                EndpointParam::Body { param } => Some(param),
                _ => None,
            })
            .map(|param| render_standard_data(render_body_value(param)))
            .unwrap_or_else(|| render_standard_data("null".to_owned())),
        RequestMode::FormUrlEncoded => render_keyed_body(endpoint, false),
        RequestMode::MultiPart => render_keyed_body(endpoint, true),
    }
}

/// Renders a standard Dio request data assignment.
fn render_standard_data(value: String) -> String {
    render_template(
        "request_data_standard",
        include_str!("templates/request_data_standard.jinja"),
        StandardDataContext { value },
    )
}

/// Renders the generated Dart expression for an endpoint path.
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

/// Renders form-url-encoded or multipart keyed request data.
fn render_keyed_body(endpoint: &EndpointSpec<'_>, multipart: bool) -> String {
    let fields = endpoint
        .params
        .iter()
        .filter_map(|param| match param {
            EndpointParam::Field { param, key } if !multipart => {
                Some(render_keyed_field(&param.name, key, param.ty.is_nullable()))
            }
            EndpointParam::Part { param, key } if multipart => {
                Some(render_keyed_field(&param.name, key, param.ty.is_nullable()))
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let fields = if fields.is_empty() {
        fields
    } else {
        format!("{fields}\n")
    };
    render_template(
        "request_data_keyed",
        include_str!("templates/request_data_keyed.jinja"),
        KeyedDataContext {
            prefix: if multipart { "FormData.fromMap(" } else { "" },
            fields,
            suffix: if multipart { ")" } else { "" },
        },
    )
}

/// Renders a single keyed body field with an optional null guard.
fn render_keyed_field(name: &str, key: &str, nullable: bool) -> String {
    render_template(
        "request_keyed_field",
        include_str!("templates/request_keyed_field.jinja"),
        KeyedFieldContext {
            nullable_prefix: if nullable {
                format!("if ({name} != null) ")
            } else {
                String::new()
            },
            key: escape_single_quoted(key),
            name,
        },
    )
}
