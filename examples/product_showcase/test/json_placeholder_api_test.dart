import 'dart:convert';
import 'dart:io';

import 'package:dio/dio.dart';
import 'package:product_showcase/product_showcase.dart';
import 'package:test/test.dart';

void main() {
  test(
    'json placeholder client covers working online CRUD-style flows',
    () async {
      final requests = <RequestOptions>[];
      final dio = Dio()
        ..interceptors.add(
          InterceptorsWrapper(
            onRequest: (options, handler) {
              requests.add(options);
              handler.resolve(
                Response<dynamic>(
                  requestOptions: options,
                  statusCode: options.path == '/posts/1' ? 200 : 201,
                  data: _responseFor(options),
                ),
              );
            },
          ),
        );

      final api = JsonPlaceholderApi(dio);
      final posts = await api.listPosts(userId: 1, limit: 1);
      final streamed = await api.streamPostsRaw(userId: 1, limit: 1);
      final streamedBytes = await _readByteStream(
        api.streamPostsBytes(userId: 1, limit: 1),
      );
      final streamedTextChunks =
          await api.streamPostsText(userId: 1, limit: 1).toList();
      final raw = await api.fetchPost(1);
      final comments = await api.listComments(postId: 1, limit: 2);
      final created = await api.createPost(
        RemotePostDraft(
          userId: 1,
          title: 'Dust keeps HTTP clients honest',
          body: 'Shared rendering now lives outside the plugin.',
        ),
      );
      final replaced = await api.replacePost(
        1,
        RemotePost(
          id: 1,
          userId: 1,
          title: 'Dust replaces a post',
          body: 'PUT keeps the request body intact.',
        ),
      );
      final patched = await api.patchPost(1, {
        'title': 'Dust patches just the title',
      });
      final deleted = await api.deletePost(1);
      final streamedText = await _readResponseBody(streamed);

      expect(posts.single.title, 'Dust keeps HTTP clients honest');
      expect(streamedText, contains('Dust keeps HTTP clients honest'));
      expect(
        utf8.decode(streamedBytes),
        contains('Dust keeps HTTP clients honest'),
      );
      expect(
        streamedTextChunks.join(),
        contains('Dust keeps HTTP clients honest'),
      );
      expect(raw.statusCode, 200);
      expect(raw.data?.id, 1);
      expect(comments.map((comment) => comment.id), [1, 2]);
      expect(created.id, 101);
      expect(created.title, 'Dust keeps HTTP clients honest');
      expect(replaced.title, 'Dust replaces a post');
      expect(patched['title'], 'Dust patches just the title');
      expect(deleted.statusCode, 200);
      expect(deleted.data, isEmpty);
      expect(requests.map((request) => request.path), [
        '/posts',
        '/posts',
        '/posts',
        '/posts',
        '/posts/1',
        '/comments',
        '/posts',
        '/posts/1',
        '/posts/1',
        '/posts/1',
      ]);
      expect(requests.first.queryParameters, {'userId': 1, '_limit': 1});
      expect(requests[1].responseType, ResponseType.stream);
      expect(requests[2].responseType, ResponseType.stream);
      expect(requests[3].responseType, ResponseType.stream);
      expect(requests[5].queryParameters, {'postId': 1, '_limit': 2});
      expect(requests[6].data, {
        'userId': 1,
        'title': 'Dust keeps HTTP clients honest',
        'body': 'Shared rendering now lives outside the plugin.',
      });
      expect(requests[7].data, {
        'id': 1,
        'userId': 1,
        'title': 'Dust replaces a post',
        'body': 'PUT keeps the request body intact.',
      });
      expect(requests[8].data, {'title': 'Dust patches just the title'});
    },
  );

  final runOnline = Platform.environment['DUST_RUN_ONLINE_HTTP_TESTS'] == '1';
  test(
    'json placeholder live smoke test',
    () async {
      final api = JsonPlaceholderApi(Dio());
      final posts = await api.listPosts(userId: 1, limit: 1);
      final streamed = await api.streamPostsRaw(userId: 1, limit: 1);
      final streamedBytes = await _readByteStream(
        api.streamPostsBytes(userId: 1, limit: 1),
      );
      final streamedTextChunks =
          await api.streamPostsText(userId: 1, limit: 1).toList();
      final response = await api.fetchPost(1);
      final comments = await api.listComments(postId: 1, limit: 2);
      final created = await api.createPost(
        RemotePostDraft(
          userId: 1,
          title: 'Dust live create',
          body: 'jsonplaceholder accepts writes without persistence.',
        ),
      );
      final replaced = await api.replacePost(
        1,
        RemotePost(
          id: 1,
          userId: 1,
          title: 'Dust live replace',
          body: 'A real fake-online PUT flow.',
        ),
      );
      final patched = await api.patchPost(1, {'title': 'Dust live patch'});
      final deleted = await api.deletePost(1);
      final streamedText = await _readResponseBody(streamed);

      expect(posts, isNotEmpty);
      expect(posts.first.userId, 1);
      expect(streamedText, contains('"userId"'));
      expect(utf8.decode(streamedBytes), contains('"userId"'));
      expect(streamedTextChunks.join(), contains('"userId"'));
      expect(response.statusCode, 200);
      expect(response.data?.id, 1);
      expect(comments, isNotEmpty);
      expect(comments.first.postId, 1);
      expect(created.id, isA<int>());
      expect(replaced.title, 'Dust live replace');
      expect(patched['title'], 'Dust live patch');
      expect(deleted.statusCode, 200);
      expect(deleted.data, isA<Map<String, dynamic>>());
    },
    skip: runOnline
        ? false
        : 'set DUST_RUN_ONLINE_HTTP_TESTS=1 to run live HTTP smoke coverage',
  );
}

dynamic _responseFor(RequestOptions options) {
  if (options.method == 'GET' &&
      options.path == '/posts' &&
      options.responseType == ResponseType.stream) {
    return ResponseBody.fromString(
      jsonEncode([
        {
          'userId': 1,
          'id': 1,
          'title': 'Dust keeps HTTP clients honest',
          'body': 'Shared rendering now lives outside the plugin.',
        },
      ]),
      200,
      headers: {
        Headers.contentTypeHeader: [Headers.jsonContentType],
      },
    );
  }
  if (options.method == 'GET' && options.path == '/posts') {
    return [
      {
        'userId': 1,
        'id': 1,
        'title': 'Dust keeps HTTP clients honest',
        'body': 'Shared rendering now lives outside the plugin.',
      },
    ];
  }
  if (options.method == 'GET' && options.path == '/posts/1') {
    return {
      'userId': 1,
      'id': 1,
      'title': 'Dust keeps HTTP clients honest',
      'body': 'Shared rendering now lives outside the plugin.',
    };
  }
  if (options.method == 'GET' && options.path == '/comments') {
    return [
      {
        'postId': 1,
        'id': 1,
        'name': 'Dust comment one',
        'email': 'first@example.com',
        'body': 'Generated HTTP clients should stay honest.',
      },
      {
        'postId': 1,
        'id': 2,
        'name': 'Dust comment two',
        'email': 'second@example.com',
        'body': 'This proves nested resources decode correctly.',
      },
    ];
  }

  if (options.method == 'DELETE' && options.path == '/posts/1') {
    return const <String, dynamic>{};
  }

  final payload = Map<String, dynamic>.from(options.data as Map);
  return <String, dynamic>{
    'id': payload['id'] ?? 101,
    'userId': payload['userId'] ?? 1,
    'title': payload['title'] ?? 'untitled',
    'body': payload['body'] ?? 'body',
  };
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
