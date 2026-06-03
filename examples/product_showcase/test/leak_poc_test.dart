import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:product_showcase/api/json_placeholder_api.dart';
import 'package:test/test.dart';

void main() {
  test('stream endpoint yields response body chunks', () async {
    final dio = Dio()
      ..interceptors.add(
        InterceptorsWrapper(
          onRequest: (options, handler) {
            handler.resolve(
              Response<dynamic>(
                requestOptions: options,
                data: ResponseBody(
                  Stream.fromIterable([
                    Uint8List.fromList([1, 2, 3]),
                  ]),
                  200,
                ),
              ),
            );
          },
        ),
      );

    final api = JsonPlaceholderApi(dio);
    final chunks = await api.streamPostsBytes().toList();

    expect(chunks, hasLength(1));
    expect(chunks.single, <int>[1, 2, 3]);
  });
}
