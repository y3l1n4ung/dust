import 'package:shelf/shelf.dart';

import '../template/template_engine.dart';

/// Answers with [markup] as an HTML document.
///
/// The charset is stated, since a browser guessing at encoding is a rendering
/// bug waiting to happen.
Response htmlResponse(String markup, {int status = 200}) {
  return Response(
    status,
    body: markup,
    headers: const {'content-type': 'text/html; charset=utf-8'},
  );
}

/// Renders [template] with [values] and answers with the result.
///
/// ```dart
/// Future<Response> page(Request request) async {
///   final templates = await const StateExtractable<TemplateEngine>()
///       .extract(request);
///   return render(templates, 'todos/index', {'todos': todos});
/// }
/// ```
Response render(
  TemplateEngine engine,
  String template,
  Map<String, Object?> values, {
  int status = 200,
}) {
  return htmlResponse(engine.render(template, values), status: status);
}
