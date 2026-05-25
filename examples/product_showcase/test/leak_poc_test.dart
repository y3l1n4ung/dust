import 'dart:async';
import 'dart:typed_data';
import 'package:dio/dio.dart';
import 'package:product_showcase/api/json_placeholder_api.dart';
import 'package:test/test.dart';

class CloseTrackingResponseBody extends ResponseBody {
  bool closeCalled = false;

  CloseTrackingResponseBody(Stream<Uint8List> stream) : super(stream, 200);

  @override
  void close() {
    closeCalled = true;
    super.close();
  }
}

void main() {
  test('reproduce HTTP connection leak (close not called on Stream endpoints)', () async {
    late CloseTrackingResponseBody trackingBody;

    final dio = Dio()
      ..interceptors.add(
        InterceptorsWrapper(
          onRequest: (options, handler) {
            trackingBody = CloseTrackingResponseBody(
              Stream.fromIterable([
                Uint8List.fromList([1, 2, 3])
              ]),
            );
            handler.resolve(
              Response<dynamic>(
                requestOptions: options,
                data: trackingBody,
              ),
            );
          },
        ),
      );

    final api = JsonPlaceholderApi(dio);

    // This calls api.streamPostsBytes which uses yield* in generated code
    final stream = api.streamPostsBytes();
    await for (final _ in stream) {
      // consume
    }

    expect(
      trackingBody.closeCalled,
      isTrue,
      reason: 'ResponseBody.close() should be called after stream is exhausted',
    );
  });
}
