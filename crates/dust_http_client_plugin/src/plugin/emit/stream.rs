use crate::plugin::model::{EndpointSpec, ReturnMode};

pub(crate) fn render_stream_yield(endpoint: &EndpointSpec<'_>) -> String {
    match endpoint.return_spec.mode {
        ReturnMode::ByteStream => {
            "    final _body = _result.data;\n    if (_body == null) return;\n    yield* _body.stream;\n"
                .to_owned()
        }
        ReturnMode::TextStream => {
            "    final _body = _result.data;\n    if (_body == null) return;\n    yield* utf8.decoder.bind(_body.stream);\n"
                .to_owned()
        }
        ReturnMode::Unary => String::new(),
    }
}
