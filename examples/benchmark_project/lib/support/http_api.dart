import 'package:dust_dart/http.dart';

import 'http_post.dart';

part 'http_api.g.dart';

@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'x-suite': 'benchmark'},
)
@GenerateTest()
abstract interface class BenchmarkHttpApi {
  factory BenchmarkHttpApi(Dio dio, {String? baseUrl}) = _$BenchmarkHttpApi;

  @GET('/posts')
  Future<List<HttpPost>> listPosts({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  @GET('/posts/{id}')
  Future<Response<HttpPost>> fetchPost(@Path() int id);

  @POST('/posts')
  Future<Map<String, dynamic>> createPost(@Body() Map<String, dynamic> payload);
}
