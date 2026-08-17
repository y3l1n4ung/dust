import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../example/access_log.dart' as access_log;
import '../../example/bearer_auth.dart' as bearer_auth;
import '../../example/compression.dart' as compression;
import '../../example/cors.dart' as cors;
import '../../example/cookies.dart' as cookies;
import '../../example/credential_schemes.dart' as credential_schemes;
import '../../example/custom_extractor.dart' as custom_extractor;
import '../../example/customize_rejection.dart' as customize_rejection;
import '../../example/fallible_extraction.dart' as fallible_extraction;
import '../../example/form_body.dart' as form_body;
import '../../example/headers_and_host.dart' as headers_and_host;
import '../../example/hello_world.dart' as hello_world;
import '../../example/json_body.dart' as json_body;
import '../../example/multipart_form.dart' as multipart_form;
import '../../example/normalize_path.dart' as normalize_path;
import '../../example/optional_extraction.dart' as optional_extraction;
import '../../example/parse_body_by_content_type.dart' as by_content_type;
import '../../example/path_params.dart' as path_params;
import '../../example/query_params.dart' as query_params;
import '../../example/request_id.dart' as request_id;
import '../../example/request_timeout.dart' as request_timeout;
import '../../example/route_layer.dart' as route_layer;
import '../../example/routing.dart' as routing;
import '../../example/security_headers.dart' as security_headers;
import '../../example/state.dart' as state_example;
import '../../example/validation_422.dart' as validation_422;
import 'serve.dart';

/// One suite for every one-question example.
///
/// Each example gets the shortest test that proves the thing it demonstrates,
/// and they share one harness. Thirty examples with thirty harnesses is thirty
/// places for a leaked socket to hide; more to the point, an example nothing
/// serves is an example that quietly stops compiling.

