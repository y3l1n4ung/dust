# HTTP Clients

Dust generates typed Dio clients from annotated Dart interfaces.

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Dart runtime package:

```bash
dart pub add dust_dart
```

`package:dust_dart/http.dart` exports the HTTP annotations, Serde API, and Dio
types used by generated clients.

## Quick Start

Define an `abstract interface class` with a redirecting factory:

```dart
import 'package:dust_dart/http.dart';

part 'user_api.g.dart';

@Derive([Serialize(), Deserialize()])
class User with _$User {
  const User({required this.id, required this.name});

  factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);

  final int id;
  final String name;
}

@HttpClient(baseUrl: 'https://api.example.com')
abstract interface class UserApi {
  factory UserApi(Dio dio, {String? baseUrl}) = _$UserApi;

  @GET('/users/{id}')
  Future<User> getUser(@Path() int id);

  @POST('/users')
  Future<User> createUser(@Body() User user);
}
```

Generate and use the client:

```bash
dust build
```

```dart
final api = UserApi(Dio());
final user = await api.getUser(42);
```

> [!IMPORTANT]
> The source needs the matching `part` directive. The client must be an
> `abstract interface class` with the factory shape shown above.

## Client Configuration

| Option | Behavior |
| :--- | :--- |
| `baseUrl` | Default URL for every endpoint. |
| `headers` | Static headers applied to every endpoint. |
| `target` | Selects `HttpTarget.dart` or `HttpTarget.flutter`. |
| `parseThread` | Selects main-isolate or background-isolate JSON decoding. |
| `generateTest` | Generates request-mapping tests under `test/generated`. |

The factory's `baseUrl` overrides the annotation value. An absolute value
replaces `Dio.options.baseUrl`; a relative value resolves against it. When
neither override nor annotation supplies a URL, the client uses
`Dio.options.baseUrl`.

## Endpoints

Every client method must have exactly one supported HTTP verb annotation:

```dart
@GET('/users')
@POST('/users')
@PUT('/users/{id}')
@PATCH('/users/{id}')
@DELETE('/users/{id}')
@HEAD('/users/{id}')
@OPTIONS('/users')
```

Method-level configuration:

| Annotation | Behavior |
| :--- | :--- |
| `@Headers({...})` | Adds static headers for one endpoint. |
| `@FormUrlEncoded()` | Builds an `application/x-www-form-urlencoded` body from `@Field` parameters. |
| `@MultiPart()` | Builds `FormData` from `@Part` parameters. |
| `@HttpParse(thread: ...)` | Overrides the client parse-thread setting for one endpoint. |

## Parameters

| Annotation | Request mapping |
| :--- | :--- |
| `@Path([name])` | Replaces a matching `{name}` path segment. The parameter name is used when omitted. |
| `@Query(name)` | Adds one query value. Nullable values are omitted when `null`. |
| `@Queries()` | Merges a `Map<String, ...>` into the query map. |
| `@Header(name)` | Adds one header. Nullable values are omitted when `null`. |
| `@HeaderMap()` | Merges a `Map<String, ...>` into the headers. |
| `@Body()` | Sends one value as the request body. |
| `@Field(name)` | Adds one form-url-encoded field. |
| `@Part(name)` | Adds one multipart field or file. |
| `@Extra(key)` | Adds one value to Dio's request `extra` map. |

`@Query`, `@Header`, `@Field`, `@Part`, and `@Extra` require an explicit string
key. Dust checks duplicate keys and verifies that every path placeholder has
one matching `@Path` parameter.

> [!TIP]
> Use `@Queries()` and `@HeaderMap()` for caller-controlled sets. Keep fixed API
> keys visible with individual `@Query` and `@Header` parameters.

Dio transport parameters need no annotation:

- `CancelToken`
- `Options`
- `ProgressCallback onSendProgress`
- `ProgressCallback onReceiveProgress`

## Request Bodies

Standard bodies use one `@Body()` parameter:

```dart
@POST('/users')
Future<User> createUser(@Body() User user);
```

Primitive values, `Map`, `List`, `Object`, and `dynamic` are sent directly.
Custom models must provide `toJson()`; deriving `Serialize()` satisfies that
requirement.

Form and multipart bodies use their matching parameter annotations:

```dart
@FormUrlEncoded()
@PATCH('/users/{id}')
Future<User> rename(
  @Path() int id,
  @Field('name') String name,
);

@MultiPart()
@POST('/uploads')
Future<void> upload(
  @Part('file') MultipartFile file,
  @Part('label') String? label,
);
```

> [!IMPORTANT]
> `GET`, `HEAD`, and `OPTIONS` cannot have standard, form, or multipart bodies.
> Form and multipart modes cannot be combined with each other or with
> `@Body()`.

## Response Types

Supported method shapes include:

- `Future<T>`
- `Future<Response<T>>`
- `Future<ResponseBody>`
- `Stream<List<int>>` for response bytes
- `Stream<String>` for decoded response text

`T` may be a primitive, `Map<String, ...>`, `List<T>`, `dynamic`, `void`, or a
custom model. Custom response models must provide
`factory Type.fromJson(Map<String, Object?> json)`; deriving `Deserialize()`
and adding the forwarding factory satisfies that requirement.

Dust checks known workspace request and response models during generation.
Types from external packages remain subject to Dart analysis.

## Headers and Request Values

Class headers are applied first, followed by method `@Headers`, then parameter
headers in signature order. Later values replace earlier values with the same
key.

Path values use `toString()` and `Uri.encodeComponent`. Query values are passed
to Dio so scalar and list encoding follows Dio's configuration. Header values
use `toString()`.

## Isolate Decoding

Use isolate decoding globally or for one endpoint:

```dart
@HttpClient(parseThread: HttpParseThread.isolate)
abstract interface class CatalogApi {
  factory CatalogApi(Dio dio, {String? baseUrl}) = _$CatalogApi;

  @GET('/catalog')
  Future<List<Product>> getCatalog();

  @HttpParse(thread: HttpParseThread.main)
  @GET('/featured')
  Future<Product> getFeatured();
}
```

Dart-targeted clients use `Isolate.run` and require:

```dart
import 'dart:isolate';
```

Flutter-targeted clients use `compute` and require:

```dart
import 'package:flutter/foundation.dart' show compute;
```

Set `target: HttpTarget.flutter` with the Flutter import. The method-level
annotation can also return decoding to `HttpParseThread.main` when the client
default is isolate decoding.

> [!NOTE]
> Isolate decoding applies to custom JSON models and lists of those models. It
> does not move primitive, map, raw-body, or stream handling to an isolate.

## Generated Request Tests

Set `generateTest: true` to generate request-mapping tests under
`test/generated`. These tests intercept Dio requests and verify the method,
path, query values, headers, extras, and body without calling a real server.

```dart
@HttpClient(
  baseUrl: 'https://api.example.com',
  generateTest: true,
)
abstract interface class UserApi {
  // ...
}
```

## Examples

- [Complete API contract](../../examples/product_showcase/lib/api/todo_api.dart)
- [Streaming and multipart API](../../examples/product_showcase/lib/api/json_placeholder_api.dart)
- [Generated request tests](../../examples/product_showcase/test/generated/api/todo_api_test.dart)
- [Runtime HTTP tests](../../examples/product_showcase/test/http_client_showcase_test.dart)
