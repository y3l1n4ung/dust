import 'package:dust_dart/http.dart';

import '../models/remote_comment.dart';
import '../models/remote_post.dart';

part 'json_placeholder_api.g.dart';

/// JSON placeholder API contract for the product showcase example.
@HttpClient(
  baseUrl: 'https://jsonplaceholder.typicode.com',
  headers: {'accept': 'application/json'},
  generateTest: true,
)
abstract interface class JsonPlaceholderApi {
  factory JsonPlaceholderApi(Dio dio, {String? baseUrl}) = _$JsonPlaceholderApi;

  /// Lists remote posts.
  @GET('/posts')
  Future<List<RemotePost>> listPosts({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  /// Streams remote posts as a raw response body.
  @GET('/posts')
  Future<ResponseBody> streamPostsRaw({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  /// Streams remote posts as bytes.
  @GET('/posts')
  Stream<List<int>> streamPostsBytes({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  /// Streams remote posts as text.
  @GET('/posts')
  Stream<String> streamPostsText({
    @Query('userId') int? userId,
    @Query('_limit') int? limit,
  });

  /// Fetches a remote post.
  @GET('/posts/{id}')
  Future<Response<RemotePost>> fetchPost(
    @Path() int id, {
    @Header('accept_encoding') String? accept,
  });

  /// Lists remote comments.
  @GET('/comments')
  Future<List<RemoteComment>> listComments({
    @Query('postId') int? postId,
    @Query('_limit') int? limit,
  });

  /// Creates a remote post.
  @POST('/posts')
  Future<RemotePost> createPost(@Body() RemotePostDraft payload);

  /// Replaces post.
  @PUT('/posts/{id}')
  Future<RemotePost> replacePost(@Path() int id, @Body() RemotePost payload);

  /// Patches a remote post.
  @PATCH('/posts/{id}')
  Future<Map<String, dynamic>> patchPost(
    @Path() int id,
    @Body() Map<String, dynamic> payload,
  );

  /// Deletes a remote post.
  @DELETE('/posts/{id}')
  Future<Response<Map<String, dynamic>>> deletePost(@Path() int id);

  /// Uploads post with file.
  @POST('/posts')
  @MultiPart()
  Future<RemotePost> uploadPostWithFile(
    @Part('userId') int userId,
    @Part('title') String title,
    @Part('body') String body,
    @Part('file') MultipartFile file,
  );
}
