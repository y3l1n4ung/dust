use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::helpers::SHARED_HELPERS;
use crate::http_client_plugin_tests::support::{
    config, future_of, http_client_class, library_for, library_for_with_imports, method,
};

/// Byte and text stream methods returned without rewrapping.
#[path = "streams/direct.rs"]
mod direct;

#[test]
fn emits_stream_fetches_for_response_body_payloads() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamPosts",
            future_of(TypeIr::named("ResponseBody")),
            vec![config("GET", Some("('/posts/raw')"))],
            Vec::new(),
        )],
    ));

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.shared_helpers.join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<ResponseBody> streamPosts() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<ResponseBody>(
      _setStreamType<ResponseBody>(
        _options
            .compose(
              _dio.options,
              '/posts/raw',
              queryParameters: _queryParameters,
              data: _data,
              cancelToken: null,
              onSendProgress: null,
              onReceiveProgress: null,
            )
            .copyWith(
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl,
              ),
            ),
      ),
    );
    return _result.data!;
  }
}
"#
    );
    assert_eq!(helpers, SHARED_HELPERS);
}

#[test]
fn returns_raw_stream_response_without_rewrapping() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "streamEnvelope",
            future_of(TypeIr::generic(
                "Response",
                vec![TypeIr::named("ResponseBody")],
            )),
            vec![config("GET", Some("('/posts/raw-response')"))],
            Vec::new(),
        )],
    ));

    let emitted = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution")
        .support_types
        .join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<Response<ResponseBody>> streamEnvelope() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    final _result = await _dio.fetch<ResponseBody>(
      _setStreamType<ResponseBody>(
        _options
            .compose(
              _dio.options,
              '/posts/raw-response',
              queryParameters: _queryParameters,
              data: _data,
              cancelToken: null,
              onSendProgress: null,
              onReceiveProgress: null,
            )
            .copyWith(
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl,
              ),
            ),
      ),
    );
    return _result;
  }
}
"#
    );
}
