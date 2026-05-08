# HttpClient Specification: Feature Examples

## 1. Complete CRUD: Todo List

Demonstrates GET, POST, PUT, DELETE with query parameters.

```dart
@HttpClient(baseUrl: 'https://api.todos.com')
abstract interface class TodoApi {
  factory TodoApi(Dio dio, {String? baseUrl}) = _$TodoApi;

  @GET('/todos')
  Future<List<Todo>> list({@Query('userId') int? userId});

  @POST('/todos')
  Future<Todo> create(@Body() TodoCreate request);

  @PUT('/todos/{id}')
  Future<Todo> update(@Path() int id, @Body() TodoUpdate request);

  @DELETE('/todos/{id}')
  Future<void> delete(@Path() int id);
}
```

---

## 2. Performance: Large Data Fetch

Demonstrates method-level isolate override.

```dart
@GET('/analytics/heavy-report')
@HttpParse(thread: DustParseThread.isolate)
Future<AnalyticsReport> getHeavyReport();
```

---

## 3. File Upload: Profile Avatar

Demonstrates Multipart support with progress tracking.

```dart
@POST('/profiles/avatar')
@MultiPart()
Future<void> uploadAvatar(
  @Part('image') File file,
  {ProgressCallback? onSendProgress}
);
```

---

## 4. Form Submission: Legacy Login

Demonstrates FormUrlEncoded support.

```dart
@POST('/login')
@FormUrlEncoded()
Future<AuthResponse> login(
  @Field('username') String user,
  @Field('password') String pass,
);
```
