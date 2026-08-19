import 'dart:io';

import 'package:dust_server/server.dart';

/// Handing the failure to the handler instead of short-circuiting.
///
/// `request.query<int>('page')` **throws** its rejection, which ends the
/// request. That is what you want almost always: the first bad input stops the
/// work, and the handler below it never runs against half a request.
///
/// `fallible(...)` hands you the `Result` instead. Two reasons to want it:
///
/// * To answer with something other than the extractor's status — a redirect
///   back to a form, say, rather than a 400 a browser shows as raw JSON.
/// * To collect several failures and report them together, the way a form does.
///
/// Run it with `dart run example/fallible_extraction.dart`:
///
/// ```bash
/// curl -s 'localhost:8080/report?from=1&to=9'
/// curl -s 'localhost:8080/report'              # 422, both fields at once
/// curl -s 'localhost:8080/report?from=x&to=9'  # 422, naming from
/// curl -i 'localhost:8080/browse?page=x'       # 303 back to the form
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/report', get(report))
    ..route('/browse', get(browse));
}

/// `GET /report?from=&to=` — every bad parameter reported at once.
Future<Result<Map<String, Object?>, Rejection>> report(Request request) async {
  final from = await request.extract(fallible(query<int>('from')));
  final to = await request.extract(fallible(query<int>('to')));

  // A record pattern reads both at once, so the success case names its values
  // without a cast and the failure case still sees every error.
  return switch ((from, to)) {
    (Ok(value: final start), Ok(value: final end)) =>
      Ok({'from': start, 'to': end}),
    _ => Err(
        Rejection.unprocessable({
          if (from case Err(:final error)) 'from': [error.message],
          if (to case Err(:final error)) 'to': [error.message],
        }),
      ),
  };
}

/// `GET /browse?page=` — a bad page sends a browser back, not a 400.
Future<Response> browse(Request request) async {
  final page = await request.extract(fallible(query<int?>('page')));

  return switch (page) {
    Err() => Redirect.to('/browse').intoResponse(),
    Ok(:final value) => jsonResponse({'page': value ?? 1}),
  };
}
