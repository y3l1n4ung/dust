import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

part 'prototype_api.g.dart';

@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'accept': 'application/json'},
)
abstract interface class PrototypeApi {
  factory PrototypeApi(Dio dio, {String? baseUrl}) = _$PrototypeApi;

  @GET('/users/{id}')
  Future<RemoteUser> fetchUser(@Path() int id);

  @GET('/todos')
  Future<List<RemoteTodo>> listTodos({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });
}
