# HttpClient Specification: Testing & Faking

This document defines the strategies for testing and faking API interactions in applications using Dust-generated clients.

## 1. Faking Network Responses (Manual)

To test UI or business logic without a real backend, we recommend using Dio's `HttpClientAdapter`.

### A. Using a Mock Adapter
Provide a mocked adapter to the `Dio` instance passed to the generated client.

```dart
test('should fetch todos from fake server', () async {
  final dio = Dio();
  final adapter = MockAdapter();
  dio.httpClientAdapter = adapter;

  final todoApi = TodoApi(dio);

  // Setup fake response
  adapter.onGet('/todos', (server) => server.reply(200, [
    {'id': 1, 'title': 'Test Todo', 'completed': false}
  ]));

  final todos = await todoApi.getTodos();
  expect(todos.first.title, 'Test Todo');
});
```

---

## 2. Auto-Generated Tests (`@GenerateTest`)

Dust can optionally generate a test suite for your API client to verify that the code generation correctly maps your parameters to HTTP requests.

### A. The Annotation
Apply `@GenerateTest()` to your API interface.

```dart
@HttpClient(baseUrl: 'https://api.example.com')
@GenerateTest()
abstract interface class UserApi { ... }
```

### B. Generated Output (`user_api_test.g.dart`)
Dust will emit a test file that uses a `MockAdapter` to verify:
1.  **Path Resolution**: Correct interpolation of `@Path` variables.
2.  **Query Params**: Verification that all `@Query` parameters are present in the final URI.
3.  **Header Injection**: Verification that `@Header` values reach the request.
4.  **Body Encoding**: Verification that `@Body` is correctly serialized to JSON/Form/Multipart.

**Example of a generated test case:**
```dart
test('UserApi.getUser(id) maps to GET /users/{id}', () async {
  final dio = Dio();
  final adapter = MockAdapter();
  dio.httpClientAdapter = adapter;
  final api = UserApi(dio);

  adapter.onGet('/users/123', (server) => server.reply(200, {}));

  await api.getUser('123'); // Should not throw
  
  // Internal verification that the path was hit exactly once
  adapter.verifyPathHit('/users/123', method: 'GET');
});
```

---

## 3. Testing Interceptors

Interceptors should be tested in isolation.

### A. AuthInterceptor Test
Verify that the `Authorization` header is correctly injected into `RequestOptions`.

---

## 4. Verification Requirements

- **Analyzer Verification**: Generated code must be run through the Dart analyzer.
- **Isolate Safety**: Tests must verify that requests marked for `DustParseThread.isolate` correctly offload decoding without blocking the test's main loop.
