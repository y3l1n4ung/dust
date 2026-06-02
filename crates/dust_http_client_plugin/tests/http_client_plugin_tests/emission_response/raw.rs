use dust_http_client_plugin::register_plugin;
use dust_ir::TypeIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};

use super::helpers::SHARED_HELPERS;
use crate::http_client_plugin_tests::support::{
    config, future_of, http_client_class, library_for, method,
};

#[test]
fn emits_raw_response_wrappers_for_prefixed_response_types() {
    let plugin = register_plugin();
    let library = library_for(http_client_class(
        vec![config("HttpClient", Some("()"))],
        vec![method(
            "fetchRaw",
            future_of(TypeIr::generic("dio.Response", vec![TypeIr::named("User")])),
            vec![config("GET", Some("('/users/raw')"))],
            Vec::new(),
        )],
    ));

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let emitted = contribution.support_types.join("\n");
    let helpers = contribution.shared_helpers.join("\n");

    assert_eq!(
        emitted,
        r#"final class _$Api implements Api {
  _$Api(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  @override
  Future<dio.Response<User>> fetchRaw() async {
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
    final _result = await _dio.fetch<Map<String, dynamic>>(
      _setStreamType<User>(
        _options
            .compose(
              _dio.options,
              '/users/raw',
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
    final _value = User.fromJson(_result.data as Map<String, dynamic>);
    return _buildResponse<User>(_result, _value);
  }
}
"#
    );
    assert_eq!(helpers, SHARED_HELPERS);
}
