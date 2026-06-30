import 'dart:isolate';

import 'package:dust_dart/http.dart';

part 'http_fixture_api.g.dart';

@HttpClient(
  baseUrl: 'https://api.example.com',
  parseThread: HttpParseThread.isolate,
  headers: {'accept': 'application/json'},
  generateTest: true,
)
abstract interface class HttpFixtureApi {
  factory HttpFixtureApi(Dio dio, {String? baseUrl}) = _$HttpFixtureApi;

  @GET('/users/{id}')
  Future<HttpUser> fetchUser(
    @Path() String id, {
    @Query('includePosts') bool includePosts = false,
    @Header('x-trace-id') String? traceId,
  });

  @POST('/users')
  Future<Map<String, dynamic>> createUser(
    @Body() Map<String, dynamic> payload, {
    @Header('x-trace-id') required String traceId,
  });
}

final class HttpUser {
  const HttpUser({required this.id, required this.name});

  factory HttpUser.fromJson(Map<String, dynamic> json) {
    return HttpUser(
      id: json['id'] as String,
      name: json['name'] as String,
    );
  }

  final String id;
  final String name;
}
