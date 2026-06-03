# dust_http_client_annotation

Annotations for Dust HTTP client generation.

## Features

- `@HttpClient()` for client-level configuration
- HTTP method annotations such as `@GET()` and `@POST()`
- request binding annotations such as `@Path()`, `@Query()`, `@Body()`, and
  `@Header()`
- content-type markers such as `@FormUrlEncoded()` and `@MultiPart()`
- `@GenerateTest()` support for generated request-mapping tests

## Getting started

Install Dust by following the root README, then add this package and `dio` to
your Dart project:

```yaml
dependencies:
  dio: ^5.9.2
  dust_http_client_annotation: ^0.1.0
```

## Usage

```dart
import 'package:dust_dart/http.dart';

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
`test/generated/api/todo_api_test.dart` when `@GenerateTest()` is present.

## Additional information

- Install Dust from the repository guide:
  [README.md](https://github.com/y3l1n4ung/dust/blob/main/README.md)
- Full HttpClient usage guide:
  [docs/usage/http.md](https://github.com/y3l1n4ung/dust/blob/main/docs/usage/http.md)
- Runnable example:
  [examples/product_showcase](https://github.com/y3l1n4ung/dust/tree/main/examples/product_showcase)
