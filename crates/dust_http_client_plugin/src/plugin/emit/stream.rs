use crate::plugin::model::{EndpointSpec, ReturnMode};

pub(crate) fn render_stream_yield(endpoint: &EndpointSpec<'_>) -> String {
    match endpoint.return_spec.mode {
        ReturnMode::ByteStream => r#"    final _body = _result.data;
    if (_body == null) return;
    yield* _body.stream;
"#
        .to_owned(),
        ReturnMode::TextStream => r#"    final _body = _result.data;
    if (_body == null) return;
    yield* utf8.decoder.bind(_body.stream);
"#
        .to_owned(),
        ReturnMode::Unary => String::new(),
    }
}
