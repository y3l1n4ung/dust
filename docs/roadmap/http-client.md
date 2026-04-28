# HttpClient Plan

## Goal

Generate type-safe HTTP clients from annotated Dart interfaces. The feature name
is `HttpClient`, not generic `Client`, to keep intent clear.

## Package Shape

- Dart annotation package: `http_client_annotation`
- Rust plugin crate: `dust_plugin_http_client`
- Optional runtime package: `dust_http`

## API Sketch

```dart
@HttpClient(baseUrl: 'https://api.example.com')
abstract interface class ProductApi {
  @GET('/products/{sku}')
  Future<Product> getProduct(@Path('sku') String sku);

  @POST('/products')
  Future<Product> createProduct(@Body() ProductCreate input);

  @GET('/products')
  Future<List<Product>> listProducts({
    @Query('page') int page = 1,
    @Header('x-request-id') String? requestId,
  });
}
```

Generated output:

```dart
final api = _$ProductApiHttpClient(transport: transport);
final product = await api.getProduct('sku-1');
```

## Generator Work

- Parse method annotations and parameter annotations.
- Lower endpoint paths, path params, query params, headers, and body into IR.
- Validate every `{path}` segment has exactly one `@Path`.
- Use existing serde generation for request and response models.
- Generate injectable transport abstraction for tests.
- Generate timeout, retry, and interceptor hooks without forcing a heavy runtime.

## Tests

- Rust validation tests for missing path params and duplicate annotations.
- Golden tests for GET, POST, PUT, PATCH, DELETE.
- Dart tests using fake transport.
- Analyzer tests for typed responses and nullable params.

## Done

- Users can generate a REST API client with typed models and no manual JSON glue.
- Generated client is testable without network.
- Failure cases produce clear diagnostics.
