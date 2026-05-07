# HttpClient Guide

Use `dust_http_client_annotation` when you want Dust to generate a Dio-backed
client from an abstract interface.

## Install

Add the required packages to `pubspec.yaml`:

```yaml
dependencies:
  dio: ^5.9.2
  dust_http_client_annotation: ^0.1.0
```

Then fetch packages:

```bash
dart pub get
```

## Minimal Client Shape

```dart
import 'package:dio/dio.dart' hide Headers;
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

part 'todo_api.g.dart';

@HttpClient(baseUrl: 'https://api.example.com')
abstract interface class TodoApi {
  factory TodoApi(Dio dio, {String? baseUrl}) = _$TodoApi;

  @GET('/todos/{id}')
  Future<Todo> fetch(@Path() String id);
}
```

Run generation:

```bash
dust build
```

Dust generates:

- `todo_api.g.dart`
- `test/generated/api/todo_api_test.dart` when `@GenerateTest()` is present

## Real Online Example

Reference file:

- [examples/product_showcase/lib/api/json_placeholder_api.dart](../../examples/product_showcase/lib/api/json_placeholder_api.dart)

`JsonPlaceholderApi` is the real fake-online showcase against
`https://jsonplaceholder.typicode.com`.

It demonstrates:

- typed `GET` list responses
- raw `Response<T>` reads
- typed `@Body()` model payloads
- raw `Map<String, dynamic>` patch payloads
- raw `Response<Map<String, dynamic>>` delete responses

Current flow:

- `GET /posts`
- `GET /posts/{id}`
- `GET /comments`
- `POST /posts` with `RemotePostDraft`
- `PUT /posts/{id}` with `RemotePost`
- `PATCH /posts/{id}` with `Map<String, dynamic>`
- `DELETE /posts/{id}`

Run the live demo from the showcase package:

```bash
dart run bin/json_placeholder_demo.dart
```

Optional live smoke test:

```bash
DUST_RUN_ONLINE_HTTP_TESTS=1 dart test test/json_placeholder_api_test.dart
```

## Full Annotation Surface

Reference file:

- [examples/product_showcase/lib/api/todo_api.dart](../../examples/product_showcase/lib/api/todo_api.dart)

`TodoApi` covers the richer annotation matrix:

- `@HttpClient(...)`
- `@GenerateTest()`
- `@Headers(...)`
- `@GET`, `@POST`, `@PUT`, `@PATCH`, `@DELETE`
- `@Path()`
- `@Query()`
- `@Queries()`
- `@Header()`
- `@HeaderMap()`
- `@Body()`
- `@Field()`
- `@Extra()`
- `@FormUrlEncoded()`
- `DustParseThread.isolate`
- Dio passthrough params such as `CancelToken`

## Supported Return Shapes

Today the generator supports these return shapes:

- `Future<T>`
- `Future<List<T>>`
- `Future<void>`
- `Future<Map<String, dynamic>>`
- `Future<Response<T>>`
- `Future<Response<List<T>>>`
- `Future<Response<void>>`
- `Future<Response<Map<String, dynamic>>>`

The plugin validates this contract directly. Methods outside these async return
shapes are rejected during `dust build`.

## Generated Request-Mapping Tests

When `@GenerateTest()` is present, Dust emits a generated Dart test under
`test/generated/..._test.dart` with request-mapping assertions.

Reference files:

- [examples/product_showcase/test/generated/api/json_placeholder_api_test.dart](../../examples/product_showcase/test/generated/api/json_placeholder_api_test.dart)
- [examples/product_showcase/test/generated/api/todo_api_test.dart](../../examples/product_showcase/test/generated/api/todo_api_test.dart)

## Output Config

You can change the primary generated suffix in `dust.yaml`:

```yaml
outputs:
  primary_suffix: .g.dart
```

If you change the suffix, the source file `part` directive must match it.

Current limitation:

- primitive and map-like request bodies can get generated fixtures
- arbitrary model-body endpoints are still skipped in generated request-mapping tests
- runtime tests in `product_showcase` cover those typed-body endpoints directly

## See Also

- [Package README](../../packages/dust_http_client_annotation/README.md)
- [Usage overview](./README.md)
