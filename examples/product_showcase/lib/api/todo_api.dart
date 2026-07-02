import 'dart:isolate';

import 'package:dust_dart/http.dart';

import '../models/todo.dart';

part 'todo_api.g.dart';

/// To-do API contract for the product showcase example.
@HttpClient(
  baseUrl: 'https://api.todos.com',
  parseThread: HttpParseThread.isolate,
  headers: {'x-api-version': '2026-05'},
  generateTest: true,
)
abstract interface class TodoApi {
  factory TodoApi(Dio dio, {String? baseUrl}) = _$TodoApi;

  /// Lists to-do items.
  @Headers({'x-endpoint': 'todos'})
  @GET('/todos')
  Future<List<Todo>> list({
    @Query('userId') int? userId,
    @Query('page') int page = 1,
    @Header('x-trace-id') String traceId = 'showcase-default',
  });

  /// Fetches a raw to-do response.
  @GET('/todos/{id}')
  Future<Response<Todo>> fetchRaw(
    @Path() String id, {
    CancelToken? cancelToken,
    @Extra('retryable') bool? retryable,
  });

  /// Creates a to-do item.
  @POST('/todos')
  Future<Todo> create(@Body() TodoCreate request);

  /// Renames a to-do item.
  @FormUrlEncoded()
  @PATCH('/todos/{id}')
  Future<Todo> rename(@Path() String id, @Field('title') String title);

  /// Updates a to-do item.
  @PUT('/todos/{id}')
  Future<Todo> update(
    @Path() String id,
    @Body() TodoUpdate request, {
    @HeaderMap() Map<String, String>? headers,
  });

  /// Deletes a to-do item.
  @DELETE('/todos/{id}')
  Future<void> delete(
    @Path() String id, {
    @Queries() Map<String, dynamic>? audit,
  });
}
