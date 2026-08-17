import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// Builds a request, with router path parameters supplied directly so
/// extractors can be tested without mounting a router.
Request request(
  String method,
  String path, {
  Map<String, String> headers = const {},
  Map<String, String> pathParameters = const {},
  Map<String, Object> context = const {},
  Object? body,
}) {
  return Request(
    method,
    Uri.parse('http://localhost$path'),
    headers: headers,
    body: body,
    context: {
      if (pathParameters.isNotEmpty) pathParametersKey: pathParameters,
      ...context,
    },
  );
}

/// A JSON request with the right `content-type`.
Request jsonRequest(String method, String path, String body) {
  return request(
    method,
    path,
    headers: const {'content-type': 'application/json'},
    body: body,
  );
}

/// Unwraps an `Ok`, failing the test on `Err`.
T expectOk<T, E>(Result<T, E> result) {
  return switch (result) {
    Ok(:final value) => value,
    Err(:final error) => fail('expected Ok, got Err($error)'),
  };
}

/// Unwraps an `Err`, failing the test on `Ok`.
E expectErr<T, E>(Result<T, E> result) {
  return switch (result) {
    Ok(:final value) => fail('expected Err, got Ok($value)'),
    Err(:final error) => error,
  };
}

/// Asserts a rejection's status code and returns it.
Rejection expectStatus<T>(Result<T, Rejection> result, int status) {
  final rejection = expectErr(result);
  expect(rejection.status, status, reason: rejection.message);
  return rejection;
}
