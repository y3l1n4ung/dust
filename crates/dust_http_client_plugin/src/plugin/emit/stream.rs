use dust_dart_emit::render_template;

use crate::plugin::model::{EndpointSpec, ReturnMode};

/// Renders the body for generated stream endpoint forwarding.
pub(crate) fn render_stream_yield(endpoint: &EndpointSpec<'_>) -> String {
    match endpoint.return_spec.mode {
        ReturnMode::ByteStream => format!(
            "{}\n",
            render_template(
                "byte_stream_yield",
                include_str!("templates/byte_stream_yield.jinja"),
                (),
            )
        ),
        ReturnMode::TextStream => format!(
            "{}\n",
            render_template(
                "text_stream_yield",
                include_str!("templates/text_stream_yield.jinja"),
                (),
            )
        ),
        ReturnMode::Unary => String::new(),
    }
}
