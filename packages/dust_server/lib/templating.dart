/// Server-rendered HTML: a template seam, an adapter, and the response helpers.
///
/// ```dart
/// import 'package:dust_server/templating.dart';
///
/// final templates = MustacheTemplates.fromDirectory('web/templates');
///
/// Future<Response> page(Request request) async =>
///     render(templates, 'todos/index', {'todos': todos});
/// ```
///
/// The adapter is over `mustache_template`; implement `TemplateEngine` to use
/// anything else. Everything here is also exported from
/// `package:dust_server/server.dart`.
library;

export 'src/response/html.dart';
export 'src/template/template.dart';
