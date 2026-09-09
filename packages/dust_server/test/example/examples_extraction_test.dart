import 'package:test/test.dart';
import '../../example/custom_extractor.dart' as custom_extractor;
import '../../example/customize_rejection.dart' as customize_rejection;
import '../../example/fallible_extraction.dart' as fallible_extraction;
import '../../example/optional_extraction.dart' as optional_extraction;
import '../../example/state.dart' as state_example;
import '../../example/validation_422.dart' as validation_422;
import 'serve.dart';

/// Extractors, and what happens when one cannot produce a value.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
void main() {
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
}
