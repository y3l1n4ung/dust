import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:product_showcase/product_showcase.dart';

Future<void> main() async {
  final api = JsonPlaceholderApi(Dio());

  final posts = await api.listPosts(userId: 1, limit: 2);
  final streamedPosts = await api.streamPostsRaw(userId: 1, limit: 1);
  final streamedBytes = await _readByteStream(
    api.streamPostsBytes(userId: 1, limit: 1),
  );
  final streamedTextChunks = await api
      .streamPostsText(userId: 1, limit: 1)
      .toList();
  final first = await api.fetchPost(1);
  final comments = await api.listComments(postId: 1, limit: 2);
  final created = await api.createPost(
    RemotePostDraft(
      userId: 1,
      title: 'Dust HttpClient demo',
      body: 'Generated code calling jsonplaceholder.typicode.com',
    ),
  );
  final replaced = await api.replacePost(
    1,
    RemotePost(
      id: 1,
      userId: 1,
      title: 'Dust HttpClient replaced post',
      body: 'This demonstrates a full PUT workflow.',
    ),
  );
  final patched = await api.patchPost(1, {
    'title': 'Dust HttpClient patched title',
  });
  final deleted = await api.deletePost(1);
  final streamedText = await _readResponseBody(streamedPosts);
  final streamedSnippet = streamedText.length > 80
      ? '${streamedText.substring(0, 80)}...'
      : streamedText;
  final byteSnippet = utf8.decode(streamedBytes);
  final textStreamSnippet = streamedTextChunks.join();

  print('Fetched ${posts.length} posts for user 1.');
  print('Streamed payload snippet: $streamedSnippet');
  print(
    'Byte-stream payload snippet: '
    '${byteSnippet.length > 80 ? '${byteSnippet.substring(0, 80)}...' : byteSnippet}',
  );
  print(
    'Text-stream payload snippet: '
    '${textStreamSnippet.length > 80 ? '${textStreamSnippet.substring(0, 80)}...' : textStreamSnippet}',
  );
  print('First remote title: ${first.data?.title}');
  print('Fetched ${comments.length} comments for post 1.');
  print('Created fake remote id: ${created.id}');
  print('Replaced title: ${replaced.title}');
  print('Patched title: ${patched['title']}');
  print('Delete status: ${deleted.statusCode}, body: ${deleted.data}');
}

Future<String> _readResponseBody(ResponseBody body) async {
  return utf8.decode(await _readByteStream(body.stream));
}

Future<List<int>> _readByteStream(Stream<List<int>> stream) async {
  final bytes = <int>[];
  await for (final chunk in stream) {
    bytes.addAll(chunk);
  }
  return bytes;
}
