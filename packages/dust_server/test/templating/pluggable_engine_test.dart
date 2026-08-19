import 'package:dust_server/server.dart';
import 'package:jinja/jinja.dart' as jinja;
import 'package:test/test.dart';

/// `TemplateEngine` is an interface so an application is not married to
/// mustache. That claim is worth only as much as a second engine behind it, so
/// this drives the whole rendering path — `withState`, `render`, the HTML
/// encoder — through a Jinja adapter.
///
/// The adapter below is the entire integration. Copy it if you want template
/// inheritance and filters instead of mustache's logic-less discipline.

/// A [TemplateEngine] over `package:jinja`.
final class JinjaTemplates implements TemplateEngine {
  /// Compiles [sources], keyed by template name.
  JinjaTemplates(Map<String, String> sources)
      : _environment = jinja.Environment(loader: jinja.MapLoader(sources)),
        _names = sources.keys.toList()..sort();

  final jinja.Environment _environment;
  final List<String> _names;

  @override
  String render(String name, Map<String, Object?> values) {
    if (!_names.contains(name)) throw TemplateNotFound(name, _names);
    return _environment.getTemplate(name).render(values);
  }
}

void main() {
  late TemplateEngine templates;

  setUp(() {
    templates = JinjaTemplates({
      'layout': '<html><body>{% block content %}{% endblock %}</body></html>',
      'page': '{% extends "layout" %}'
          '{% block content %}<h1>{{ title }}</h1>{% endblock %}',
      'list': '{% for todo in todos %}<li>{{ todo }}</li>{% endfor %}',
      'filtered': '{{ name | upper }}',
      'conditional': '{% if done %}done{% else %}open{% endif %}',
      'escaped': '{{ body }}',
    });
  });

  group('a Jinja engine behind the interface', () {
    test('renders an interpolation', () {
      expect(templates.render('filtered', {'name': 'dust'}), 'DUST');
    });

    test('renders a loop', () {
      expect(
        templates.render('list', {
          'todos': ['a', 'b'],
        }),
        '<li>a</li><li>b</li>',
      );
    });

    test('renders a conditional', () {
      expect(templates.render('conditional', {'done': true}), 'done');
      expect(templates.render('conditional', {'done': false}), 'open');
    });

    test('inherits a layout, which mustache cannot express', () {
      expect(
        templates.render('page', {'title': 'Todos'}),
        '<html><body><h1>Todos</h1></body></html>',
      );
    });

    test('reports a missing template the way the interface says', () {
      expect(
        () => templates.render('absent', const {}),
        throwsA(isA<TemplateNotFound>()),
      );
    });
  });

  group('the rendering path', () {
    Future<Response> serve(TemplateEngine engine) async {
      final app = Router()
        ..route('/', get((request) async {
          final templates = await state<TemplateEngine>().require(request);
          return render(templates, 'page', {'title': 'Todos'});
        }))
        ..withState<TemplateEngine>(engine);

      return app.handler(Request('GET', Uri.parse('http://localhost/')));
    }

    test('carries a Jinja engine through withState', () async {
      final response = await serve(templates);

      expect(await response.readAsString(), contains('<h1>Todos</h1>'));
    });

    test('answers HTML with a stated charset either way', () async {
      final response = await serve(templates);

      expect(response.headers['content-type'], 'text/html; charset=utf-8');
    });

    test('swaps engines without touching the handler', () async {
      final mustache = MustacheTemplates(const {
        'page': '<html><body><h1>{{title}}</h1></body></html>',
      });

      final fromJinja = await serve(templates);
      final fromMustache = await serve(mustache);

      expect(
        await fromMustache.readAsString(),
        await fromJinja.readAsString(),
      );
    });
  });

  group('escaping, which is where the two engines differ most', () {
    test('mustache escapes every interpolation by default', () {
      final mustache = MustacheTemplates(const {'escaped': '{{body}}'});

      expect(
        mustache.render('escaped', {'body': '<script>'}),
        '&lt;script&gt;',
      );
    });

    test('jinja does not escape by default, so a template can inject', () {
      // `Environment` in jinja 0.6.6 has no autoescape switch; escaping is per
      // interpolation. A template that forgets it renders whatever a visitor
      // typed straight into the page.
      expect(templates.render('escaped', {'body': '<script>'}), '<script>');
    });

    test('jinja escapes when the template asks with the e filter', () {
      final safe = JinjaTemplates(const {'escaped': '{{ body | e }}'});

      expect(safe.render('escaped', {'body': '<script>'}), '&lt;script&gt;');
    });

    test('the difference survives the whole rendering path', () async {
      final unsafe = JinjaTemplates(const {'page': '{{ title }}'});
      final app = Router()
        ..route('/', get((request) async {
          final engine = await state<TemplateEngine>().require(request);
          return render(engine, 'page', {'title': '<script>alert(1)</script>'});
        }))
        ..withState<TemplateEngine>(unsafe);

      final response = await app.handler(
        Request('GET', Uri.parse('http://localhost/')),
      );

      // Pinned, not endorsed: choosing jinja means owning escaping.
      expect(await response.readAsString(), contains('<script>'));
    });
  });
}
