import 'package:dust_server/server.dart';
import 'package:test/test.dart';

void main() {
  group('MustacheTemplates', () {
    test('renders values into a template', () {
      final templates = MustacheTemplates({'hello': 'Hi {{name}}!'});

      expect(templates.render('hello', {'name': 'Ada'}), 'Hi Ada!');
    });

    test('escapes by default, including the solidus', () {
      final templates = MustacheTemplates({'x': '<p>{{value}}</p>'});

      // `/` is escaped as well, which closes the `</script>` break-out that
      // escaping only the angle brackets would leave open.
      expect(
        templates.render('x', {'value': '<script>alert(1)</script>'}),
        '<p>&lt;script&gt;alert(1)&lt;&#x2F;script&gt;</p>',
      );
    });

    test('escapes quotes in an attribute position', () {
      final templates = MustacheTemplates({'x': '<a title="{{value}}">'});

      expect(
        templates.render('x', {'value': '" onclick="steal()'}),
        contains('&quot;'),
      );
    });

    test('leaves triple braces unescaped', () {
      final templates = MustacheTemplates({'x': '{{{markup}}}'});

      expect(templates.render('x', {'markup': '<b>bold</b>'}), '<b>bold</b>');
    });

    test('iterates a list section', () {
      final templates = MustacheTemplates({
        'list': '{{#todos}}<li>{{title}}</li>{{/todos}}',
      });

      final rendered = templates.render('list', {
        'todos': [
          {'title': 'first'},
          {'title': 'second'},
        ],
      });

      expect(rendered, '<li>first</li><li>second</li>');
    });

    test('renders an inverted section when empty', () {
      final templates = MustacheTemplates({
        'list': '{{^todos}}nothing yet{{/todos}}',
      });

      expect(templates.render('list', {'todos': <Object>[]}), 'nothing yet');
    });

    test('resolves partials against the same set', () {
      final templates = MustacheTemplates({
        'layout': '<html>{{> header}}<main>{{body}}</main></html>',
        'header': '<h1>{{title}}</h1>',
      });

      expect(
        templates.render('layout', {'title': 'Todos', 'body': 'hello'}),
        '<html><h1>Todos</h1><main>hello</main></html>',
      );
    });

    test('reads nested values by dotted path', () {
      final templates = MustacheTemplates({'x': '{{user.name}}'});

      expect(
        templates.render('x', {
          'user': {'name': 'Ada'},
        }),
        'Ada',
      );
    });

    test('lists what it knows', () {
      final templates = MustacheTemplates({'b': '', 'a': ''});

      expect(templates.names, ['a', 'b']);
    });

    test('refuses an unknown template by name', () {
      final templates = MustacheTemplates({'known': ''});

      expect(
        () => templates.render('missing', const {}),
        throwsA(
          isA<TemplateNotFound>().having(
            (error) => error.toString(),
            'message',
            allOf(contains('missing'), contains('known')),
          ),
        ),
      );
    });

    test('refuses a template that will not parse', () {
      expect(
        () => MustacheTemplates({'broken': '{{#open}}never closed'}),
        throwsA(anything),
      );
    });

    test('refuses a value the template asked for but nobody supplied', () {
      final templates = MustacheTemplates({'x': '{{missing}}'});

      expect(() => templates.render('x', const {}), throwsA(anything));
    });
  });
}
