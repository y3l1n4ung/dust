import 'dart:isolate';

import 'package:dust_dart/http.dart';

part 'http_fixture_api.g.dart';

/// HTTP fixture API contract for the HTTP fixture app.
@HttpClient(
  baseUrl: 'https://api.example.com',
  parseThread: HttpParseThread.isolate,
  headers: {'accept': 'application/json'},
  generateTest: true,
)
abstract interface class HttpFixtureApi {
  factory HttpFixtureApi(Dio dio, {String? baseUrl}) = _$HttpFixtureApi;

  /// Fetches a user.
  @GET('/users/{id}')
  Future<HttpUser> fetchUser(
    @Path() String id, {
    @Query('includePosts') bool includePosts = false,
    @Header('x-trace-id') String? traceId,
  });

  /// Creates a user.
  @POST('/users')
  Future<Map<String, dynamic>> createUser(
    @Body() Map<String, dynamic> payload, {
    @Header('x-trace-id') required String traceId,
  });

  /// Checks encoding policy handling.
  @GET('/encoding/{slug}')
  Future<void> encodingPolicy(
    @Path() String slug, {
    @Query('tags') List<String> tags = const ['dust'],
    @Queries() required Map<String, dynamic> filters,
    @Header('x-page') int page = 1,
    @HeaderMap() required Map<String, String> headers,
  });
}

/// HTTP user model for the HTTP fixture app.
final class HttpUser {
  /// Creates a [HttpUser].
  const HttpUser({required this.id, required this.name});

  /// Creates a [HttpUser] from JSON.
  factory HttpUser.fromJson(Map<String, dynamic> json) {
    return HttpUser(
      id: json['id'] as String,
      name: json['name'] as String,
    );
  }

  /// Unique identifier.
  final String id;

  /// Name.
  final String name;
}
