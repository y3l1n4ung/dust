use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::support::{
    config, future_of, http_client_class, library_for, method, required_named_param,
};

#[test]
fn preserves_explicit_required_nullable_named_parameters() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "search",
            future_of(TypeIr::named("void")),
            vec![config("GET", Some("('/search')"))],
            vec![required_named_param(
                "traceId",
                TypeIr::string().nullable(),
                vec![config("Header", Some("('x-trace-id')"))],
            )],
        )],
    ));

    let emitted = plugin
        .emit(&library, &SymbolPlan::default())
        .support_types
        .join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<void> search({required String? traceId}) async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    if (traceId != null) _headers['x-trace-id'] = traceId;
    final Object? _data = null;
    final _options = Options(
      method: 'GET',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    await _dio.fetch<void>(
      _setStreamType<void>(
        _options
            .compose(
              _dio.options,
              '/search',
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
    return;
  }
}
"#
    );
}
