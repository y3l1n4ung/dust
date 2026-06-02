use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use crate::http_client_plugin_tests::support::{
    config, future_of, http_client_class, library_for, method,
};

#[test]
fn omits_result_binding_for_void_methods() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "deleteUser",
            future_of(TypeIr::named("void")),
            vec![config("DELETE", Some("('/users/me')"))],
            Vec::new(),
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
  Future<void> deleteUser() async {
    final _queryParameters = <String, dynamic>{};
    final _headers = <String, dynamic>{};
    final _extra = <String, dynamic>{};
    final Object? _data = null;
    final _options = Options(
      method: 'DELETE',
      headers: _headers,
      extra: _extra,
      contentType: null,
    );
    await _dio.fetch<void>(
      _setStreamType<void>(
        _options
            .compose(
              _dio.options,
              '/users/me',
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
