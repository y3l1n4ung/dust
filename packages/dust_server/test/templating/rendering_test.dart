import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

import '../support.dart';

final _templates = MustacheTemplates({
  'layout': '<!doctype html><title>{{title}}</title>{{> body}}',
  'body': '<ul>{{#todos}}<li>{{title}}</li>{{/todos}}</ul>',
  'missing-value': '{{nope}}',
});

Router _app() {
  return Router()
    ..route('/page', get((request) async {
      final engine =
          await const StateExtractable<TemplateEngine>().extract(request);
      return switch (engine) {
        Ok(:final value) => render(value, 'layout', {
            'title': 'Todos',
            'todos': [
              {'title': 'first'},
            ],
          }),
        Err(:final error) => error.intoResponse(),
      };
    }))
    ..route(
        '/broken',
        get((request) async => guard(() async {
              return render(_templates, 'missing-value', const {});
            })))
    ..withState<TemplateEngine>(_templates);
}

void main() {
  group('server-rendered HTML', () {
    test('answers with a rendered document', () async {
      final response = await _app().handler(request('GET', '/page'));

      expect(response.statusCode, 200);
      expect(
        await response.readAsString(),
        '<!doctype html><title>Todos</title><ul><li>first</li></ul>',
      );
    });

    test('states the content type and charset', () async {
      final response = await _app().handler(request('GET', '/page'));

      expect(response.headers['content-type'], 'text/html; charset=utf-8');
    });

    test('carries a status when one is given', () {
      final response = htmlResponse('<p>gone</p>', status: 410);

      expect(response.statusCode, 410);
    });

    test('turns a render failure into a 500, not a broken page', () async {
      final response = await _app().handler(request('GET', '/broken'));

      expect(response.statusCode, 500);
    });

    test('reaches the engine through router state', () async {
      final response = await (Router()
            ..route('/page', get((request) async {
              final engine = await const StateExtractable<TemplateEngine>()
                  .extract(request);
              return switch (engine) {
                Ok() => htmlResponse('<p>ok</p>'),
                Err(:final error) => error.intoResponse(),
              };
            })))
          .handler(request('GET', '/page'));

      expect(response.statusCode, 500, reason: 'no engine was attached');
    });
  });

  group('over a real socket', () {
    late ServerHandle server;

    setUp(() async {
      server = await serve(_app(), InternetAddress.loopbackIPv4, 0);
    });

    tearDown(() => server.close());

    test('a browser receives the document', () async {
      final response = await http.get(
        Uri.parse('http://${server.address.host}:${server.port}/page'),
      );

      expect(response.statusCode, 200);
      expect(response.headers['content-type'], 'text/html; charset=utf-8');
      expect(response.body, contains('<li>first</li>'));
    });
  });
}
