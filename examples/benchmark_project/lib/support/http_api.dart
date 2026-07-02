import 'package:dust_dart/http.dart';

import 'http_post.dart';

part 'http_api.g.dart';

/// Benchmark HTTP API contract for the benchmark example.
@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'x-suite': 'benchmark'},
  generateTest: true,
)
abstract interface class BenchmarkHttpApi {
  factory BenchmarkHttpApi(Dio dio, {String? baseUrl}) = _$BenchmarkHttpApi;

  /// Lists posts.
  @GET('/posts')
  Future<List<HttpPost>> listPosts({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  /// Fetches a post.
  @GET('/posts/{id}')
  Future<Response<HttpPost>> fetchPost(@Path() int id);

  /// Creates a post.
  @POST('/posts')
  Future<Map<String, dynamic>> createPost(@Body() Map<String, dynamic> payload);
}
