import 'package:dust_server/annotations.dart' as annotations;
import 'package:dust_server/extraction.dart' as extraction;
import 'package:dust_server/response.dart' as response;
import 'package:dust_server/router.dart' as router;
import 'package:dust_server/server.dart' as server;
import 'package:test/test.dart';

/// Each entry point has to stand on its own: a file that imports only
/// `router.dart` must not need `server.dart` to compile. These references are
/// the assertion; if a library stopped exporting something, this file would
/// fail to analyze.

void main() {
  group('router.dart', () {
    test('carries the route tree and its composition', () {
      final app = router.Router()
        ..route('/a', router.get((request) async => router.Response.ok('a')))
        ..nest('/api', router.Router())
        ..layer(
          (router.Handler inner) => (router.Request request) => inner(request),
        )
        ..withState(const _Repo())
        ..fallback((request) async => router.Response.notFound('gone'));

      expect(app.describe().single.path, '/a');
      expect(app.handler, isA<router.Handler>());
      expect(router.normalizePrefix('api/'), '/api');
      expect(const router.MethodRouter().handlers, isEmpty);
      expect(router.serve, isA<Function>());
    });

    test('names every verb builder', () {
      expect(
        [
          router.get,
          router.post,
          router.put,
          router.patch,
          router.delete,
          router.head,
          router.options,
        ],
        hasLength(7),
      );
    });
  });

  group('extraction.dart', () {
    test('carries the interfaces, built-ins, and fp types', () {
      expect(const extraction.PathExtractable<String>('id'),
          isA<extraction.FromRequestParts<String>>());
      expect(const extraction.RawBodyExtractable(),
          isA<extraction.FromRequest<Object>>());
      expect(const extraction.StateExtractable<_Repo>(), isNotNull);
      expect(const extraction.QueryExtractable<int>('n'), isNotNull);
      expect(const extraction.HeaderMapExtractable(), isNotNull);
      expect(const extraction.JsonExtractable<Object>(_decode), isNotNull);
      expect(const extraction.FormExtractable(), isNotNull);
      expect(const extraction.MultipartExtractable(), isNotNull);
      expect(const extraction.ContextExtractable<String>('k'), isNotNull);
      expect(const extraction.PeerExtractable(), isNotNull);
      expect(extraction.defaultBodyLimit, 1024 * 1024);
      expect(extraction.Ok<int, String>(1).isOk, isTrue);
    });
  });

  group('response.dart', () {
    test('carries rejections, encoders, and the error sink', () {
      expect(const response.Rejection.notFound('gone').status, 404);
      expect(response.jsonResponse({'a': 1}).statusCode, 200);
      expect(response.noContent().statusCode, 204);
      expect(response.textResponse('hi').statusCode, 200);
      expect(response.guard, isA<Function>());
      expect(response.ServerErrors.report, isA<Function>());
      expect(const _Gone(), isA<response.IntoResponse>());
    });
  });

  group('annotations.dart', () {
    test('carries every annotation the generator reads', () {
      expect(const annotations.Controller('/todos').path, '/todos');
      expect(const annotations.GET('/').method, 'GET');
      expect(const annotations.Routes('/notes').path, '/notes');
      expect(const annotations.Extract(_Repo).extractor, _Repo);
      expect(const annotations.Path('id').name, 'id');
      expect(const annotations.State(), isNotNull);
      expect(const annotations.Body(), isNotNull);
      expect(const annotations.MultiPart(), isNotNull);
    });
  });

  group('server.dart', () {
    test('re-exports all four', () {
      expect(server.Router, isNotNull);
      expect(server.Rejection, isNotNull);
      expect(server.Controller, isNotNull);
      expect(server.PathExtractable, isNotNull);
    });
  });
}

final class _Repo {
  const _Repo();
}

final class _Gone implements response.IntoResponse {
  const _Gone();

  @override
  response.Response intoResponse() => response.Response.notFound('gone');
}

Object _decode(Map<String, Object?> json) => json;
