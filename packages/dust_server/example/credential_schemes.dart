import 'dart:io';

import 'package:dust_server/server.dart';

/// Accepting whichever credential arrives.
///
/// A bearer token, an API key, HTTP Basic, and a session cookie are four wire
/// formats for one question: who is calling. Each is a `FromRequestParts` that
/// answers with the same type, so `firstOf` tries them in order and the handler
/// never learns which header carried it.
///
/// This is why an extractor must return `Err` rather than throw. `firstOf`
/// composes by inspecting the failure and moving on; a throw would end the
/// request at the first scheme and the other three would never run.
///
/// > **What the 401 says is the last scheme's message.** `firstOf` returns the
/// > final `Err` when every scheme declines, so a caller with no credential at
/// > all is told `expected a session cookie` and gets `WWW-Authenticate: Cookie`
/// > — accurate for the scheme that ran last, and misleading about the other
/// > three. HTTP allows several challenges in one response; if the message
/// > matters to your clients, order the list so the most likely scheme is last,
/// > or answer the 401 yourself rather than letting the last extractor do it.
///
/// > **`allowQuery: false` on the API key.** `ApiKeyExtractable` will read
/// > `?api_key=` if you let it, and the default lets it. A key in a URL is
/// > written to every access log, proxy log, and browser history it passes, and
/// > `Referer` leaks it to the next site. Turn it off unless a client genuinely
/// > cannot set a header.
///
/// Run it with `dart run example/credential_schemes.dart`:
///
/// ```bash
/// curl -s localhost:8080/whoami -H 'authorization: Bearer t-ada'
/// curl -s localhost:8080/whoami -H 'x-api-key: k-robot'
/// curl -s localhost:8080/whoami -u ada:secret
/// curl -s localhost:8080/whoami -H 'cookie: session=s-ada'
/// curl -s 'localhost:8080/whoami?api_key=k-robot'   # 401, keys in URLs leak
/// curl -i localhost:8080/whoami                      # 401 + challenge
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/whoami', get(whoAmI));

/// `GET /whoami`
Future<Map<String, Object?>> whoAmI(Request request) async {
  final caller = await request.extract(
    firstOf([
      const _Scheme('bearer', BearerTokenExtractable()),
      // Header only. A key in the query string ends up in logs and history.
      const _Scheme('api-key', ApiKeyExtractable(allowQuery: false)),
      const _Basic(),
      const _Scheme('session', SessionIdExtractable()),
    ]),
  );

  return {'via': caller.via, 'id': caller.id};
}

/// Who is calling, and which format said so.
final class Caller {
  /// Creates a [Caller].
  const Caller(this.via, this.id);

  /// Which scheme carried the credential.
  final String via;

  /// Who it names.
  final String id;
}

/// Which credentials exist, standing in for a database.
const _known = {
  't-ada': 'ada',
  'k-robot': 'robot',
  's-ada': 'ada',
};

/// Looks a secret up, and reports which scheme found it.
final class _Scheme implements FromRequestParts<Caller> {
  const _Scheme(this.via, this.inner);

  final String via;
  final FromRequestParts<String> inner;

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    switch (await inner.extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final secret):
        final id = _known[secret];
        return id == null
            ? const Err(Rejection.unauthorized('unknown credential'))
            : Ok(Caller(via, id));
    }
  }
}

/// HTTP Basic, where the secret is a password rather than a token.
final class _Basic implements FromRequestParts<Caller> {
  const _Basic();

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    switch (await const BasicCredentialsExtractable().extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(value: final credentials):
        // A real application stores a password hash and verifies it. Comparing
        // a plaintext password, as here, is demo-grade and nothing else.
        return credentials.password == 'secret'
            ? Ok(Caller('basic', credentials.username))
            : const Err(Rejection.unauthorized('wrong password'));
    }
  }
}
