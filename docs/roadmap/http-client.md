# HttpClient Plan

## Goal

Generate Retrofit-style, Dio-backed HTTP clients from annotated Dart
interfaces. Dust should generate type-safe request construction, response
decoding, and analyzer-clean client implementations without `build_runner`.

## Package Shape

- Dart annotation package: `dust_http_client_annotation`
- Rust plugin crate: `dust_http_client_plugin`
- Runtime dependency: generated clients use `package:dio/dio.dart` directly

No Dust transport runtime in v1. Dio already provides interceptors, timeout,
cancel tokens, progress callbacks, multipart, and request options.

## API Sketch

```dart
import 'package:dio/dio.dart';
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

part 'product_api.g.dart';

@HttpClient(
  baseUrl: 'https://api.example.com',
  target: DustHttpTarget.flutter,
  parseThread: DustParseThread.isolate,
)
abstract interface class ProductApi {
  factory ProductApi(Dio dio, {String? baseUrl}) = _$ProductApi;

  @GET('/products/{sku}')
  @HttpParse(thread: DustParseThread.main)
  Future<Product> getProduct(@Path('sku') String sku);

  @GET('/products')
  Future<List<Product>> listProducts({
    @Query('page') int page = 1,
    @Query('q') String? query,
    @Header('x-request-id') String? requestId,
    CancelToken? cancelToken,
  });

  @POST('/products')
  Future<Product> createProduct(@Body() ProductCreate input);

  @DELETE('/products/{sku}')
  Future<void> deleteProduct(@Path('sku') String sku);

  @GET('/products/{sku}')
  Future<Response<Product>> getProductRaw(@Path('sku') String sku);
}
```

## Annotation API

```dart
enum DustHttpTarget {
  dart,
  flutter,
}

enum DustParseThread {
  main,
  isolate,
}

final class HttpClient {
  final String baseUrl;
  final DustHttpTarget target;
  final DustParseThread parseThread;
  final Map<String, String> headers;

  const HttpClient({
    required this.baseUrl,
    this.target = DustHttpTarget.dart,
    this.parseThread = DustParseThread.main,
    this.headers = const {},
  });
}

final class HttpParse {
  final DustParseThread? thread;
  const HttpParse({this.thread});
}
```

Supported method annotations:

- `@GET`
- `@POST`
- `@PUT`
- `@PATCH`
- `@DELETE`
- `@HEAD`
- `@OPTIONS`

Supported parameter annotations:

- `@Path`
- `@Query`
- `@Queries`
- `@Header`
- `@Headers`
- `@Body`
- `@Part`
- `@Field`
- `@Extra`

Dio special parameters:

- `CancelToken`
- `Options`
- `ProgressCallback onSendProgress`
- `ProgressCallback onReceiveProgress`

## Serialization Strategy

Do not add a `DustJsonCodec` enum in v1. It has no useful effect when generated
code calls the same structural model API:

- `Model.fromJson(Map<String, dynamic>)`
- `model.toJson()`

This convention works with Dust serde, `json_serializable`, or hand-written
models. Future custom conversion should use converter annotations with real
effect, for example `@HttpConverter(...)`.

## Parse Thread Strategy

`parseThread` controls where response decoding runs:

- `DustParseThread.main`: decode directly in generated method.
- `DustParseThread.isolate` with `DustHttpTarget.flutter`: use Flutter
  `compute(...)`.
- `DustParseThread.isolate` with `DustHttpTarget.dart`: use `Isolate.run(...)`.

Method-level `@HttpParse` overrides the client default.

## Generated Example

```dart
final class _$ProductApi implements ProductApi {
  final Dio _dio;
  final String? _baseUrl;

  _$ProductApi(this._dio, {String? baseUrl}) : _baseUrl = baseUrl;

  @override
  Future<Product> getProduct(String sku) async {
    final response = await _dio.fetch<Map<String, dynamic>>(
      _setStreamType<Product>(
        Options(method: 'GET')
            .compose(
              _dio.options,
              '/products/${Uri.encodeComponent(sku)}',
            )
            .copyWith(
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl ?? 'https://api.example.com',
              ),
            ),
      ),
    );

    return Product.fromJson(response.data!);
  }

  @override
  Future<List<Product>> listProducts({
    int page = 1,
    String? query,
    String? requestId,
    CancelToken? cancelToken,
  }) async {
    final queryParameters = <String, dynamic>{
      'page': page,
      if (query != null) 'q': query,
    };
    final headers = <String, dynamic>{
      if (requestId != null) 'x-request-id': requestId,
    };

    final response = await _dio.fetch<List<dynamic>>(
      _setStreamType<List<Product>>(
        Options(method: 'GET', headers: headers)
            .compose(
              _dio.options,
              '/products',
              queryParameters: queryParameters,
              cancelToken: cancelToken,
            )
            .copyWith(
              baseUrl: _combineBaseUrls(
                _dio.options.baseUrl,
                _baseUrl ?? 'https://api.example.com',
              ),
            ),
      ),
    );

    return compute(_$ProductListFromJson, response.data!);
  }
}

List<Product> _$ProductListFromJson(List<dynamic> json) {
  return json
      .map((item) => Product.fromJson(item as Map<String, dynamic>))
      .toList();
}
```

## Return Types

V1 supports:

- `Future<T>`
- `Future<List<T>>`
- `Future<void>`
- `Future<Response<T>>`
- `Future<Response<List<T>>>`
- `Future<Response<void>>`
- primitive `T`: `String`, `int`, `double`, `bool`, `num`, `dynamic`

Later:

- `Stream<T>`
- websocket
- generated retry policy annotations
- endpoint-specific converter registry

## Validation

Dust must emit diagnostics when:

- `@HttpClient` target is not an abstract interface class.
- Factory constructor is missing or points to the wrong generated class.
- A method has no HTTP method annotation.
- A method has more than one HTTP method annotation.
- A path placeholder has no matching `@Path`.
- A `@Path` parameter has no matching path placeholder.
- More than one `@Body` exists.
- `GET`, `HEAD`, or `DELETE` uses `@Body`.
- Return type is not `Future<T>` or `Future<Response<T>>`.
- A request body model has no visible `toJson()`.
- A response model has no visible `fromJson(...)`.
- Duplicate query, header, path, part, field, or extra keys exist.
- `DustParseThread.isolate` is used for unsupported response shapes.

## Tests

- Rust parser and IR tests for all annotations.
- Rust validation tests for every diagnostic above.
- Golden tests for generated GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS.
- Dart analyzer tests for typed model, list model, primitive, void, and
  `Response<T>` returns.
- Runtime tests with Dio fake adapter.
- Flutter-only test for `compute(...)` generated parsing.
- Pure Dart test for `Isolate.run(...)` generated parsing.

## Done

- Users can define a Dio-backed API interface and get generated implementation.
- Generated code is analyzer-clean and close to Retrofit style.
- Main-thread and isolate parsing are documented and tested.
- No unnecessary codec enum exists in v1.
- CI blocks generator and generated-code regressions.
