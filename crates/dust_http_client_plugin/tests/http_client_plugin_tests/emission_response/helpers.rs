pub(crate) const SHARED_HELPERS: &str = r#"RequestOptions _setStreamType<T>(RequestOptions requestOptions) {
  if (T != dynamic &&
      requestOptions.responseType != ResponseType.bytes &&
      requestOptions.responseType != ResponseType.stream) {
    if (T == ResponseBody) {
      requestOptions.responseType = ResponseType.stream;
    } else if (T == String) {
      requestOptions.responseType = ResponseType.plain;
    } else {
      requestOptions.responseType = ResponseType.json;
    }
  }
  return requestOptions;
}

String _combineBaseUrls(String dioBaseUrl, String? baseUrl) {
  if (baseUrl == null || baseUrl.trim().isEmpty) {
    return dioBaseUrl;
  }
  final url = Uri.parse(baseUrl);
  if (url.isAbsolute) {
    return url.toString();
  }
  return Uri.parse(dioBaseUrl).resolveUri(url).toString();
}

Response<T> _buildResponse<T>(Response<dynamic> response, T data) {
  return Response<T>(
    data: data,
    headers: response.headers,
    isRedirect: response.isRedirect,
    redirects: response.redirects,
    requestOptions: response.requestOptions,
    statusCode: response.statusCode,
    statusMessage: response.statusMessage,
    extra: response.extra,
  );
}"#;
