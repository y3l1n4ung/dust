import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:state_management_prototype/shared/api/prototype_api.dart';

void main() {
  group('PrototypeApi generated client', () {
    late Dio dio;
    late PrototypeApi api;

    setUp(() {
      dio = Dio()..httpClientAdapter = _FixtureAdapter();
      api = PrototypeApi(dio, baseUrl: 'https://example.test');
    });

    test('fetchUser decodes generated serde model', () async {
      final user = await api.fetchUser(1);

      expect(user.id, 1);
      expect(user.name, 'Ada Lovelace');
      expect(user.company.name, 'Dust Labs');
    });

    test(
      'listTodos sends query params and decodes Dust-specific fields',
      () async {
        final todos = await api.listTodos(userId: 1, limit: 2);

        expect(todos, hasLength(2));
        expect(todos.every((todo) => todo.userId == 1), isTrue);
        expect(todos.first.lane, 'Design');
        expect(todos.first.priority, 'High');
      },
    );
  });
}

final class _FixtureAdapter implements HttpClientAdapter {
  @override
  void close({bool force = false}) {}

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    final path = options.uri.path;
    final query = options.uri.queryParameters;
    final Object body = switch (path) {
      '/users/1' => {
        'id': 1,
        'name': 'Ada Lovelace',
        'username': 'ada',
        'email': 'ada@dust.dev',
        'phone': '555-0100',
        'website': 'dust.dev',
        'company': {'name': 'Dust Labs', 'catchPhrase': 'Ship sharp code'},
      },
      '/todos' => [
        {
          'userId': int.parse(query['userId'] ?? '1'),
          'id': 1,
          'title': 'Review design QA',
          'completed': false,
          'lane': 'Design',
          'priority': 'High',
        },
        {
          'userId': int.parse(query['userId'] ?? '1'),
          'id': 2,
          'title': 'Ship release notes',
          'completed': true,
          'lane': 'Platform',
          'priority': 'Medium',
        },
      ].take(int.parse(query['_limit'] ?? '2')).toList(),
      _ => throw StateError('Unexpected fixture request: $path'),
    };

    return ResponseBody.fromString(
      jsonEncode(body),
      200,
      headers: {
        Headers.contentTypeHeader: [Headers.jsonContentType],
      },
    );
  }
}
