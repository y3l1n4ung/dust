import 'dart:io';

import 'package:dust_server/server.dart';

/// Decoding an HTML form post.
///
/// `application/x-www-form-urlencoded` is what a `<form method="post">` sends
/// without JavaScript, so this is the shape a server-rendered page needs.
///
/// `form()` hands back a [FormMap], and its `field<T>` returns a **`Result`**
/// rather than throwing. That is the difference from `body()`: a form is the one
/// place you usually want every mistake at once, so the page can be re-rendered
/// with all of them marked, instead of the first failure ending the request.
///
/// Run it with `dart run example/form_body.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/subscribe \
///   -d 'email=ada@example.com&quantity=2'
/// curl -s -X POST localhost:8080/subscribe -d 'email=ada@example.com'
/// curl -s -X POST localhost:8080/subscribe -d 'quantity=x'   # 422, both fields
/// curl -i -X POST localhost:8080/subscribe \
///   -H 'content-type: application/json' -d '{}'              # 415
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/subscribe', post(subscribe));

/// `POST /subscribe`
Future<Result<Map<String, Object?>, Rejection>> subscribe(
  Request request,
) async {
  final form = await request.form();

  final errors = <String, List<String>>{};
  final email = form.field<String>('email').okOr(errors, 'email');
  final quantity = form.field<int?>('quantity').okOr(errors, 'quantity');

  if (errors.isNotEmpty) return Err(Rejection.unprocessable(errors));

  return Ok({'email': email, 'quantity': quantity ?? 1});
}

/// Collects the failure instead of ending the request on it.
extension on Result<Object?, Rejection> {
  T? okOr<T>(Map<String, List<String>> errors, String field) {
    switch (this) {
      case Ok(:final value):
        return value as T?;
      case Err(:final error):
        errors[field] = [error.message];
        return null;
    }
  }
}
