import 'package:dust_server/server.dart';

/// A PNG signature followed by bytes that are not valid UTF-8.
const pngHeader = <int>[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0xFF, 0xFE, 0x00];

/// The router every TestClient suite exercises.
///
/// It was a local inside `main()` while the suites shared one file.

Router app() {
  final router = Router();
  router.route(
    '/hello',
    get((_) async => textResponse('world')),
  );
  router.route(
    '/echo',
    post((request) async {
      final body = await request.readAsString();
      return jsonResponse({'echo': body});
    }),
  );
  router.route(
    '/greet/{name}',
    get((request) async {
      final params = pathParametersOf(request);
      return textResponse('hello ${params['name']}');
    }),
  );
  router.route(
    '/status/{code}',
    get((request) async {
      final params = pathParametersOf(request);
      final code = int.parse(params['code']!);
      return Response(code, body: 'status $code');
    }),
  );
  router.route(
    '/headers',
    get((request) async {
      final auth = request.headers['authorization'] ?? 'none';
      return jsonResponse({'authorization': auth});
    }),
  );
  router.route(
    '/cookie-echo',
    get((request) async {
      final cookie = request.headers['cookie'] ?? 'none';
      return Response.ok(cookie);
    }),
  );
  router.route(
    '/search',
    get((request) async {
      final uri = request.requestedUri;
      return jsonResponse({
        'q': uri.queryParameters['q'] ?? '',
        'page': uri.queryParameters['page'] ?? '',
      });
    }),
  );
  router.route(
    '/empty',
    get((_) async => Response(204)),
  );
  router.route(
    '/verb',
    any((request) async => textResponse(request.method)),
  );
  router.route(
    '/list',
    get((_) async => jsonResponse([1, 2, 3])),
  );
  router.route(
    '/binary',
    get(
      (_) async => Response.ok(
        pngHeader,
        headers: {'content-type': 'application/octet-stream'},
      ),
    ),
  );
  router.route(
    '/mixed-case-headers',
    get(
      (_) async => Response.ok(
        '{"ok":true}',
        headers: {
          'Content-Type': 'application/json',
          'X-Trace-Id': 'abc123',
        },
      ),
    ),
  );
  router.route(
    '/two-cookies',
    get(
      (_) async => Response.ok(
        'set',
        headers: {
          'set-cookie': [
            'session=abc; Path=/; HttpOnly',
            'prefs=dark; Path=/; Expires=Wed, 09 Jun 2027 10:18:14 GMT',
          ],
        },
      ),
    ),
  );
  return router;
}
