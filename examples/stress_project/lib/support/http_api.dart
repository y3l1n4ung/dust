import 'package:dio/dio.dart' hide Headers;
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

import 'http_post.dart';

part 'http_api.g.dart';

@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'x-suite': 'stress'},
)
@GenerateTest()
abstract interface class StressHttpApi {
  factory StressHttpApi(Dio dio, {String? baseUrl}) = _$StressHttpApi;

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
