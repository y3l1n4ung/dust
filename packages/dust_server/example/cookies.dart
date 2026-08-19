import 'dart:io';

import 'package:dust_server/server.dart';

/// Reading cookies, and setting one safely.
///
/// Reading is an extractor: `cookie<T>('name')` for one, coerced, and
/// `cookies()` for the whole jar. A nullable type makes it optional.
///
/// **Setting** one is a header you write yourself, and the attributes are not
/// optional extras — each closes a specific hole:
///
/// | Attribute | Without it |
/// | :--- | :--- |
/// | `HttpOnly` | any script on the page can read the session |
/// | `Secure` | the cookie travels over plain HTTP and can be sniffed |
/// | `SameSite=Lax` | the cookie rides along on cross-site requests, which is CSRF |
/// | `Path=/` | the cookie is scoped narrower or wider than you meant |
/// | `Max-Age` | it is a session cookie, gone when the browser closes |
///
/// A cookie value is client input on the way back in. Signing or encrypting it
/// is what stops a user editing `role=admin` into their own cookie jar — see
/// `sessions.dart`.
///
/// Run it with `dart run example/cookies.dart`:
///
/// ```bash
/// curl -i localhost:8080/sign-in
/// curl -s localhost:8080/whoami -H 'cookie: user=ada'
/// curl -s localhost:8080/whoami                        # null, not an error
/// curl -s localhost:8080/all -H 'cookie: a=1; b=2'
/// curl -i localhost:8080/sign-out
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
    ..route('/sign-in', get(signIn))
    ..route('/sign-out', get(signOut))
    ..route('/whoami', get(whoAmI))
    ..route('/all', get(allCookies));
}

/// `GET /sign-in` — sets the cookie.
Response signIn(Request request) {
  return Response.ok(
    '{"signedIn":true}',
    headers: {
      'content-type': 'application/json',
      'set-cookie':
          cookieHeader('user', 'ada', maxAge: const Duration(days: 7)),
    },
  );
}

/// `GET /sign-out` — clears it, by setting it expired.
///
/// There is no "delete cookie" instruction in HTTP. You send the same cookie
/// with `Max-Age=0`, and the attributes have to match or the browser keeps the
/// original.
Response signOut(Request request) {
  return Response.ok(
    '{"signedIn":false}',
    headers: {
      'content-type': 'application/json',
      'set-cookie': cookieHeader('user', '', maxAge: Duration.zero),
    },
  );
}

/// `GET /whoami` — one cookie, or null.
Future<Map<String, Object?>> whoAmI(Request request) async => {
      'user': await request.cookie<String?>('user'),
    };

/// `GET /all` — the whole jar.
Future<Map<String, Object?>> allCookies(Request request) async {
  final jar = await request.cookies();

  return {'cookies': jar.values};
}

/// Builds a `Set-Cookie` value with the attributes that matter.
///
/// `secure` defaults to true. A default of false is the kind that ships.
String cookieHeader(
  String name,
  String value, {
  Duration maxAge = const Duration(hours: 1),
  bool secure = true,
  bool httpOnly = true,
  String sameSite = 'Lax',
  String path = '/',
}) {
  return [
    '$name=${Uri.encodeComponent(value)}',
    'Path=$path',
    'Max-Age=${maxAge.inSeconds}',
    'SameSite=$sameSite',
    if (httpOnly) 'HttpOnly',
    if (secure) 'Secure',
  ].join('; ');
}
