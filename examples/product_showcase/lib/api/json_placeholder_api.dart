import 'dart:convert';

import 'package:dio/dio.dart' hide Headers;
import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';

import '../models/remote_comment.dart';
import '../models/remote_post.dart';

part 'json_placeholder_api.g.dart';

@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'accept': 'application/json'},
)
@GenerateTest()
abstract interface class JsonPlaceholderApi {
  factory JsonPlaceholderApi(Dio dio, {String? baseUrl}) = _$JsonPlaceholderApi;

  @GET('/posts')
  Future<List<RemotePost>> listPosts({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  @GET('/posts')
  Future<ResponseBody> streamPostsRaw({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  @GET('/posts')
  Stream<List<int>> streamPostsBytes({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  @GET('/posts')
  Stream<String> streamPostsText({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  @GET('/posts/{id}')
  Future<Response<RemotePost>> fetchPost(@Path() int id);

  @GET('/comments')
  Future<List<RemoteComment>> listComments({
    @Query('postId') int? postId,
    @Query('_limit') int? limit,
  });

  @POST('/posts')
  Future<RemotePost> createPost(@Body() RemotePostDraft payload);

  @PUT('/posts/{id}')
  Future<RemotePost> replacePost(@Path() int id, @Body() RemotePost payload);

  @PATCH('/posts/{id}')
  Future<Map<String, dynamic>> patchPost(
    @Path() int id,
    @Body() Map<String, dynamic> payload,
  );

  @DELETE('/posts/{id}')
  Future<Response<Map<String, dynamic>>> deletePost(@Path() int id);
}
