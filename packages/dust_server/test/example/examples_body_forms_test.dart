import 'package:test/test.dart';
import '../../example/form_body.dart' as form_body;
import '../../example/parse_body_by_content_type.dart' as by_content_type;
import 'serve.dart';

/// Reading a request: paths, queries, headers, bodies and forms.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Form and multipart bodies, and dispatch by content type.
void main() {
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
