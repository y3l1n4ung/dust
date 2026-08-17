import 'package:test/test.dart';

import '../../example/customize_rejection.dart' as customize_rejection;
import '../../example/form_body.dart' as form_body;
import '../../example/headers_and_host.dart' as headers_and_host;
import '../../example/hello_world.dart' as hello_world;
import '../../example/json_body.dart' as json_body;
import '../../example/multipart_form.dart' as multipart_form;
import '../../example/parse_body_by_content_type.dart' as by_content_type;
import '../../example/path_params.dart' as path_params;
import '../../example/query_params.dart' as query_params;
import '../../example/routing.dart' as routing;
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
}
