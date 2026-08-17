import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// CORS is a browser-side check, so every mistake here is invisible on the
/// server and fatal in the browser: a missing header is an opaque network
/// error, a wrong one is a hole. These pin the cases that are easy to get
/// wrong — refused origins, credentialed wildcards, `Vary`, and the difference
/// between a preflight and an ordinary `OPTIONS`.

void main() {
  const app = 'https://app.example';
  const other = 'https://evil.example';

  Handler serve(Cors cors, {Handler? inner}) {
    return cors.toMiddleware()(
      inner ?? (request) async => Response.ok('body'),
    );
  }

  Future<Response> send(
    Handler handler, {
    String method = 'GET',
    String? origin,
    String? requestMethod,
    String? requestHeaders,
  }) {
    return Future.sync(
      () => handler(
        request(
          method,
          '/',
          headers: {
            if (origin != null) 'origin': origin,
            if (requestMethod != null)
              'access-control-request-method': requestMethod,
            if (requestHeaders != null)
              'access-control-request-headers': requestHeaders,
          },
        ),
      ),
    );
  }

  group('an ordinary request', () {
    test('is allowed from anywhere by default', () async {
      final response = await send(serve(Cors()), origin: app);

      expect(response.headers['access-control-allow-origin'], '*');
    });

    test('still runs the handler', () async {
      final response = await send(serve(Cors()), origin: app);

      expect(await response.readAsString(), 'body');
    });

    test('is answered even with no Origin header at all', () async {
      // A same-origin request or a curl call sends no Origin; it must not be
      // treated as a refusal.
      final response = await send(serve(Cors()));

      expect(response.statusCode, 200);
      expect(await response.readAsString(), 'body');
    });

    test('names a listed origin back, rather than a wildcard', () async {
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await send(serve(cors), origin: app);

      expect(response.headers['access-control-allow-origin'], app);
    });

    test('gives an unlisted origin no allowance at all', () async {
      // No header is what makes the browser block it. A wrong header would be
      // worse than none.
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await send(serve(cors), origin: other);

      expect(response.headers, isNot(contains('access-control-allow-origin')));
    });

    test('compares origins case-sensitively, as the specification does',
        () async {
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await send(serve(cors), origin: 'https://App.example');

      expect(response.headers, isNot(contains('access-control-allow-origin')));
    });

    test('accepts whatever a predicate accepts', () async {
      final cors = Cors(
        origins: AllowedOrigins.matching((origin) => origin.endsWith('.dev')),
      );

      expect(
        (await send(serve(cors), origin: 'https://a.dev'))
            .headers['access-control-allow-origin'],
        'https://a.dev',
      );
      expect(
        (await send(serve(cors), origin: 'https://a.com')).headers,
        isNot(contains('access-control-allow-origin')),
      );
    });

    test('exposes the headers it was told to', () async {
      final cors = Cors(exposeHeaders: const {'x-request-id', 'traceparent'});
      final response = await send(serve(cors), origin: app);

      expect(
        response.headers['access-control-expose-headers'],
        'x-request-id, traceparent',
      );
    });

    test('says nothing about exposed headers when there are none', () async {
      final response = await send(serve(Cors()), origin: app);

      expect(
        response.headers,
        isNot(contains('access-control-expose-headers')),
      );
    });

    test('keeps the allowance on a failure the handler produced', () async {
      // A 401 without CORS headers reaches the browser as a network error
      // rather than as "you are not signed in".
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final handler = serve(
        cors,
        inner: (request) async =>
            const Rejection.unauthorized('nope').intoResponse(),
      );

      final response = await send(handler, origin: app);

      expect(response.statusCode, 401);
      expect(response.headers['access-control-allow-origin'], app);
    });
  });

  group('Vary', () {
    test('is set when the answer depends on the origin', () async {
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await send(serve(cors), origin: app);

      expect(response.headers['vary'], 'Origin');
    });

    test('is set even when the origin was refused', () async {
      // The refusal is still origin-dependent, so a cache must not reuse it.
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await send(serve(cors), origin: other);

      expect(response.headers['vary'], 'Origin');
    });

    test('is absent for a wildcard, which depends on nothing', () async {
      final response = await send(serve(Cors()), origin: app);

      expect(response.headers, isNot(contains('vary')));
    });
  });

  group('a preflight', () {
    Future<Response> preflight(Cors cors, {String? origin = app}) => send(
          serve(cors, inner: (request) async => fail('handler must not run')),
          method: 'OPTIONS',
          origin: origin,
          requestMethod: 'POST',
          requestHeaders: 'content-type',
        );

    test('is answered without running the handler', () async {
      expect((await preflight(Cors())).statusCode, 204);
    });

    test('names the methods it allows', () async {
      final response = await preflight(Cors(methods: const {'GET', 'POST'}));

      expect(response.headers['access-control-allow-methods'], 'GET, POST');
    });

    test('names the headers it allows', () async {
      final response =
          await preflight(Cors(headers: const {'content-type', 'x-token'}));

      expect(
        response.headers['access-control-allow-headers'],
        'content-type, x-token',
      );
    });

    test('offers a cache lifetime when one was set', () async {
      final response =
          await preflight(Cors(maxAge: const Duration(minutes: 10)));

      expect(response.headers['access-control-max-age'], '600');
    });

    test('offers none when none was set', () async {
      expect(
        (await preflight(Cors())).headers,
        isNot(contains('access-control-max-age')),
      );
    });

    test('grants a refused origin nothing but an answer', () async {
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await preflight(cors, origin: other);

      expect(response.statusCode, 204);
      expect(response.headers, isNot(contains('access-control-allow-origin')));
      expect(response.headers, isNot(contains('access-control-allow-methods')));
    });

    test('varies on everything the answer read', () async {
      final cors = Cors(origins: const AllowedOrigins.only({app}));
      final response = await preflight(cors);

      expect(
        response.headers['vary'],
        'Origin, Access-Control-Request-Method, Access-Control-Request-Headers',
      );
    });
  });

  group('an OPTIONS that is not a preflight', () {
    test('reaches the handler, because it is asking what the path serves',
        () async {
      // Without `Access-Control-Request-Method` this is an ordinary OPTIONS,
      // and swallowing it would break a client asking for `Allow`.
      var reached = false;
      final handler = serve(
        Cors(),
        inner: (request) async {
          reached = true;
          return Response.ok('allowed');
        },
      );

      final response = await send(handler, method: 'OPTIONS', origin: app);

      expect(reached, isTrue);
      expect(await response.readAsString(), 'allowed');
    });
  });

  group('credentials', () {
    test('are announced when asked for', () async {
      final cors = Cors(
        origins: const AllowedOrigins.only({app}),
        credentials: true,
      );
      final response = await send(serve(cors), origin: app);

      expect(response.headers['access-control-allow-credentials'], 'true');
    });

    test('are absent by default', () async {
      final response = await send(serve(Cors()), origin: app);

      expect(
        response.headers,
        isNot(contains('access-control-allow-credentials')),
      );
    });

    test('cannot be combined with a wildcard origin', () async {
      // A browser refuses a credentialed response allowed to `*`, so this is
      // caught at construction rather than becoming a silent failure in the
      // one environment that matters.
      expect(
        () => Cors(credentials: true),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('are fine with a named origin', () {
      expect(
        () => Cors(
          origins: const AllowedOrigins.only({app}),
          credentials: true,
        ),
        returnsNormally,
      );
    });
  });

  group('inside a router', () {
    test('answers a preflight before the route is even matched', () async {
      final app = Router()
        ..layer(Cors())
        ..route('/todos', get((request) async => {'ok': true}));

      final response = await app.handler(
        request(
          'OPTIONS',
          '/nothing-here',
          headers: const {
            'origin': 'https://app.example',
            'access-control-request-method': 'GET',
          },
        ),
      );

      expect(response.statusCode, 204);
    });

    test('adds the allowance to a real route', () async {
      final app = Router()
        ..layer(Cors())
        ..route('/todos', get((request) async => {'ok': true}));

      final response = await app.handler(
        request('GET', '/todos', headers: const {'origin': 'https://a.test'}),
      );

      expect(response.headers['access-control-allow-origin'], '*');
    });
  });
}
