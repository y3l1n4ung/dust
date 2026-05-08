# HttpClient Specification: Runtime & Error Handling

This document defines the recommended application-level architecture for using the generated clients.

## 1. Dio Configuration Factory

Production apps should centralize Dio initialization to ensure consistent behavior across all API clients.

```dart
Dio createProductionDio({
  required String baseUrl,
  List<Interceptor> additionalInterceptors = const [],
}) {
  final dio = Dio(
    BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: const Duration(seconds: 15),
      receiveTimeout: const Duration(seconds: 15),
    ),
  );

  dio.interceptors.addAll([
    AuthInterceptor(),
    LoggingInterceptor(),
    ...additionalInterceptors,
  ]);

  return dio;
}
```

## 2. Standard Interceptors

### A. AuthInterceptor
- **Responsibility**: Injecting Bearer tokens and handling 401 refresh loops.
- **Behavior**: Must use a separate "refresh" Dio instance to avoid infinite 401 loops during token renewal.

### B. LoggingInterceptor
- **Responsibility**: Pretty-printing requests/responses for developers.
- **Production Safety**: Must redact `Authorization` headers and sensitive body fields (e.g., `password`).

## 3. Domain Error Mapping

Generated code throws `DioException`. Applications should map these to a domain-specific `ApiException` hierarchy.

### Hierarchy
- `ApiException`: Base sealed class.
- `NetworkException`: No connectivity / Timeout.
- `ServerException`: 5xx errors.
- `UnauthorizedException`: 401 errors.
- `ValidationException`: 422 errors with a map of field-specific errors.

## 4. The `safeApiCall` Wrapper

To avoid `try-catch` blocks in every UI component, use a centralized wrapper.

```dart
Future<T> safeApiCall<T>(Future<T> Function() call) async {
  try {
    return await call();
  } on DioException catch (e) {
    throw _mapToDomainError(e);
  }
}
```