void main() {
  group('hello_world', () {
    test('answers the root as text, not as a quoted JSON string', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/');

      expect(response.statusCode, 200);
      expect(response.body, 'Hello, world!');
      expect(response.headers['content-type'], startsWith('text/plain'));
    });

    test('reads the name out of the path', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/hello/ada')).body, 'Hello, ada!');
    });

    test('encodes a returned map as JSON without being asked', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/json');

      expect(response.headers['content-type'], 'application/json');
      expect(app.object(response), {'greeting': 'Hello, world!'});
    });

    test('an unmatched path answers 404 without a fallback', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/nope')).statusCode, 404);
    });
  });

  group('routing', () {
    test('nest serves the inner routes under the prefix', () async {
      final app = await example(routing.buildApp());

      expect(app.array(await app.get('/api/notes')), ['first', 'second']);
      expect(app.object(await app.get('/api/notes/7')), {'id': '7'});
    });

    test('two verbs chain onto one path', () async {
      final app = await example(routing.buildApp());

      expect((await app.get('/api/notes')).statusCode, 200);
      expect((await app.post('/api/notes', null)).statusCode, 201);
    });

    test('merge folds a router in at the same level', () async {
      final app = await example(routing.buildApp());

      expect(app.object(await app.get('/health')), {'status': 'ok'});
    });

    test('a known path with an unknown method answers 405 and Allow', () async {
      final app = await example(routing.buildApp());

      final response = await app.send('PUT', '/api/notes');

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'GET, HEAD, POST');
    });

    test('the fallback answers what matched nothing', () async {
      final app = await example(routing.buildApp());

      final response = await app.get('/nothing-here');

      expect(response.statusCode, 404);
      expect(app.object(response)['error'], 'no such route');
    });
  });

  group('path_params', () {
    test('a plain parameter coerces to the type asked for', () async {
      final app = await example(path_params.buildApp());

      expect(app.object(await app.get('/orders/41')), {'id': 41});
    });

    test('a value that will not coerce is a 400, not a 500', () async {
      final app = await example(path_params.buildApp());

      expect((await app.get('/orders/abc')).statusCode, 400);
    });

    test('coercion is base 10, so 0x10 is not 16', () async {
      final app = await example(path_params.buildApp());

      expect((await app.get('/orders/0x10')).statusCode, 400);
    });

    test('a constrained parameter refuses at the route, with 404', () async {
      // The difference from the case above: nothing matched, so no handler and
      // no extractor ever ran.
      final app = await example(path_params.buildApp());

      expect((await app.get('/strict/41')).statusCode, 200);
      expect((await app.get('/strict/abc')).statusCode, 404);
    });

    test('a catch-all keeps the slashes', () async {
      final app = await example(path_params.buildApp());

      expect(
        app.object(await app.get('/files/css/app.css')),
        {'path': 'css/app.css'},
      );
    });

    test('several parameters are read by name', () async {
      final app = await example(path_params.buildApp());

      expect(
        app.object(await app.get('/teams/dust/members/ada')),
        {'team': 'dust', 'member': 'ada'},
      );
    });
  });

  group('query_params', () {
    test('a required value is read and coerced', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/search?q=shirt&page=2')),
        {'q': 'shirt', 'page': 2},
      );
    });

    test('a nullable type makes it optional', () async {
      final app = await example(query_params.buildApp());

      expect(app.object(await app.get('/search?q=shirt'))['page'], 1);
    });

    test('a missing required value is a 400', () async {
      final app = await example(query_params.buildApp());

      expect((await app.get('/search')).statusCode, 400);
    });

    test('queryList takes every value of a repeated key', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/filter?tag=red&tag=blue'))['tags'],
        ['red', 'blue'],
      );
    });

    test('rawQuery hands back the undecoded string', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/raw?a=1&b=%20two'))['query'],
        'a=1&b=%20two',
      );
    });
  });

  group('headers_and_host', () {
    test('a header is read, and an absent one is null', () async {
      final app = await example(headers_and_host.buildApp());

      expect(
        app.object(await app.get('/echo', headers: {'x-trace': 'abc'})),
        {'trace': 'abc'},
      );
      expect(app.object(await app.get('/echo')), {'trace': null});
    });

    test('every header can be read at once', () async {
      final app = await example(headers_and_host.buildApp());

      final all = app.object(await app.get('/all', headers: {'a': '1'}));

      expect((all['headers']! as Map)['a'], '1');
    });

    test('a host on the list is accepted', () async {
      final app = await example(headers_and_host.buildApp());

      expect((await app.get('/host')).statusCode, 200);
    });

    test('a forged host is refused, because Host is client input', () async {
      final app = await example(headers_and_host.buildApp());

      final response = await app.get(
        '/host',
        headers: {'host': 'evil.example'},
      );

      expect(response.statusCode, 400);
      expect(app.object(response)['error'], 'unrecognised host');
    });
  });

  group('json_body', () {
    test('decodes an object into the model and answers 201', () async {
      final app = await example(json_body.buildApp());

      final response = await app.post(
        '/notes',
        const {'title': 'buy milk', 'body': 'two litres'},
      );

      expect(response.statusCode, 201);
      expect(app.object(response), {'title': 'buy milk', 'body': 'two litres'});
    });

    test('a wrong content-type is 415, before any decoding', () async {
      final app = await example(json_body.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: 'title=x',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );

      expect(response.statusCode, 415);
    });

    test('bytes that are not JSON are a 400', () async {
      final app = await example(json_body.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: 'not json',
        headers: const {'content-type': 'application/json'},
      );

      expect(response.statusCode, 400);
    });

    test('a missing required field is a 422, not a 500', () async {
      final app = await example(json_body.buildApp());

      expect((await app.post('/notes', const {'body': 'x'})).statusCode, 422);
    });

    test('an array where an object belongs is a 422', () async {
      final app = await example(json_body.buildApp());

      expect((await app.post('/notes', const [1, 2])).statusCode, 422);
    });
  });

  group('form_body', () {
    test('decodes an urlencoded post', () async {
      final app = await example(form_body.buildApp());

      final response = await app.send(
        'POST',
        '/subscribe',
        body: 'email=ada@example.com&quantity=2',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );

      expect(app.object(response), {'email': 'ada@example.com', 'quantity': 2});
    });

    test('an absent optional field takes its default', () async {
      final app = await example(form_body.buildApp());

      final response = await app.send(
        'POST',
        '/subscribe',
        body: 'email=ada@example.com',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );

      expect(app.object(response)['quantity'], 1);
    });

    test('every bad field is reported at once, not just the first', () async {
      // The reason `field` returns a Result: a form is re-rendered with all of
      // the mistakes marked, so collecting beats short-circuiting.
      final app = await example(form_body.buildApp());

      final response = await app.send(
        'POST',
        '/subscribe',
        body: 'quantity=x',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );

      expect(response.statusCode, 422);
      expect(
        (app.object(response)['fields']! as Map).keys,
        containsAll(['email', 'quantity']),
      );
    });

    test('JSON to a form endpoint is a 415', () async {
      final app = await example(form_body.buildApp());

      expect((await app.post('/subscribe', const {})).statusCode, 415);
    });
  });

  group('multipart_form', () {
    /// Builds a multipart body by hand, so the test does not depend on a client.
    ({String body, String type}) multipart({bool withFile = true}) {
      const boundary = 'X-DUST-BOUNDARY';
      final caption = '--$boundary\r\n'
          'content-disposition: form-data; name="caption"\r\n\r\n'
          'my cat\r\n';
      final photo = '--$boundary\r\n'
          'content-disposition: form-data; name="photo"; '
          'filename="cat.txt"\r\n'
          'content-type: text/plain\r\n\r\n'
          'meow\r\n';
      final parts = <String>[
        caption,
        if (withFile) photo,
        '--$boundary--\r\n',
      ];
      return (
        body: parts.join(),
        type: 'multipart/form-data; boundary=$boundary',
      );
    }

    test('reads the file and the ordinary field beside it', () async {
      final app = await example(multipart_form.buildApp());
      final sent = multipart();

      final response = await app.send(
        'POST',
        '/upload',
        body: sent.body,
        headers: {'content-type': sent.type},
      );

      expect(response.statusCode, 200);
      expect(app.object(response), {
        'filename': 'cat.txt',
        'bytes': 4,
        'caption': 'my cat',
      });
    });

    test('a missing file part is a rejection, not a crash', () async {
      final app = await example(multipart_form.buildApp());
      final sent = multipart(withFile: false);

      final response = await app.send(
        'POST',
        '/upload',
        body: sent.body,
        headers: {'content-type': sent.type},
      );

      expect(response.statusCode, greaterThanOrEqualTo(400));
      expect(response.statusCode, lessThan(500));
    });

    test('a non-multipart body is a 415', () async {
      final app = await example(multipart_form.buildApp());

      expect((await app.post('/upload', const {})).statusCode, 415);
    });
  });

  group('validation_422', () {
    test('a valid payload passes through', () async {
      final app = await example(validation_422.buildApp());

      final response = await app.post(
        '/products',
        const {'title': 'Tee', 'priceCents': 2500},
      );

      expect(response.statusCode, 201);
      expect(app.object(response), {'title': 'Tee', 'priceCents': 2500});
    });

    test('every broken rule is reported, not just the first', () async {
      final app = await example(validation_422.buildApp());

      final response = await app.post(
        '/products',
        const {'title': '', 'priceCents': 0},
      );

      expect(response.statusCode, 422);
      expect(app.object(response)['fields'], {
        'title': ['is required'],
        'priceCents': ['must be more than zero'],
      });
    });

    test('a shape failure and a rule failure share one status', () async {
      // One error format for both is the point: a client writes one renderer.
      final app = await example(validation_422.buildApp());

      final shape = await app.post('/products', const {'priceCents': 2500});
      final rule = await app.post(
        '/products',
        const {'title': '', 'priceCents': 1},
      );

      expect(shape.statusCode, 422);
      expect(rule.statusCode, 422);
    });

    test('whitespace is not a title', () async {
      final app = await example(validation_422.buildApp());

      final response = await app.post(
        '/products',
        const {'title': '   ', 'priceCents': 1},
      );

      expect(response.statusCode, 422);
    });
  });

  group('customize_rejection', () {
    test('a success is left alone', () async {
      final app = await example(customize_rejection.buildApp());

      final response = await app.get('/orders/41');

      expect(response.statusCode, 200);
      expect(response.headers['content-type'], 'application/json');
      expect(app.object(response), {'id': 41});
    });

    test('an extractor failure is reshaped as problem+json', () async {
      final app = await example(customize_rejection.buildApp());

      final response = await app.get('/orders/abc');

      expect(response.statusCode, 400);
      expect(response.headers['content-type'], 'application/problem+json');
      expect(app.object(response), {
        'type': 'about:blank',
        'title': 'path parameter "id" is not a valid integer',
        'status': 400,
        'instance': '/orders/abc',
      });
    });

    test('the router own 404 is reshaped too, not only handler errors',
        () async {
      // The reason to do this in a layer: a rejection raised before any handler
      // ran still comes out in the published shape.
      final app = await example(customize_rejection.buildApp());

      final response = await app.get('/nothing');

      expect(response.statusCode, 404);
      expect(response.headers['content-type'], 'application/problem+json');
      expect(app.object(response)['status'], 404);
    });
  });

  group('parse_body_by_content_type', () {
    test('JSON takes the JSON branch', () async {
      final app = await example(by_content_type.buildApp());

      final response = await app.post('/notes', const {'title': 'from json'});

      expect(response.statusCode, 201);
      expect(app.object(response), {'from': 'json', 'title': 'from json'});
    });

    test('a form takes the form branch', () async {
      final app = await example(by_content_type.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: 'title=from a form',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );

      expect(app.object(response), {'from': 'form', 'title': 'from a form'});
    });

    test('a charset parameter does not break the match', () async {
      // Comparing the whole header would fail here, and `; charset=utf-8` is an
      // ordinary thing for a client to send.
      final app = await example(by_content_type.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: '{"title":"with charset"}',
        headers: const {'content-type': 'application/json; charset=utf-8'},
      );

      expect(app.object(response)['from'], 'json');
    });

    test('anything else is a 415 naming what is accepted', () async {
      final app = await example(by_content_type.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: '<x/>',
        headers: const {'content-type': 'text/xml'},
      );

      expect(response.statusCode, 415);
      expect(
        app.object(response)['error'],
        'send application/json or application/x-www-form-urlencoded',
      );
    });
  });

  group('state', () {
    test('a handler reads what withState attached', () async {
      final app = await example(state_example.buildApp());

      expect(app.array(await app.get('/notes')), ['first']);
    });

    test('two types coexist, neither overwriting the other', () async {
      final app = await example(state_example.buildApp());

      expect(app.array(await app.get('/notes')), isNotEmpty);
      expect(app.object(await app.get('/config')), {'currency': 'GBP'});
    });

    test('state outlives one request', () async {
      final app = await example(state_example.buildApp());

      await app.post('/notes', const {'title': 'second'});

      expect(app.array(await app.get('/notes')), ['first', 'second']);
    });

    test('a type nothing attached is a 500, not a 404', () async {
      // A wiring mistake in the route table, not something a client did.
      final app = await example(state_example.buildApp());

      expect((await app.get('/missing')).statusCode, 500);
    });
  });

  group('custom_extractor', () {
    test('reads and coerces the header', () async {
      final app = await example(custom_extractor.buildApp());

      final response = await app.get('/page', headers: {'x-page-size': '25'});

      expect(app.object(response), {'size': 25});
    });

    test('an absent header takes the default', () async {
      final app = await example(custom_extractor.buildApp());

      expect(app.object(await app.get('/page')), {'size': 10});
    });

    test('a value over the cap is refused, in one place', () async {
      // The reason this is an extractor: ?limit=1000000 is a denial-of-service
      // request dressed as pagination, and one place to refuse it beats twenty.
      final app = await example(custom_extractor.buildApp());

      final response = await app.get('/page', headers: {'x-page-size': '500'});

      expect(response.statusCode, 400);
      expect(app.object(response)['error'], 'x-page-size may not exceed 100');
    });

    test('a non-numeric value is a 400 from the coercion', () async {
      final app = await example(custom_extractor.buildApp());

      expect(
        (await app.get('/page', headers: {'x-page-size': 'big'})).statusCode,
        400,
      );
    });

    test('zero is refused, not treated as absent', () async {
      final app = await example(custom_extractor.buildApp());

      expect(
        (await app.get('/page', headers: {'x-page-size': '0'})).statusCode,
        400,
      );
    });
  });

  group('optional_extraction', () {
    test('a required value absent is a 400', () async {
      final app = await example(optional_extraction.buildApp());

      expect((await app.get('/strict')).statusCode, 400);
      expect(app.object(await app.get('/strict?page=2')), {'page': 2});
    });

    test('a nullable type makes absent fine but malformed still an error',
        () async {
      final app = await example(optional_extraction.buildApp());

      expect(app.object(await app.get('/nullable')), {'page': 1});
      expect((await app.get('/nullable?page=x')).statusCode, 400);
    });

    test('optional swallows a malformed value as well as an absent one',
        () async {
      // The trap: a client with a bug gets silence and page one. A nullable
      // type is the right default; optional is for composing.
      final app = await example(optional_extraction.buildApp());

      expect(app.object(await app.get('/optional')), {'page': 'none'});
      expect(app.object(await app.get('/optional?page=x')), {'page': 'none'});
      expect(app.object(await app.get('/optional?page=3')), {'page': 3});
    });
  });

  group('fallible_extraction', () {
    test('both failures are reported together', () async {
      final app = await example(fallible_extraction.buildApp());

      final response = await app.get('/report');

      expect(response.statusCode, 422);
      expect(
        (app.object(response)['fields']! as Map).keys,
        containsAll(['from', 'to']),
      );
    });

    test('one failure names only that field', () async {
      final app = await example(fallible_extraction.buildApp());

      final response = await app.get('/report?from=x&to=9');

      expect(response.statusCode, 422);
      expect((app.object(response)['fields']! as Map).keys, ['from']);
    });

    test('both valid passes through', () async {
      final app = await example(fallible_extraction.buildApp());

      expect(
        app.object(await app.get('/report?from=1&to=9')),
        {'from': 1, 'to': 9},
      );
    });

    test('a bad value can answer with a redirect instead of a 400', () async {
      final app = await example(fallible_extraction.buildApp());

      final response = await app.raw('GET', '/browse?page=x');

      expect(response.statusCode, 303);
      expect(response.headers['location'], '/browse');
    });
  });

  group('cookies', () {
    test('sign-in sets a cookie with every attribute that matters', () async {
      final app = await example(cookies.buildApp());

      final header = (await app.get('/sign-in')).headers['set-cookie']!;

      expect(header, contains('user=ada'));
      expect(header, contains('HttpOnly'));
      expect(header, contains('Secure'));
      expect(header, contains('SameSite=Lax'));
      expect(header, contains('Path=/'));
    });

    test('sign-out expires the same cookie, since HTTP has no delete',
        () async {
      final app = await example(cookies.buildApp());

      final header = (await app.get('/sign-out')).headers['set-cookie']!;

      expect(header, contains('Max-Age=0'));
    });

    test('one cookie is read, and absent is null rather than an error',
        () async {
      final app = await example(cookies.buildApp());

      expect(
        app.object(await app.get('/whoami', headers: {'cookie': 'user=ada'})),
        {'user': 'ada'},
      );
      expect(app.object(await app.get('/whoami')), {'user': null});
    });

    test('the whole jar is read at once', () async {
      final app = await example(cookies.buildApp());

      final response = await app.get('/all', headers: {'cookie': 'a=1; b=2'});

      expect(app.object(response)['cookies'], {'a': '1', 'b': '2'});
    });
  });

  group('bearer_auth', () {
    test('a known token names its user', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Bearer t-ada'},
      );

      expect(app.object(response), {'user': 'ada'});
    });

    test('no credential is 401 with the challenge', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get('/me');

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], contains('Bearer'));
    });

    test('the wrong scheme is 401, not 403', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Basic YWRhOnNlY3JldA=='},
      );

      expect(response.statusCode, 401);
    });

    test('a real credential that is not allowed is 403', () async {
      // A client that retries on 401 loops forever if a wrong token answers
      // 401. The distinction is what tells it to stop.
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Bearer nope'},
      );

      expect(response.statusCode, 403);
    });

    test('a route that asks for no credential requires none', () async {
      final app = await example(bearer_auth.buildApp());

      expect((await app.get('/public')).statusCode, 200);
    });
  });

  group('credential_schemes', () {
    test('a bearer token is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
          await app.get('/whoami', headers: {'authorization': 'Bearer t-ada'}),
        ),
        {'via': 'bearer', 'id': 'ada'},
      );
    });

    test('an API key header is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(await app.get('/whoami', headers: {'x-api-key': 'k-robot'})),
        {'via': 'api-key', 'id': 'robot'},
      );
    });

    test('HTTP Basic is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
          await app.get(
            '/whoami',
            headers: {'authorization': 'Basic YWRhOnNlY3JldA=='},
          ),
        ),
        {'via': 'basic', 'id': 'ada'},
      );
    });

    test('a session cookie is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
            await app.get('/whoami', headers: {'cookie': 'session=s-ada'})),
        {'via': 'session', 'id': 'ada'},
      );
    });

    test('a key in the query string is refused, because URLs leak', () async {
      // Query strings reach access logs, proxy logs, browser history, and the
      // Referer header. allowQuery: false is the setting that closes it.
      final app = await example(credential_schemes.buildApp());

      expect((await app.get('/whoami?api_key=k-robot')).statusCode, 401);
    });

    test('no credential at all is 401 carrying the last scheme challenge',
        () async {
      // Pinning a wart rather than pretending it away: firstOf returns the
      // final Err, so the challenge and the message come from whichever scheme
      // ran last — accurate for that one, misleading about the other three.
      final app = await example(credential_schemes.buildApp());

      final response = await app.get('/whoami');

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], 'Cookie');
      expect(app.object(response)['error'], 'expected a session cookie');
    });

    test('a wrong Basic password is refused', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        (await app.get(
          '/whoami',
          headers: {'authorization': 'Basic YWRhOnd)cm9uZw=='},
        ))
            .statusCode,
        401,
      );
    });
  });

  group('cors', () {
    test('an allowed origin gets the allow header and Vary', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://app.example'},
      );

      expect(
        response.headers['access-control-allow-origin'],
        'https://app.example',
      );
      // Without Vary a cache can serve one origin's response to another.
      expect(response.headers['vary'], contains('Origin'));
    });

    test('an origin not on the list gets no allow header', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://evil.example'},
      );

      // The body still comes back: CORS instructs the browser, it is not a
      // server-side gate. curl sees the data either way.
      expect(response.statusCode, 200);
      expect(response.headers['access-control-allow-origin'], isNull);
    });

    test('a preflight is answered without reaching the handler', () async {
      final app = await example(cors.buildApp());

      final response = await app.raw(
        'OPTIONS',
        '/api/notes',
        headers: {
          'origin': 'https://app.example',
          'access-control-request-method': 'POST',
        },
      );

      expect(response.statusCode, anyOf(200, 204));
      expect(
        response.headers['access-control-allow-methods'],
        contains('POST'),
      );
      expect(response.headers['access-control-max-age'], '600');
    });

    test('exposeHeaders is what lets fetch read x-request-id', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://app.example'},
      );

      expect(
        response.headers['access-control-expose-headers'],
        contains('x-request-id'),
      );
    });

    test('credentials with a wildcard origin throws at construction', () {
      // A browser refuses a credentialed response allowed for "*", so this is
      // caught here rather than in someone console.
      expect(
        () => Cors(origins: const AllowedOrigins.any(), credentials: true),
        throwsArgumentError,
      );
    });
  });

  group('compression', () {
    test('a big enough body is gzipped, and Vary is set', () async {
      final app = await example(compression.buildApp());

      final client = HttpClient()..autoUncompress = false;
      addTearDown(client.close);
      final request = await client.getUrl(app.uri('/rows'));
      request.headers.set('accept-encoding', 'gzip');
      final response = await request.close();
      final bytes = await response.fold<List<int>>(
        <int>[],
        (all, chunk) => all..addAll(chunk),
      );

      expect(response.headers.value('content-encoding'), 'gzip');
      expect(response.headers.value('vary'), contains('Accept-Encoding'));
      expect(gzip.decode(bytes).length, greaterThan(bytes.length));
    });

    test('gzip;q=0 is a refusal, not an absence', () async {
      final app = await example(compression.buildApp());

      final response = await app.get(
        '/rows',
        headers: {'accept-encoding': 'gzip;q=0'},
      );

      expect(response.headers['content-encoding'], isNull);
    });

    test('a body under the threshold is left alone', () async {
      final app = await example(compression.buildApp());

      final response = await app.get(
        '/ping',
        headers: {'accept-encoding': 'gzip'},
      );

      expect(response.body.length, lessThan(1024));
      expect(response.headers['content-encoding'], isNull);
    });
  });

  group('request_id', () {
    test('every answer carries an id', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      expect((await app.get('/notes')).headers['x-request-id'], isNotEmpty);
    });

    test('a client-supplied id is kept, so one id spans every hop', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final response = await app.get(
        '/notes',
        headers: {'x-request-id': 'from-the-gateway'},
      );

      expect(response.headers['x-request-id'], 'from-the-gateway');
    });

    test('the handler reads the same id the response carries', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final response = await app.get(
        '/echo-id',
        headers: {'x-request-id': 'abc-123'},
      );

      expect(app.object(response), {'requestId': 'abc-123'});
      expect(response.headers['x-request-id'], 'abc-123');
    });

    test('two requests get different ids', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final first = (await app.get('/notes')).headers['x-request-id'];
      final second = (await app.get('/notes')).headers['x-request-id'];

      expect(first, isNot(second));
    });
  });

  group('access_log', () {
    test('records the method, path, and status', () async {
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes');

      expect(records.single.method, 'GET');
      expect(records.single.path, '/notes');
      expect(records.single.status, 200);
    });

    test('records a 404 too, because it is a request', () async {
      // Above the routes on purpose: a request that never reaches the log is
      // one nobody can explain.
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/nothing');

      expect(records.single.status, 404);
      expect(records.single.path, '/nothing');
    });

    test('the recorded path carries no query string', () async {
      // Query strings carry API keys and reset tokens. An access log is a
      // recognised place they leak.
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes?api_key=secret');

      expect(records.single.path, '/notes');
      expect(records.single.path, isNot(contains('secret')));
    });

    test('the record carries the request id, so the two logs join up',
        () async {
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes', headers: {'x-request-id': 'abc-123'});

      expect(records.single.requestId, 'abc-123');
    });
  });

  group('normalize_path', () {
    test('a trailing slash is rewritten, and the client sees one response',
        () async {
      final app = await example(normalize_path.buildApp());

      final bare = await app.get('/notes');
      final slashed = await app.get('/notes/');

      expect(slashed.statusCode, 200);
      expect(slashed.body, bare.body);
    });

    test('a nested route is covered when the layer sits above the nest',
        () async {
      final app = await example(normalize_path.buildApp());

      expect((await app.get('/api/notes/')).statusCode, 200);
    });

    test('the same layer inside a nested router silently does nothing',
        () async {
      // The trap this example keeps on purpose. A nested router's layer runs
      // only after one of its routes matched, and normalizing exists to make a
      // path match — so it never runs for the request it was added to fix.
      final right = await example(normalize_path.buildApp());
      final wrong = await example(normalize_path.buildMisplacedApp());

      // Same layer, same route, same request. Only the placement differs.
      expect((await right.get('/api/notes/')).statusCode, 200);
      expect((await wrong.get('/api/notes/')).statusCode, 404);

      // And the route itself is fine — it is only the trailing slash that is
      // left unhandled, which is what makes the mistake hard to spot.
      expect((await wrong.get('/api/notes')).statusCode, 200);
    });

    test('the root is never touched', () async {
      // Stripping its slash would leave an empty path nothing can match.
      final app = await example(normalize_path.buildApp());

      expect(app.object(await app.get('/')), {'root': true});
    });
  });

  group('security_headers', () {
    test('a page carries the whole set, CSP included', () async {
      final app = await example(security_headers.buildApp());

      final response = await app.get('/page');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['x-frame-options'], 'DENY');
      expect(response.headers['referrer-policy'], isNotNull);
      expect(response.headers['content-security-policy'], contains("'self'"));
    });

    test('the CSP names its sources and allows no inline script', () async {
      // An allowlist that permits inline scripts permits the injected one too.
      final app = await example(security_headers.buildApp());

      final policy =
          (await app.get('/page')).headers['content-security-policy']!;

      expect(policy, isNot(contains('unsafe-inline')));
      expect(policy, contains("frame-ancestors 'none'"));
    });

    test('the API gets the cheap headers and no CSP', () async {
      final app = await example(security_headers.buildApp());

      final response = await app.get('/api/notes');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['content-security-policy'], isNull);
    });

    test('HSTS is absent, because this example serves plain HTTP', () async {
      final app = await example(security_headers.buildApp());

      expect(
        (await app.get('/page')).headers['strict-transport-security'],
        isNull,
      );
    });
  });

  group('route_layer', () {
    test('a matched route without a credential is 401', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get('/admin/orders');

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], contains('Bearer'));
    });

    test('the credential gets through', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get(
        '/admin/orders',
        headers: {'authorization': 'Bearer staff'},
      );

      expect(app.array(response), ['order-1']);
    });

    test('an unmatched path under the prefix is 404, not 401', () async {
      // The whole reason for routeLayer. With a plain layer this answers 401,
      // and a typo in your own route table looks like an auth problem.
      final app = await example(route_layer.buildApp());

      expect((await app.get('/admin/typo')).statusCode, 404);
    });

    test('routes outside the guard are untouched', () async {
      final app = await example(route_layer.buildApp());

      expect((await app.get('/health')).statusCode, 200);
    });

    test('a real credential that is not the staff one is 403', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get(
        '/admin/orders',
        headers: {'authorization': 'Bearer intern'},
      );

      expect(response.statusCode, 403);
    });
  });

  group('request_timeout', () {
    test('a request inside the budget is untouched', () async {
      final app = await example(request_timeout.buildApp(onTimeout: (_) {}));

      expect(app.object(await app.get('/quick')), {'ok': true});
    });

    test('a request over the budget is a 503', () async {
      final app = await example(request_timeout.buildApp(onTimeout: (_) {}));

      final response = await app.get('/slow');

      expect(response.statusCode, 503);
      expect(app.object(response)['error'], contains('200ms'));
    });

    test('onTimeout fires, so a 503 can be counted', () async {
      // A 503 nobody counted is an outage nobody noticed.
      final timedOut = <String>[];
      final app = await example(
        request_timeout.buildApp(
          onTimeout: (request) => timedOut.add(request.url.path),
        ),
      );

      await app.get('/slow');

      expect(timedOut, ['slow']);
    });
  });
}
