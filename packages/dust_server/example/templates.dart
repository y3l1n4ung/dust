import 'dart:io';

import 'package:dust_server/server.dart';

/// Rendering HTML from a template.
///
/// Mustache escapes every `{{value}}`, which is the whole reason to use a
/// template engine rather than string interpolation. A product description typed
/// by a vendor cannot become a `<script>` on the page.
///
/// `{{{value}}}` — three braces — does **not** escape. Hand it only markup you
/// produced yourself. Every stored-XSS bug is a third brace around something a
/// user sent.
///
/// Two things about partials are worth knowing before you hit them:
///
/// * A partial name is fixed at parse time, so a layout cannot choose its
///   content template. Render the page first and pass the result to the layout —
///   `page` below.
/// * `MustacheTemplates.fromDirectory` names a template by its path, but a
///   `{{> partial}}` tag allows only letters, digits, `-`, `_` and `.`. A
///   fragment meant to be included has to carry a flat name.
///
/// Run it with `dart run example/templates.dart`:
///
/// ```bash
/// curl -s localhost:8080/
/// curl -s localhost:8080/notes/1
/// curl -s 'localhost:8080/notes/2'   # the title is escaped, not executed
/// curl -si localhost:8080/notes/9    # 404, as a page rather than as JSON
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// The pages, held in a map so the example runs from a bare checkout.
///
/// A deployed application uses `MustacheTemplates.fromDirectory('web')` and
/// changes nothing else.
final TemplateEngine templates = MustacheTemplates(const {
  'layout': '<!doctype html><html lang="en"><head><meta charset="utf-8">'
      '<title>{{title}}</title></head><body>{{{body}}}</body></html>',
  'index': '<h1>Notes</h1><ul>'
      '{{#notes}}<li><a href="/notes/{{id}}">{{title}}</a></li>{{/notes}}'
      '</ul>',
  'note': '<h1>{{title}}</h1><p>{{body}}</p>',
  'not-found': '<h1>Not here</h1><p><a href="/">Back</a></p>',
});

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/', get(index))
    ..route('/notes/{id}', get(readNote))
    ..withState<TemplateEngine>(templates);
}

/// The notes, one of which is named by somebody hostile.
const notes = {
  1: ('First note', 'Nothing unusual.'),
  2: ('<script>alert(1)</script>', 'The title above is escaped by the engine.'),
};

/// `GET /`
Future<Response> index(Request request) async {
  final engine = await request.state<TemplateEngine>();

  return page(engine, 'index', 'Notes', {
    'notes': [
      for (final entry in notes.entries)
        {'id': entry.key, 'title': entry.value.$1},
    ],
  });
}

/// `GET /notes/{id}` — a 404 as a page, because the caller is holding a browser.
Future<Response> readNote(Request request) async {
  final id = await request.path<int>('id');
  final engine = await request.state<TemplateEngine>();

  final note = notes[id];
  if (note == null) {
    return page(engine, 'not-found', 'Not here', const {}, status: 404);
  }

  return page(engine, 'note', note.$1, {'title': note.$1, 'body': note.$2});
}

/// Renders [template] inside the shared layout.
///
/// Two passes, since a partial name cannot be chosen at render time. The layout's
/// `{{{body}}}` is the one unescaped slot, and it only ever receives markup this
/// function produced.
Response page(
  TemplateEngine engine,
  String template,
  String title,
  Map<String, Object?> values, {
  int status = 200,
}) {
  return render(
    engine,
    'layout',
    {'title': title, 'body': engine.render(template, values)},
    status: status,
  );
}
