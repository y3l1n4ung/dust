# HTTP Client Generation

Dust generates Dio-backed API clients from abstract interfaces. It automates request mapping, header injection, and JSON parsing based on annotations.

---

## Installation

Add the required packages to your `pubspec.yaml`:

```yaml
dependencies:
  dio: ^5.0.0
  dust_dart: ^0.1.0
```

---

## Basic Example

Define an `abstract interface class` and annotate it with `@HttpClient`.

```dart
import 'package:dust_dart/http.dart';

part 'api_client.g.dart';

@HttpClient(baseUrl: 'https://api.example.com')
abstract interface class ApiClient {
  factory ApiClient(Dio dio, {String? baseUrl}) = _$ApiClient;

  @GET('/users/{id}')
  Future<User> getUser(@Path() String id);

  @POST('/users')
  Future<User> createUser(@Body() User user);
}
```

> [!IMPORTANT]
> **Requirements for Generation:**
> 1. You **must** include the `part 'filename.g.dart';` directive.
> 2. You **must** provide a redirecting factory constructor: `factory ClassName(Dio dio, {String? baseUrl}) = _$ClassName;`.

---

## Configuration Reference

### `@HttpClient` (Class Level)

| Property | Type | Description |
| :--- | :--- | :--- |
| `baseUrl` | `String` | The base URL for all methods in this client. |
| `target` | `HttpTarget` | `dart` (default) or `flutter`. Use `flutter` to enable Flutter-specific optimizations. |
| `parseThread` | `HttpParseThread` | `main` (default) or `isolate`. Use `isolate` to offload JSON decoding to a background thread. |
| `headers` | `Map<String, String>` | Static headers applied to every request from this client. |
| `generateTest` | `bool` | Generates a companion request-mapping test file when set to `true`. |

### Method Annotations

| Annotation | Description |
| :--- | :--- |
| `@GET(path)`, `@POST(path)`, etc. | Defines the HTTP verb and relative path. |
| `@Headers({...})` | Sets static headers for a specific method. |
| `@FormUrlEncoded()` | Sets the content-type to `application/x-www-form-urlencoded`. |
| `@MultiPart()` | Sets the content-type to `multipart/form-data`. |
| `@HttpParse(thread: ...)` | Overrides the class-level `parseThread` strategy for this specific method. |

### Parameter Annotations

| Annotation | Description |
| :--- | :--- |
| `@Path([name])` | Maps a parameter to a `{name}` in the path. If name is omitted, uses the parameter name. |
| `@Query(key)` | **Required.** Maps a parameter to a URL query key. |
| `@Queries()` | Maps a `Map<String, dynamic>` parameter to multiple query keys. |
| `@Body()` | Encodes the parameter as the request body (auto-serialized to JSON). |
| `@Header(key)` | **Required.** Maps a parameter to a specific HTTP header. |
| `@HeaderMap()` | Maps a `Map<String, String>` parameter to multiple request headers. |
| `@Field(name)` | **Required.** Maps a parameter to a form field (requires `@FormUrlEncoded`). |
| `@Part(name)` | **Required.** Maps a parameter to a multipart part (requires `@MultiPart`). |
| `@Extra(key)` | **Required.** Maps a parameter to Dio's `RequestOptions.extra` map. |

> [!NOTE]
> For annotations marked **Required**, you must provide a string literal for the key/name. Only `@Path()` allows omitting the argument to fallback to the parameter name.

> [!IMPORTANT]
> Request bodies are supported only on `POST`, `PUT`, `PATCH`, and `DELETE`. Dust rejects `@Body()`, `@FormUrlEncoded()` fields, and `@MultiPart()` parts on `GET`, `HEAD`, and `OPTIONS`.

---

## Performance: Offloading JSON Parsing

For large JSON payloads, you can offload the decoding process to a background isolate.

```dart
@HttpClient(parseThread: HttpParseThread.isolate)
abstract interface class BigDataApi { ... }
```

> [!IMPORTANT]
> Dart-targeted clients use `Isolate.run`, so add `import 'dart:isolate';` when isolate parsing is enabled. Flutter-targeted clients use Flutter's `compute` helper, so add `import 'package:flutter/foundation.dart' show compute;` when Flutter targeting and isolate parsing are enabled.

> [!TIP]
> Use `HttpParse` to enable isolates only for specific heavy endpoints while keeping lightweight calls on the main thread. This provides granular control over resource usage.

---

## Multipart Requests

To upload files or mixed data, use `@MultiPart` and `@Part`.

```dart
@POST('/upload')
@MultiPart()
Future<void> uploadFile(@Part('file') MultipartFile file, @Part('id') String id);
```

> [!WARNING]
> When using `@MultiPart`, ensure all non-file parameters are also annotated with `@Part`. Standard `@Query` or `@Body` annotations may not behave as expected within a multipart request depending on the server implementation.

---

## Generated Testing

When `generateTest` is enabled on your client, Dust generates a test suite that verifies request mapping logic without requiring a real server.

```dart
@HttpClient(baseUrl: '...', generateTest: true)
abstract interface class ApiClient { ... }
```

Dust creates `test/generated/api/api_client_test.dart` containing assertions for path segments, query parameters, headers, and body serialization.

---

## Generation Output

Dust generates a concrete implementation class (`_$ClassName`) that utilizes Dio.

```dart
// api_client.g.dart (Simplified)
class _$ApiClient implements ApiClient {
  _$ApiClient(this._dio, {this.baseUrl});

  final Dio _dio;
  String? baseUrl;

  @override
  Future<User> getUser(String id) async {
    final _result = await _dio.fetch<Map<String, dynamic>>(
      _setStreamType<User>(Options(method: 'GET'))
        .compose(baseUrl: baseUrl, path: '/users/$id'),
    );
    return User.fromJson(_result.data!);
  }
}
```

---

## Migration Guide

**Coming from `retrofit`?**

| Feature | `retrofit` | Dust |
| :--- | :--- | :--- |
| Main Annotation | `@RestApi()` | `@HttpClient()` |
| Path Param | `@Path("id")` | `@Path()` (optional arg) |
| Query Param | `@Query("q")` | `@Query("q")` (required arg) |
| Multithreading | Optional | Built-in via `HttpParseThread.isolate` |
| Build Tool | `build_runner` | **Standalone Binary** |

> [!TIP]
> If you are migrating a large codebase, use `dust build` regularly to catch mismatched path parameters or unsupported return types early via the built-in diagnostic engine.
