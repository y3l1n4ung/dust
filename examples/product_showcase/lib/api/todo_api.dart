import 'dart:isolate';

import 'package:dio/dio.dart' hide Headers;
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

import '../models/todo.dart';

part 'todo_api.g.dart';

@HttpClient(
  baseUrl: 'https://api.todos.com',
  parseThread: DustParseThread.isolate,
  headers: {'x-api-version': '2026-05'},
)
@GenerateTest()
abstract interface class TodoApi {
  factory TodoApi(Dio dio, {String? baseUrl}) = _$TodoApi;

  @Headers({'x-endpoint': 'todos'})
  @GET('/todos')
  Future<List<Todo>> list({
    @Query('userId') int? userId,
    @Query('page') int? page,
    @Header('x-trace-id') String? traceId,
  });

  @GET('/todos/{id}')
  Future<Response<Todo>> fetchRaw(
    @Path() String id, {
    CancelToken? cancelToken,
    @Extra('retryable') bool? retryable,
  });

  @POST('/todos')
  Future<Todo> create(@Body() TodoCreate request);

  @FormUrlEncoded()
  @PATCH('/todos/{id}')
  Future<Todo> rename(@Path() String id, @Field('title') String title);

  @PUT('/todos/{id}')
  Future<Todo> update(
    @Path() String id,
    @Body() TodoUpdate request, {
    @HeaderMap() Map<String, String>? headers,
  });

  @DELETE('/todos/{id}')
  Future<void> delete(
    @Path() String id, {
    @Queries() Map<String, dynamic>? audit,
  });
}
