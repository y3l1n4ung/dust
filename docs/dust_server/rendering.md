# Rendering

Everything a Flutter application's backend needs to serve a web front end
alongside its API: HTML from templates, static assets, and a single-page
fallback.

## Templates

Templating is not implemented here. `mustache_template` is the most used
template package on pub, so this package adapts it and defines a seam so
anything else can replace it.

```dart
final templates = MustacheTemplates.fromDirectory('web/templates');

final app = Router()
  ..route('/', get(homePage))
  ..withState<TemplateEngine>(templates);

Future<Response> homePage(Request request) async {
  final engine = await const StateExtractable<TemplateEngine>().extract(request);
  return switch (engine) {
    Ok(:final value) => render(value, 'todos/index', {'todos': todos}),
    Err(:final error) => error.intoResponse(),
  };
}
```

`MustacheTemplates.fromDirectory` names a template by its path below the
directory without the extension, so `web/templates/mail/welcome.html` is
`mail/welcome`. Partials resolve against the same set, so a layout including a
header costs one parse each.

An unknown name throws `TemplateNotFound`, listing what does exist, because a
typo should be a failure rather than an empty page.

### Escaping

`{{value}}` is escaped, `{{{value}}}` is not. The escaping covers `&`, `<`, `>`,
both quote forms, **and `/`** — the last one closes the `</script>` break-out
that escaping only angle brackets would leave open.

### Mustache or Jinja

`mustache_template` is the one that ships, because escaping-by-default is the
right posture for a page assembled from what users typed. It is not the only
option: `TemplateEngine` is an interface, and
`test/templating/pluggable_engine_test.dart` drives the whole rendering path
through `package:jinja` to prove it.

| | mustache | jinja |
| :--- | :--- | :--- |
| Escaping | every `{{value}}`, including `/` | **none by default** — write `{{ v \| e }}` |
| Layouts | partials only (`{{> header}}`) | `{% extends %}` / `{% block %}` |
| Logic | none, by design | `if`, `for`, filters, expressions |
| Partial names | letters, digits, `-`, `_`, `.` — no `/` | any loader key |
| Shipped | yes | adapter, ~15 lines |

Pick mustache when templates should stay dumb and the escaping should not be
something anyone has to remember. Pick jinja when pages share a layout — that
is the one thing mustache genuinely cannot express, and faking it with partials
gets ugly fast — and accept that **you now own escaping on every
interpolation**.

The whole adapter:

```dart
final class JinjaTemplates implements TemplateEngine {
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
```

Handlers read `TemplateEngine`, never the adapter, so swapping engines is a
change at the composition site and nowhere else.

### Is this SSR?

It is server-rendered HTML: the response carries the finished page, so a
crawler and a reader with no JavaScript both get content. It is not the
hydration-and-islands sense of the term — there is no client runtime to hand
state to, and no streaming of a partial document.

For a built browser application, see [Web applications](web-apps.md). The two
compose on one router: render the pages that must be readable immediately, and
serve the application on the rest.

## Static files

`staticFiles` passes through to `shelf_static`, which already handles content
types, ranges, conditional requests, and refusing paths that climb out of the
directory.

```dart
final app = Router()
  ..mount('/assets', staticFiles('web/assets'))
  ..route('/api/todos', get(listTodos));
```

Mount it, so the handler sees paths relative to where it lives.

## Single-page applications

`singlePageApp` answers one file for every document request:

```dart
final app = Router()
  ..mount('/api', apiRoutes)
  ..mount('/assets', staticFiles('web/assets'))
  ..fallback(singlePageApp('web/index.html'));
```

For a whole build directory — the usual case — use `staticFiles(directory, html: true)`
instead, which also gets the cache policy right. See [Web applications](web-apps.md).

The client-side router owns the path, so the server answers the same document
for anything nothing else matched. Only `GET` and `HEAD` are answered: a `POST`
to an unmatched path is a wrong request, not a page view, and handing it an HTML
document would hide that.

Dart normalizes `..` while parsing a URI, so a traversal never arrives as one;
it collapses before matching, and no file outside the directory is reachable.

## Trying it

```bash
dart run example/chat_server.dart
```

```bash
# a rendered page, with the charset stated
curl -i localhost:8081/rooms/general | head -3

# what a visitor typed comes back escaped, not executed
curl -s -X POST 'localhost:8081/api/rooms?name=xss'
curl -s localhost:8081/rooms/xss
```

Mustache escapes every interpolation, so markup in a message renders as
`&lt;script&gt;` rather than running.
