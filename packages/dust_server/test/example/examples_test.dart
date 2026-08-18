import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

import '../../example/access_log.dart' as access_log;
import '../../example/bearer_auth.dart' as bearer_auth;
import '../../example/compression.dart' as compression;
import '../../example/background_tasks.dart' as background_tasks;
import '../../example/body_limits.dart' as body_limits;
import '../../example/client_ip.dart' as client_ip;
import '../../example/clustered_isolates.dart' as clustered;
import '../../example/cors.dart' as cors;
import '../../example/cookies.dart' as cookies;
import '../../example/credential_schemes.dart' as credential_schemes;
import '../../example/custom_extractor.dart' as custom_extractor;
import '../../example/customize_rejection.dart' as customize_rejection;
import '../../example/error_handling.dart' as error_handling;
import '../../example/global_404.dart' as global_404;
import '../../example/graceful_shutdown.dart' as graceful_shutdown;
import '../../example/handle_head_request.dart' as head_request;
import '../../example/fallible_extraction.dart' as fallible_extraction;
import '../../example/form_body.dart' as form_body;
import '../../example/headers_and_host.dart' as headers_and_host;
import '../../example/hello_world.dart' as hello_world;
import '../../example/json_body.dart' as json_body;
import '../../example/health_checks.dart' as health_checks;
import '../../example/metrics.dart' as metrics;
import '../../example/multipart_form.dart' as multipart_form;
import '../../example/multipart_stream.dart' as multipart_stream;
import '../../example/normalize_path.dart' as normalize_path;
import '../../example/optional_extraction.dart' as optional_extraction;
import '../../example/parse_body_by_content_type.dart' as by_content_type;
import '../../example/path_params.dart' as path_params;
import '../../example/query_params.dart' as query_params;
import '../../example/print_request_response.dart' as print_both;
import '../../example/redirects.dart' as redirects;
import '../../example/request_id.dart' as request_id;
import '../../example/request_timeout.dart' as request_timeout;
import '../../example/route_layer.dart' as route_layer;
import '../../example/routing.dart' as routing;
import '../../example/security_headers.dart' as security_headers;
import '../../example/sessions.dart' as sessions;
import '../../example/sse.dart' as sse;
import '../../example/static_files.dart' as static_files;
import '../../example/templates.dart' as templates;
import '../../example/testing.dart' as testing;
import '../../example/tls.dart' as tls;
import '../../example/tracing.dart' as tracing;
import '../../example/versioning.dart' as versioning;
import '../../example/webhook_signatures.dart' as webhooks;
import '../../example/websockets.dart' as websockets;
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
    test('a returned String goes out as text', () async {
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

  group('multipart_stream', () {
    /// A multipart body with one field and one file of [bytes] bytes.
    ({String type, List<int> body}) upload({int bytes = 200000}) {
      const boundary = 'X-DUST-STREAM';
      final out = <int>[
        ...utf8.encode('--$boundary\r\n'
            'content-disposition: form-data; name="caption"\r\n\r\n'
            'a big one\r\n'),
        ...utf8.encode('--$boundary\r\n'
            'content-disposition: form-data; name="file"; '
            'filename="../../etc/passwd"\r\n'
            'content-type: application/octet-stream\r\n\r\n'),
        ...List.filled(bytes, 7),
        ...utf8.encode('\r\n--$boundary--\r\n'),
      ];

      return (type: 'multipart/form-data; boundary=$boundary', body: out);
    }

    Future<String> directory() async {
      final root = await Directory.systemTemp.createTemp('dust-example-up-');
      addTearDown(() => root.delete(recursive: true));
      return root.path;
    }

    test('writes the file to disk and reports what it stored', () async {
      final root = await directory();
      final app = await example(multipart_stream.buildApp(root));
      final sent = upload();

      final response = await app.send(
        'POST',
        '/upload',
        body: String.fromCharCodes(sent.body),
        headers: {'content-type': sent.type},
      );

      expect(response.statusCode, 201);
      final decoded = app.object(response);
      expect((decoded['fields']! as Map)['caption'], 'a big one');
      expect(decoded['files'], hasLength(1));
      expect((decoded['files']! as List).first, isA<Map<String, Object?>>());
    });

    test('stores under a generated id, never the client filename', () async {
      // `../../etc/passwd` is a valid filename as far as the client is
      // concerned. It is kept as data and never used as a path.
      final root = await directory();
      final app = await example(multipart_stream.buildApp(root));
      final sent = upload();

      final response = await app.send(
        'POST',
        '/upload',
        body: String.fromCharCodes(sent.body),
        headers: {'content-type': sent.type},
      );

      final file = (app.object(response)['files']! as List).first! as Map;
      expect(file['filename'], '../../etc/passwd');
      expect(file['id'], matches(r'^[a-z0-9]{16}$'));

      final written = Directory(root).listSync().whereType<File>().toList();
      expect(written, hasLength(1));
      expect(written.single.path, endsWith(file['id'] as String));
    });

    test('two ids never collide', () async {
      final root = await directory();
      final app = await example(multipart_stream.buildApp(root));

      final ids = <Object?>{};
      for (var index = 0; index < 4; index++) {
        final sent = upload(bytes: 32);
        final response = await app.send(
          'POST',
          '/upload',
          body: String.fromCharCodes(sent.body),
          headers: {'content-type': sent.type},
        );
        ids.add(((app.object(response)['files']! as List).first! as Map)['id']);
      }

      expect(ids, hasLength(4));
    });

    test('a body over the limit is refused', () async {
      final root = await directory();
      final app = await example(multipart_stream.buildApp(root));
      final sent = upload(bytes: 11 * 1024 * 1024);

      final response = await app.send(
        'POST',
        '/upload',
        body: String.fromCharCodes(sent.body),
        headers: {'content-type': sent.type},
      );

      expect(response.statusCode, 413);
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

    test('no credential at all is 401 offering every scheme', () async {
      // It used to name only whichever scheme ran last — accurate for that one
      // and misleading about the other three.
      final app = await example(credential_schemes.buildApp());

      final response = await app.get('/whoami');

      expect(response.statusCode, 401);
      final challenge = response.headers['www-authenticate']!;
      expect(challenge, contains('Bearer'));
      expect(challenge, contains('Cookie'));
      expect(app.object(response)['error'], 'no credentials were supplied');
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

    test('a layer on a nested router covers that prefix, 404s included',
        () async {
      final app = await example(normalize_path.buildScopedApp());

      expect((await app.get('/api/notes')).statusCode, 200);
      expect((await app.get('/api/notes/')).statusCode, 200);
    });

    test('and covers nothing outside it', () async {
      // Scoping matters when one half of an application has URLs you must not
      // rewrite — a webhook whose signature covers the exact path, say.
      final app = await example(normalize_path.buildScopedApp());

      expect((await app.get('/shop/notes')).statusCode, 200);
      expect((await app.get('/shop/notes/')).statusCode, 404);
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

      final response = await app.get('/app');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['x-frame-options'], 'DENY');
      expect(response.headers['referrer-policy'], isNotNull);
      expect(response.headers['content-security-policy'], contains("'self'"));
    });

    test('the CSP names its sources and allows no inline script', () async {
      // An allowlist that permits inline scripts permits the injected one too.
      final app = await example(security_headers.buildApp());

      final policy =
          (await app.get('/app')).headers['content-security-policy']!;

      expect(policy, isNot(contains('unsafe-inline')));
      expect(policy, contains("frame-ancestors 'none'"));
    });

    test('the API gets the cheap headers and no CSP', () async {
      // Two nested routers, so each policy covers one half. A merged router has
      // no prefix, and its layer would cover both.
      final app = await example(security_headers.buildApp());

      final response = await app.get('/api/notes');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['content-security-policy'], isNull);
    });

    test('HSTS is absent, because this example serves plain HTTP', () async {
      final app = await example(security_headers.buildApp());

      expect(
        (await app.get('/app')).headers['strict-transport-security'],
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

  group('redirects', () {
    test('a POST answers 303, so a reload does not re-submit', () async {
      final app = await example(redirects.buildApp());

      final response = await app.raw('POST', '/notes');

      expect(response.statusCode, 303);
      expect(response.headers['location'], '/notes/1');
    });

    test('permanent is 308, which keeps the method', () async {
      final app = await example(redirects.buildApp());

      final response = await app.raw('GET', '/old-home');

      expect(response.statusCode, 308);
      expect(response.headers['location'], '/');
    });

    test('temporary is 307, so nothing caches it', () async {
      final app = await example(redirects.buildApp());

      expect((await app.raw('GET', '/maintenance')).statusCode, 307);
    });

    test('a newline in the target is stripped from Location', () async {
      // Location is one of the few places caller input reaches a header, and a
      // newline there would let a client inject headers of its own.
      final app = await example(redirects.buildApp());

      final response = await app.raw(
        'GET',
        '/search?q=a%0d%0aX-Evil%3A%201',
      );

      expect(response.statusCode, 303);
      expect(response.headers['location'], isNot(contains('\n')));
      expect(response.headers['location'], isNot(contains('\r')));
      expect(response.headers['x-evil'], isNull);
    });
  });

  group('sse', () {
    test('sets the three headers a stream needs', () async {
      final app = await example(sse.buildApp());

      final response = await app.get('/ticks');

      expect(response.headers['content-type'], startsWith('text/event-stream'));
      expect(response.headers['cache-control'], 'no-cache');
      // Without this nginx buffers, and the stream looks like a hung server.
      expect(response.headers['x-accel-buffering'], 'no');
    });

    test('each event carries its data and id', () async {
      final app = await example(sse.buildApp());

      final body = (await app.get('/ticks')).body;

      // No space after the colon. The specification makes one optional and
      // tells clients to strip it, so both forms are legal on the wire.
      expect(body, contains('id:1'));
      expect(body, contains('data:{"tick":1}'));
      expect(body, contains('id:5'));
    });

    test('Last-Event-ID resumes rather than replaying', () async {
      // A server that ignores it drops whatever happened while the client was
      // away, which is the reconnect the browser makes automatically.
      final app = await example(sse.buildApp());

      final body =
          (await app.get('/ticks', headers: {'last-event-id': '2'})).body;

      expect(body, contains('id:3'));
      expect(body, isNot(contains('id:1')));
    });

    test('a named event is distinguishable from a default one', () async {
      final app = await example(sse.buildApp());

      final body = (await app.get('/progress')).body;

      expect(body, contains('event:step'));
      expect(body, contains('data:fetching'));
      expect(body, contains('event:done'));
    });
  });

  group('websockets', () {
    /// The headers a real handshake sends.
    Map<String, String> handshake() => const {
          'connection': 'Upgrade',
          'upgrade': 'websocket',
          'sec-websocket-version': '13',
          'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==',
        };

    test('an upgrade without a ticket is refused while it is still HTTP',
        () async {
      // After the upgrade there is no status code to send, which is why the
      // check belongs here.
      final app = await example(websockets.buildApp());

      final response = await app.raw('GET', '/echo', headers: handshake());

      expect(response.statusCode, 401);
    });

    test(
        'a foreign origin is refused, because same-origin does not apply to '
        'WebSockets', () async {
      // Any page on the internet may open one carrying the user's cookies.
      // Checking Origin is the whole defence.
      final app = await example(websockets.buildApp());

      final response = await app.raw(
        'GET',
        '/echo?token=t-ada',
        headers: {...handshake(), 'origin': 'https://evil.example'},
      );

      expect(response.statusCode, 403);
    });

    test('an allowed origin with a ticket upgrades', () async {
      final app = await example(websockets.buildApp());

      final response = await app.raw(
        'GET',
        '/echo?token=t-ada',
        headers: {...handshake(), 'origin': 'http://localhost:3000'},
      );

      expect(response.statusCode, 101);
    });

    test('a plain GET to an upgrade route is not an upgrade', () async {
      final app = await example(websockets.buildApp());

      expect((await app.get('/echo?token=t-ada')).statusCode, isNot(101));
    });

    test('echo round-trips a message', () async {
      final app = await example(websockets.buildApp());

      final socket = await WebSocket.connect(
        'ws://${app.uri('/echo?token=t-ada').authority}'
        '/echo?token=t-ada',
        headers: {'origin': 'http://localhost:3000'},
      );
      addTearDown(socket.close);

      socket.add('ping');

      expect(await socket.first, 'echo: ping');
    });

    test('the negotiated subprotocol reaches the handler', () async {
      // It used to be dropped, so session.protocol was always null.
      final app = await example(websockets.buildApp());

      final socket = await WebSocket.connect(
        'ws://${app.uri('/greeter?token=t-ada').authority}'
        '/greeter?token=t-ada',
        protocols: const ['greeting.v2'],
        headers: {'origin': 'http://localhost:3000'},
      );
      addTearDown(socket.close);

      expect(socket.protocol, 'greeting.v2');
      expect(await socket.first, '{"hello":true}');
    });
  });

  group('templates', () {
    test('renders a page inside the shared layout', () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/');

      expect(response.headers['content-type'], 'text/html; charset=utf-8');
      expect(response.body, startsWith('<!doctype html>'));
      expect('<head>'.allMatches(response.body).length, 1);
      expect(response.body, contains('href="/notes/1"'));
    });

    test('interpolation is escaped, which is the reason to use an engine',
        () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/notes/2');

      expect(response.body, contains('&lt;script&gt;'));
      expect(response.body, isNot(contains('<script>alert(1)</script>')));
    });

    test('an unknown id answers 404 as a page, not as JSON', () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/notes/9');

      expect(response.statusCode, 404);
      expect(response.headers['content-type'], 'text/html; charset=utf-8');
      expect(response.body, contains('Back'));
    });
  });

  group('static_files', () {
    /// Writes a throwaway build for one test.
    Future<String> build() async {
      final root =
          await Directory.systemTemp.createTemp('dust-example-static-');
      addTearDown(() => root.delete(recursive: true));
      await File('${root.path}/index.html').writeAsString(
        '<!doctype html><title>App</title><div id="app">loading</div>',
      );
      await File('${root.path}/main.a1b2c3.js')
          .writeAsString('console.log("fingerprinted");');
      return root.path;
    }

    test('the root serves the default document', () async {
      final app = await example(static_files.buildApp(await build()));

      final response = await app.get('/');

      expect(response.statusCode, 200);
      expect(response.body, contains('<div id="app">'));
    });

    test('a deep link serves the same document, so the client router runs',
        () async {
      // Without html: true this is a 404 — there is no such file, and the
      // router that would have handled it has not loaded yet.
      final app = await example(static_files.buildApp(await build()));

      final response = await app.get('/orders/41');

      expect(response.statusCode, 200);
      expect(response.body, contains('<div id="app">'));
    });

    test('the document is revalidated, not cached hard', () async {
      // It is how a browser learns the new asset names. Cache it for a year and
      // users stay on a deploy you have replaced.
      final app = await example(static_files.buildApp(await build()));

      final cache = (await app.get('/')).headers['cache-control'] ?? '';

      expect(cache, isNot(contains('immutable')));
    });

    test('a fingerprinted asset is immutable', () async {
      final app = await example(static_files.buildApp(await build()));

      final cache =
          (await app.get('/main.a1b2c3.js')).headers['cache-control'] ?? '';

      expect(cache, contains('immutable'));
    });

    test('the API is reachable, because it is mounted first', () async {
      // mount('/') claims everything below it, so order decides whether /api
      // reaches its routes or gets the document.
      final app = await example(static_files.buildApp(await build()));

      expect(app.array(await app.get('/api/notes')), ['first']);
    });
  });

  group('global_404', () {
    test('a browser gets a page', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get('/nothing');

      expect(response.statusCode, 404);
      expect(response.headers['content-type'], 'text/html; charset=utf-8');
    });

    test('an API path gets JSON whatever it says it accepts', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get(
        '/api/nothing',
        headers: {'accept': 'text/html'},
      );

      expect(response.statusCode, 404);
      expect(app.object(response)['error'], 'no such route');
    });

    test('an Accept header alone is enough to ask for JSON', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get(
        '/nothing',
        headers: {'accept': 'application/json'},
      );

      expect(app.object(response)['error'], 'no such route');
    });

    test('a 405 does not reach the fallback', () async {
      // A path that exists for another method is answered with Allow, which is
      // more useful to a client than a 404.
      final app = await example(global_404.buildApp());

      final response = await app.send('PUT', '/api/notes');

      expect(response.statusCode, 405);
      expect(response.headers['allow'], contains('GET'));
    });
  });

  group('handle_head_request', () {
    test('HEAD is answered from the GET route, with no body', () async {
      final app = await example(head_request.buildApp());

      final get = await app.get('/notes');
      final head = await app.send('HEAD', '/notes');

      expect(head.statusCode, 200);
      expect(head.body, isEmpty);
      expect(head.headers['content-type'], get.headers['content-type']);
    });

    test('HEAD appears in Allow', () async {
      final app = await example(head_request.buildApp());

      expect(
          (await app.send('PUT', '/notes')).headers['allow'], contains('HEAD'));
    });

    test('an explicit HEAD route overrides the automatic one', () async {
      final app = await example(head_request.buildApp());

      final head = await app.send('HEAD', '/report');

      expect(head.statusCode, 200);
      expect(head.headers['content-length'], '28');
    });

    test('the handler still runs for HEAD, body discarded', () async {
      // A GET that increments a counter does so for every HEAD too. If that is
      // unwanted, the work does not belong in a GET.
      final app = await example(head_request.buildApp());
      head_request.counter.calls = 0;

      await app.get('/counted');
      await app.send('HEAD', '/counted');

      expect(head_request.counter.calls, 2);
    });
  });

  group('versioning', () {
    test('each path version keeps its own shape', () async {
      final app = await example(versioning.buildApp());

      expect(app.array(await app.get('/v1/notes')), ['first', 'second']);
      expect(
        (app.array(await app.get('/v2/notes')).first! as Map)['id'],
        1,
      );
    });

    test('no version header means the oldest, not the newest', () async {
      // Defaulting to latest breaks a client the day you ship v3 — silently,
      // with no deploy of theirs to blame.
      final app = await example(versioning.buildApp());

      expect(app.array(await app.get('/notes')), ['first', 'second']);
    });

    test('the header selects a version', () async {
      final app = await example(versioning.buildApp());

      final response = await app.get(
        '/notes',
        headers: {'accept': 'application/vnd.notes.v2+json'},
      );

      expect((app.array(response).first! as Map)['title'], 'first');
    });

    test('an unknown version is a 406 naming what exists', () async {
      final app = await example(versioning.buildApp());

      final response = await app.get(
        '/notes',
        headers: {'accept': 'application/vnd.notes.v9+json'},
      );

      expect(response.statusCode, 406);
      expect(app.object(response)['error'], contains('try v1 or v2'));
    });
  });

  group('graceful_shutdown', () {
    test('a request already accepted finishes after close begins', () async {
      // The whole point. A process that exits on the signal drops these, and
      // they are the slow ones — the ones most likely to be mid-write.
      final app = await ExampleApp.serve(graceful_shutdown.buildApp());

      final slow = app.get('/slow');
      await Future<void>.delayed(const Duration(milliseconds: 100));
      final settled = await app.stop();

      expect(settled, isTrue);
      expect((await slow).statusCode, 200);
    });

    test('close reports whether everything finished', () async {
      // The return value is the only way to learn that requests were abandoned,
      // and it is the thing most code throws away.
      final app = await ExampleApp.serve(graceful_shutdown.buildApp());

      await app.get('/quick');

      expect(await app.stop(), isTrue);
    });

    test('nothing new is accepted once close has begun', () async {
      final app = await ExampleApp.serve(graceful_shutdown.buildApp());
      final origin = app.origin;

      await app.stop();

      await expectLater(
        HttpClient().getUrl(Uri.parse('$origin/quick')).then((r) => r.close()),
        throwsA(isA<SocketException>()),
      );
    });
  });

  group('tracing', () {
    test('the span is named after the route, not the URL', () async {
      // /orders/41 and /orders/42 must share a name, or a dashboard has one
      // series per order and nothing can group it.
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');
      await app.get('/orders/42');

      expect(
        spans.spans.map((span) => span.name),
        ['GET /orders/{id}', 'GET /orders/{id}'],
      );
    });

    test('a 404 is traced too', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/nothing');

      expect(spans.spans, hasLength(1));
      expect(spans.spans.single.attributes['http.response.status_code'], 404);
    });

    test('an incoming traceparent is continued, not replaced', () async {
      const traceId = '4bf92f3577b34da6a3ce929d0e0e4736';
      const parentId = '00f067aa0ba902b7';
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get(
        '/orders/41',
        headers: {'traceparent': '00-$traceId-$parentId-01'},
      );

      expect(spans.spans.single.context.traceId, traceId);
      expect(spans.spans.single.parentSpanId, parentId);
    });

    test('an absent traceparent starts a trace', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');

      expect(spans.spans.single.context.traceId, hasLength(32));
      expect(spans.spans.single.parentSpanId, isNull);
    });

    test('attributes set inside the handler reach the span', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');

      expect(spans.spans.single.attributes['order.id'], 41);
      expect(spans.spans.single.attributes['cache.hit'], false);
    });

    test('nameSpan overrides the route name', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/legacy/rebuild');

      expect(spans.spans.single.name, 'legacy.rebuild');
    });
  });

  group('clustered_isolates', () {
    test('the factory builds a working application on its own', () async {
      // The cluster itself is covered by the runtime's serving tests. What this
      // example owns is that its factory is a valid top-level one.
      final app = await example(clustered.buildApp());

      final response = await app.get('/whoami');

      expect(response.statusCode, 200);
      expect(app.object(response)['seen'], 1);
    });

    test('state is per-application, which is per-isolate in a cluster',
        () async {
      // Two applications from one factory share nothing. In a cluster that is
      // exactly what each isolate gets, and why an in-memory counter counts a
      // fraction of the traffic.
      final first = await example(clustered.buildApp());
      final second = await example(clustered.buildApp());

      await first.get('/whoami');
      await first.get('/whoami');

      expect(app0(await first.get('/whoami')), 3);
      expect(app0(await second.get('/whoami')), 1);
    });
  });

  group('tls', () {
    test('the application serves plainly, so TLS is a deployment choice',
        () async {
      final app = await example(tls.buildApp());

      expect(app.object(await app.get('/health')), {'status': 'ok'});
    });

    test('no HSTS without a real certificate', () async {
      // Sent from a host whose certificate later lapses, it locks users out.
      final app = await example(tls.buildApp());

      expect(
        (await app.get('/health')).headers['strict-transport-security'],
        isNull,
      );
    });

    test('the redirect application sends everything to https with 308',
        () async {
      // 308, so a POST is not silently turned into a GET and stripped of its
      // body.
      final app = await example(tls.buildRedirectApp(port: 8443));

      final response = await app.raw('GET', '/health');

      expect(response.statusCode, 308);
      expect(response.headers['location'], startsWith('https://'));
      expect(response.headers['location'], contains(':8443/health'));
    });

    test('port 443 is left out of the redirect target', () async {
      final app = await example(tls.buildRedirectApp(port: 443));

      final location = (await app.raw('GET', '/health')).headers['location']!;

      expect(location, isNot(contains(':443')));
    });
  });

  group('metrics', () {
    test('two URLs on one route share one series', () async {
      // The reason to label by matched route. Labelling by path gives one time
      // series per order, which is how a metrics backend falls over.
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/orders/41');
      await app.get('/orders/42');

      final scraped = (await app.get('/metrics')).body;

      expect(
        scraped,
        contains('http_requests_total{route="/orders/{id}",'
            'method="GET",status="200"} 2'),
      );
      expect(scraped, isNot(contains('/orders/41')));
    });

    test('a 404 collapses to one series, not one per probed path', () async {
      // An unmatched path is client-controlled. A scanner walking your URLs
      // would otherwise create a series each.
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/nothing');
      await app.get('/also-nothing');

      expect(
        (await app.get('/metrics')).body,
        contains('http_requests_total{route="<unmatched>",'
            'method="GET",status="404"} 2'),
      );
    });

    test('the histogram buckets are cumulative', () async {
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/orders/41');
      final scraped = (await app.get('/metrics')).body;

      final infinite =
          RegExp(r'le="\+Inf"\} (\d+)').firstMatch(scraped)!.group(1);

      expect(infinite, '1');
    });

    test('the scrape endpoint is plain text, as Prometheus expects', () async {
      final app = await example(metrics.buildApp());

      final response = await app.get('/metrics');

      expect(response.headers['content-type'], startsWith('text/plain'));
      expect(response.body, contains('# TYPE http_requests_total counter'));
    });
  });

  group('sessions', () {
    const secret = 'a-test-secret-that-is-long-enough-to-use';

    test('signing in sets a signed cookie with every attribute', () async {
      final app = await example(sessions.buildApp(secret: secret));

      final response = await app.send(
        'POST',
        '/sign-in',
        body: 'user=ada',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );
      final cookie = response.headers['set-cookie']!;

      expect(cookie, contains('HttpOnly'));
      expect(cookie, contains('Secure'));
      expect(cookie, contains('SameSite=Lax'));
    });

    test('the cookie round-trips to the user it names', () async {
      final app = await example(sessions.buildApp(secret: secret));
      final signer = sessions.Sessions(secret);

      final response = await app.get(
        '/me',
        headers: {'cookie': 'session=${_valueOf(signer.cookieFor("ada"))}'},
      );

      expect(app.object(response), {'user': 'ada'});
    });

    test('no cookie is a 401', () async {
      final app = await example(sessions.buildApp(secret: secret));

      expect((await app.get('/me')).statusCode, 401);
    });

    test('a tampered payload is refused', () async {
      // Signed, so the payload cannot be edited — a user cannot promote
      // themselves by rewriting the cookie.
      final app = await example(sessions.buildApp(secret: secret));
      final signer = sessions.Sessions(secret);
      final valid = _valueOf(signer.cookieFor('ada'));
      final forged = 'ZZZ${valid.substring(3)}';

      expect(
        (await app.get('/me', headers: {'cookie': 'session=$forged'}))
            .statusCode,
        401,
      );
    });

    test('a cookie signed with another secret is refused', () async {
      final app = await example(sessions.buildApp(secret: secret));
      final other = sessions.Sessions('a-completely-different-secret-value!!');

      expect(
        (await app.get(
          '/me',
          headers: {'cookie': 'session=${_valueOf(other.cookieFor("ada"))}'},
        ))
            .statusCode,
        401,
      );
    });

    test('an expired session is refused even though the signature is good',
        () async {
      // Max-Age is a hint to the browser. A client can keep sending an expired
      // cookie forever, so the expiry has to be signed and checked here.
      final app = await example(sessions.buildApp(secret: secret));
      final expired = sessions.Sessions(
        secret,
        lifetime: const Duration(days: -1),
      );

      expect(
        (await app.get(
          '/me',
          headers: {'cookie': 'session=${_valueOf(expired.cookieFor("ada"))}'},
        ))
            .statusCode,
        401,
      );
    });

    test('a malformed cookie is refused rather than crashing', () async {
      final app = await example(sessions.buildApp(secret: secret));

      for (final value in ['', 'nodot', 'a.b.c', 'not-base64.signature']) {
        expect(
          (await app.get('/me', headers: {'cookie': 'session=$value'}))
              .statusCode,
          401,
          reason: 'cookie "$value"',
        );
      }
    });

    test('signing out expires the cookie', () async {
      final app = await example(sessions.buildApp(secret: secret));

      final response = await app.send('POST', '/sign-out');

      expect(response.headers['set-cookie'], contains('Max-Age=0'));
    });
  });

  group('testing', () {
    test('the in-process handler needs no socket', () async {
      // Fast enough to run thousands of, and right for statuses and bodies.
      final app = testing.buildApp(testing.NoteStore(['only']));

      final response = await app.handler(
        Request('GET', Uri.parse('http://localhost/notes')),
      );

      expect(response.statusCode, 200);
      expect(await response.readAsString(), '["only"]');
    });

    test('the injected store is what the test asserts on', () async {
      final store = testing.NoteStore([]);
      final app = await example(testing.buildApp(store));

      await app.post('/notes', const {'title': 'written'});

      expect(store.titles, ['written']);
    });

    test('a socket catches what the wire does', () async {
      // gzip only exists on a socket. The in-process handler would return the
      // uncompressed body and the assertion would prove nothing.
      final store = testing.NoteStore(
        List.generate(80, (index) => 'a note with a reasonably long title'),
      );
      final app = await example(testing.buildApp(store));

      final client = HttpClient()..autoUncompress = false;
      addTearDown(client.close);
      final request = await client.getUrl(app.uri('/notes'));
      request.headers.set('accept-encoding', 'gzip');
      final response = await request.close();
      await response.drain<void>();

      expect(response.headers.value('content-encoding'), 'gzip');
    });

    test('a missing note is a 404 from the Result, not a throw', () async {
      final app = await example(testing.buildApp(testing.NoteStore([])));

      expect((await app.get('/notes/1')).statusCode, 404);
    });
  });

  group('error_handling', () {
    test('a throw is a 500 that says nothing about what broke', () async {
      // An exception message routinely carries a path, a SQL fragment, or a
      // connection string. Returning it is free reconnaissance.
      final app = await example(error_handling.buildApp(onError: (_, __) {}));

      final response = await app.get('/throws');

      expect(response.statusCode, 500);
      expect(app.object(response), {'error': 'Internal server error'});
      expect(response.body, isNot(contains('hunter2')));
    });

    test('onError sees the detail the client did not', () async {
      final faults = <Object>[];
      final app = await example(
        error_handling.buildApp(onError: (error, _) => faults.add(error)),
      );

      await app.get('/throws');

      expect(faults.single.toString(), contains('hunter2'));
    });

    test('a thrown Rejection keeps its own status', () async {
      final app = await example(error_handling.buildApp(onError: (_, __) {}));

      final response = await app.get('/rejects');

      expect(response.statusCode, 409);
      expect(app.object(response)['error'], 'that name is taken');
    });

    test('a returned Err answers the same as a thrown one', () async {
      final app = await example(error_handling.buildApp(onError: (_, __) {}));

      final thrown = await app.get('/rejects');
      final returned = await app.get('/returns');

      expect(returned.statusCode, thrown.statusCode);
      expect(returned.body, thrown.body);
    });

    test('a Rejection is not reported as a fault', () async {
      // It is a decision, not a bug. Reporting it would bury the real faults.
      final faults = <Object>[];
      final app = await example(
        error_handling.buildApp(onError: (error, _) => faults.add(error)),
      );

      await app.get('/rejects');
      await app.get('/returns');

      expect(faults, isEmpty);
    });
  });

  group('body_limits', () {
    test('a small body is accepted', () async {
      final app = await example(body_limits.buildApp());

      expect(
          (await app.post('/notes', const {'title': 'small'})).statusCode, 201);
    });

    test('a route tightened below the application ceiling refuses', () async {
      // An extractor asking for 16 KB is not widened by the router's 5 MB.
      final app = await example(body_limits.buildApp());

      final response = await app.send(
        'POST',
        '/notes',
        body: '{"title":"${'a' * 40000}"}',
        headers: const {'content-type': 'application/json'},
      );

      expect(response.statusCode, 413);
      expect(app.object(response)['error'], 'body exceeds 16384 bytes');
    });

    test('the route the ceiling was raised for accepts more', () async {
      final app = await example(body_limits.buildApp());

      final response = await app.send(
        'POST',
        '/avatar',
        body: 'a' * 100000,
        headers: const {'content-type': 'application/octet-stream'},
      );

      expect(response.statusCode, 201);
      expect(app.object(response)['bytes'], 100000);
    });
  });

  group('client_ip', () {
    test('with no proxies the socket address wins, header or not', () async {
      // An unproxied server has no reason to believe X-Forwarded-For, and a
      // default that trusts it is a spoofing hole for everyone who never
      // configured it.
      final app = await example(client_ip.buildApp());

      final response = await app.get(
        '/whoami',
        headers: {'x-forwarded-for': '1.2.3.4'},
      );

      expect(app.object(response)['client'], '127.0.0.1');
    });

    test('with one proxy the rightmost entry is the client', () async {
      final app = await example(client_ip.buildApp(trustedProxies: 1));

      final response = await app.get(
        '/whoami',
        headers: {'x-forwarded-for': '9.9.9.9, 203.0.113.7'},
      );

      expect(app.object(response)['client'], '203.0.113.7');
    });

    test('a spoofed leftmost entry is ignored', () async {
      // Anyone may send X-Forwarded-For: 1.2.3.4, and a proxy appends rather
      // than replaces. Counting from the left trusts whatever the client wrote.
      final app = await example(client_ip.buildApp(trustedProxies: 1));

      final response = await app.get(
        '/whoami',
        headers: {'x-forwarded-for': '1.2.3.4, 203.0.113.7'},
      );

      expect(app.object(response)['client'], '203.0.113.7');
    });

    test('more hops trusted than claimed falls back to the socket', () async {
      final app = await example(client_ip.buildApp(trustedProxies: 3));

      final response = await app.get(
        '/whoami',
        headers: {'x-forwarded-for': '1.2.3.4'},
      );

      expect(app.object(response)['client'], '127.0.0.1');
    });

    test('an absent header falls back to the socket', () async {
      final app = await example(client_ip.buildApp(trustedProxies: 1));

      expect(app.object(await app.get('/whoami'))['client'], '127.0.0.1');
    });
  });

  group('webhook_signatures', () {
    const secret = 'shh';

    String stamp([int offsetSeconds = 0]) =>
        '${DateTime.now().millisecondsSinceEpoch ~/ 1000 + offsetSeconds}';

    String sign(String body, String timestamp) => base64.encode(
          Hmac(sha256, utf8.encode(secret))
              .convert(utf8.encode('$timestamp.$body'))
              .bytes,
        );

    Future<http.Response> send(
      ExampleApp app,
      String body, {
      String? signature,
      String? timestamp,
    }) {
      final at = timestamp ?? stamp();
      return app.send(
        'POST',
        '/hooks',
        body: body,
        headers: {
          'content-type': 'application/json',
          'x-timestamp': at,
          if (signature != null) 'x-signature': signature,
        },
      );
    }

    test('a correctly signed request is accepted', () async {
      final app = await example(webhooks.buildApp(secret: secret));
      const body = '{"event":"invoice.paid"}';
      final at = stamp();

      final response =
          await send(app, body, signature: sign(body, at), timestamp: at);

      expect(response.statusCode, 200);
      expect(app.object(response), {'received': 'invoice.paid'});
    });

    test('a wrong signature is refused', () async {
      final app = await example(webhooks.buildApp(secret: secret));

      expect((await send(app, '{"event":"x"}', signature: 'wrong')).statusCode,
          401);
    });

    test('a body altered after signing is refused', () async {
      final app = await example(webhooks.buildApp(secret: secret));
      final at = stamp();

      final response = await send(
        app,
        '{"amount":1000000}',
        signature: sign('{"amount":1}', at),
        timestamp: at,
      );

      expect(response.statusCode, 401);
    });

    test('re-serializing the body would break the signature', () async {
      // Same object, different bytes: key order differs. Verifying a re-encoded
      // body compares against something nobody signed.
      final app = await example(webhooks.buildApp(secret: secret));
      const sent = '{"a":1,"b":2}';
      final at = stamp();
      final reordered = jsonEncode(jsonDecode('{"b":2,"a":1}'));

      expect(reordered, isNot(sent));
      expect(
        (await send(app, reordered, signature: sign(sent, at), timestamp: at))
            .statusCode,
        401,
      );
    });

    test('an old timestamp is refused, so a capture cannot be replayed',
        () async {
      final app = await example(webhooks.buildApp(secret: secret));
      const body = '{"event":"x"}';
      final old = stamp(-3600);

      expect(
        (await send(app, body, signature: sign(body, old), timestamp: old))
            .statusCode,
        401,
      );
    });

    test('a timestamp far in the future is refused too', () async {
      final app = await example(webhooks.buildApp(secret: secret));
      const body = '{"event":"x"}';
      final ahead = stamp(3600);

      expect(
        (await send(app, body, signature: sign(body, ahead), timestamp: ahead))
            .statusCode,
        401,
      );
    });

    test('a missing signature is refused', () async {
      final app = await example(webhooks.buildApp(secret: secret));

      expect((await send(app, '{}')).statusCode, 401);
    });

    test('the refusal says nothing about which half failed', () async {
      // Telling an attacker whether the signature or the clock was wrong halves
      // their search.
      final app = await example(webhooks.buildApp(secret: secret));
      final old = stamp(-3600);

      final badSignature = await send(app, '{}', signature: 'wrong');
      final badClock =
          await send(app, '{}', signature: sign('{}', old), timestamp: old);

      expect(badSignature.body, badClock.body);
    });
  });

  group('print_request_response', () {
    test('the handler still receives the body the layer read', () async {
      // A body reads once. Without handing a fresh one down, the handler finds
      // an empty body and answers a puzzling 400.
      final app = await example(print_both.buildApp(log: (_) {}));

      final response = await app.post('/notes', const {'title': 'buy milk'});

      expect(response.statusCode, 201);
      expect(app.object(response), {'id': 1, 'title': 'buy milk'});
    });

    test('it logs the request and the response', () async {
      final lines = <String>[];
      final app = await example(print_both.buildApp(log: lines.add));

      await app.post('/notes', const {'title': 'buy milk'});

      expect(lines.first, startsWith('--> POST /notes'));
      expect(lines, contains('--> {"title":"buy milk"}'));
      expect(lines.last, startsWith('<-- 201'));
    });

    test('credential headers are redacted', () async {
      final lines = <String>[];
      final app = await example(print_both.buildApp(log: lines.add));

      await app.post(
        '/notes',
        const {'title': 'x'},
        headers: {'authorization': 'Bearer secret-token'},
      );

      expect(lines.first, contains('<redacted>'));
      expect(lines.join(), isNot(contains('secret-token')));
    });
  });

  group('health_checks', () {
    test('liveness answers without touching a dependency', () async {
      // When the database blips, liveness must not fail on every instance at
      // once and get the whole fleet restarted.
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': false},
      );
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/live')).statusCode, 200);
    });

    test('readiness fails when a dependency is down', () async {
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': false},
      );
      final app = await example(health_checks.buildApp(checks));

      final response = await app.get('/health/ready');

      expect(response.statusCode, 503);
      expect(app.object(response)['error'], 'a dependency is down');
    });

    test('readiness reports up or down and nothing else', () async {
      // No versions, no connection strings, no error text: the endpoint is
      // unauthenticated.
      final checks = health_checks.HealthChecks(
        probe: () async => const {'database': true, 'cache': true},
      );
      final app = await example(health_checks.buildApp(checks));

      expect(app.object(await app.get('/health/ready'))['dependencies'], {
        'database': true,
        'cache': true,
      });
    });

    test('draining fails readiness while liveness still passes', () async {
      final checks = health_checks.HealthChecks();
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/ready')).statusCode, 200);
      checks.markDraining();

      expect((await app.get('/health/ready')).statusCode, 503);
      // Still alive, so nothing kills it mid-drain.
      expect((await app.get('/health/live')).statusCode, 200);
    });

    test('startup fails until warm-up finishes', () async {
      final checks = health_checks.HealthChecks();
      final app = await example(health_checks.buildApp(checks));

      expect((await app.get('/health/startup')).statusCode, 503);
      checks.markStarted();
      expect((await app.get('/health/startup')).statusCode, 200);
    });
  });

  group('background_tasks', () {
    test('the response does not wait for the task', () async {
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));

      final response =
          await app.post('/orders', const {'email': 'ada@example.com'});

      expect(response.statusCode, 201);
      expect(app.object(response), {'placed': true, 'receiptQueued': true});
      // Not sent yet: the handler returned before the work finished.
      expect(app.object(await app.get('/receipts'))['sent'], isEmpty);
    });

    test('the task finishes afterwards', () async {
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));

      await app.post('/orders', const {'email': 'ada@example.com'});
      await tasks.settled(const Duration(seconds: 2));

      expect(
          app.object(await app.get('/receipts'))['sent'], ['ada@example.com']);
    });

    test('a draining registry refuses, and the handler is told', () async {
      // The order was placed and the receipt will not be sent. Knowing that is
      // the difference between an outbox row and a silent loss.
      final tasks = BackgroundTasks(onError: (_, __) {});
      final app = await example(background_tasks.buildApp(tasks));
      await tasks.close(within: const Duration(milliseconds: 10));

      final response =
          await app.post('/orders', const {'email': 'ada@example.com'});

      expect(app.object(response)['receiptQueued'], isFalse);
      expect(app.object(await app.get('/receipts'))['sent'], isEmpty);
    });
  });
}

/// The `seen` count out of a `/whoami` body.
int app0(http.Response response) =>
    jsonDecode(response.body)['seen'] as int? ?? -1;

/// The cookie value out of a `Set-Cookie` header.
String _valueOf(String setCookie) =>
    setCookie.split(';').first.split('=').sublist(1).join('=');

/// Keeps every span so a test can look at one.
final class CollectingExporter implements SpanExporter {
  /// What has been exported.
  final spans = <Span>[];

  @override
  void export(Span span) => spans.add(span);
}
