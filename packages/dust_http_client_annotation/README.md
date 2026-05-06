# dust_http_client_annotation

Annotation package for Dust HTTP client generation. It defines the public API
surface used by `dust_http_client_plugin` to generate Dio-backed clients and
optional request-mapping tests.

## What it covers

- `@HttpClient()` class configuration
- HTTP verb annotations such as `@GET()` and `@POST()`
- request binding annotations such as `@Path()`, `@Query()`, `@Body()`, and
  `@Header()`
- content-type markers such as `@FormUrlEncoded()` and `@MultiPart()`
- `@GenerateTest()` support for generated `.test.g.dart` request-mapping tests

## Usage

```dart
import 'package:dio/dio.dart';
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

part 'todo_api.g.dart';

@HttpClient(
  baseUrl: 'https://api.todos.com',
  parseThread: DustParseThread.isolate,
  headers: {'x-api-version': '2026-05'},
)
@GenerateTest()
abstract interface class TodoApi {
  factory TodoApi(Dio dio, {String? baseUrl}) = _$TodoApi;

  @GET('/todos/{id}')
  Future<Todo> get(@Path() String id);

  @POST('/todos')
  Future<Todo> create(@Body() TodoCreate request);
}
```

Dust generates `todo_api.g.dart` for the concrete Dio client and
`todo_api.test.g.dart` when `@GenerateTest()` is present.

## Full Usage Guide

See the root usage docs for the full HttpClient guide:

- [../../docs/usage/http.md](../../docs/usage/http.md)

That guide covers:
- model generation with derive and serde
- typed and map-based `@Body()` usage
- request binding, shared headers, and raw `Response<T>` returns
- the real online `jsonplaceholder` demo
- generated request-mapping tests
