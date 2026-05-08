# HttpClient Specification: Annotations

This document details all supported annotations for the HttpClient generator.

## 1. Class Annotations

Applied to an `abstract interface class`.

| Annotation | Property | Type | Description |
| :--- | :--- | :--- | :--- |
| `@HttpClient` | `baseUrl` | `String` | Default base URL for all methods. |
| | `parseThread` | `DustParseThread` | Global default for JSON decoding (Main or Isolate). |
| `@GenerateTest` | - | - | Instructs Dust to generate a test suite (`.test.g.dart`) for this API. |

---

## 2. Method Annotations (HTTP Verbs)

Every API method must have exactly one verb annotation.

| Annotation | Path Support | Body Allowed |
| :--- | :--- | :--- |
| `@GET(path)` | Yes | No |
| `@POST(path)` | Yes | Yes |
| `@PUT(path)` | Yes | Yes |
| `@PATCH(path)` | Yes | Yes |
| `@DELETE(path)` | Yes | Optional |
| `@HEAD(path)` | Yes | No |
| `@OPTIONS(path)` | Yes | No |

---

## 3. Parameter Annotations

### A. Path & Query
- **`@Path([String? name])`**: Maps parameter to `{name}` in the URL path. If name is omitted, uses parameter name.
- **`@Query(String name)`**: Adds a query parameter. Supports primitives and `List<Primitive>`.
- **`@Queries()`**: Maps a `Map<String, dynamic>` parameter to multiple query parameters.

### B. Headers
- **`@Header(String name)`**: Maps parameter to a request header.
- **`@Headers(Map<String, String> values)`**: **Method-level** annotation for static headers.
- **`@HeaderMap()`**: Maps a `Map<String, String>` parameter to multiple request headers.

### C. Body & Forms
- **`@Body()`**: Encodes the parameter as the request body. Only one allowed per method.
- **`@Field(String name)`**: Adds a field to `application/x-www-form-urlencoded`. Requires `@FormUrlEncoded()` on method.
- **`@Part(String name)`**: Adds a part to `multipart/form-data`. Requires `@MultiPart()` on method.

### D. Dio Extensions
- **`@Extra(String key)`**: Maps parameter to `RequestOptions.extra[key]`. Useful for passing data to interceptors.

---

## 4. Specialized Method Annotations

- **`@FormUrlEncoded()`**: Sets content-type to `application/x-www-form-urlencoded`.
- **`@MultiPart()`**: Sets content-type to `multipart/form-data`.
- **`@HttpParse(thread)`**: Overrides the class-level parsing strategy for a single method.

---

## 5. Implementation Rules
1. **URL Interpolation**: Paths like `/users/{id}` must have a matching `@Path('id')` parameter.
2. **Encoding**: All `@Path` and `@Query` values must be escaped using `Uri.encodeComponent`.
3. **Validation**: Dust must error if `@Body` is used on a `GET` or `HEAD` request.
4. **Default Values**: Standard Dart default parameter values are supported and respected during request construction.
5. **Nullability**: Nullable parameters are omitted from the request (e.g., a null `@Query` parameter will not be added to the URL).
