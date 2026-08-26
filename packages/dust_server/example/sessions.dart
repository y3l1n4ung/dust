import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:dust_server/server.dart';

/// Signed sessions over the cookie extractor.
///
/// Sessions are not in the runtime because every part of one is a policy
/// decision: what a session holds, how long it lasts, where it is stored,
/// whether it can be revoked. The runtime reads the cookie and stops.
///
/// This is the **stateless** shape: the session travels in the cookie, signed so
/// it cannot be edited. Cheap, needs no store, survives a restart, works across
/// isolates — and cannot be revoked before it expires, which is the trade. A
/// server-side store is the other shape: revocable, and now you have a store to
/// run.
///
/// > **Signed is not encrypted.** The payload here is base64, which anyone can
/// > read. The signature stops it being *changed* — a user cannot promote
/// > themselves to `role=admin` — but do not put anything private in it.
///
/// Four things this gets right, and each is a real bug when it is missing:
///
/// * **Constant-time comparison.** `==` on the signature stops at the first
///   wrong byte, and the timing difference lets an attacker forge one byte at a
///   time.
/// * **An expiry inside the signed payload.** A cookie's `Max-Age` is a hint to
///   the browser; a client can keep sending an expired cookie forever. The
///   expiry has to be signed, and checked here.
/// * **`HttpOnly`, `Secure`, `SameSite`.** Without the first, any script reads
///   the session. Without the second it crosses plain HTTP. Without the third it
///   rides along on cross-site requests, which is CSRF.
/// * **A secret that is not in the source.** This one comes from the
///   environment, and a missing one is a startup failure rather than a default.
///
/// Run it with `dart run example/sessions.dart`:
///
/// ```bash
/// curl -si -X POST localhost:8080/sign-in -d 'user=ada'
/// curl -s  localhost:8080/me -H 'cookie: session=<the value above>'
/// curl -si localhost:8080/me                       # 401
/// curl -si localhost:8080/me -H 'cookie: session=forged.signature'
/// ```
Future<void> main() async {
  // A missing secret is a failure, not a default. A default secret in a shipped
  // binary is the same as no signature at all.
  final secret = Platform.environment['SESSION_SECRET'];
  if (secret == null || secret.length < 32) {
    stderr.writeln('set SESSION_SECRET to at least 32 characters');
    exitCode = 78;
    return;
  }

  final server = await serve(
    buildApp(secret: secret),
    InternetAddress.anyIPv4,
    8080,
  );
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({required String secret}) {
  return Router()
    ..route('/sign-in', post(signIn))
    ..route('/sign-out', post(signOut))
    ..route('/me', get(me))
    ..withState(Sessions(secret));
}

/// `POST /sign-in`
Future<Response> signIn(Request request) async {
  final sessions = await request.state<Sessions>();
  final form = await request.form();

  final user = switch (form.field<String>('user')) {
    Ok(:final value) => value,
    Err(:final error) => throw error,
  };

  return Response.ok(
    '{"signedIn":true}',
    headers: {
      'content-type': 'application/json',
      'set-cookie': sessions.cookieFor(user),
    },
  );
}

/// `POST /sign-out` — the same cookie, expired.
Future<Response> signOut(Request request) async {
  final sessions = await request.state<Sessions>();

  return Response.ok(
    '{"signedIn":false}',
    headers: {
      'content-type': 'application/json',
      'set-cookie': sessions.expiredCookie(),
    },
  );
}

/// `GET /me`
Future<Map<String, Object?>> me(Request request) async {
  final sessions = await request.state<Sessions>();
  final user = await request.extract(SessionUser(sessions));

  return {'user': user};
}

/// Reads the signed session cookie, or refuses.
final class SessionUser implements FromRequestParts<String> {
  /// Verifies against [sessions].
  const SessionUser(this.sessions);

  /// The signer.
  final Sessions sessions;

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    // The class, so an absent cookie is an Err this can answer for rather than
    // a throw that escapes composition.
    switch (await const SessionIdExtractable().extract(request)) {
      case Err():
        return const Err(Rejection.unauthorized('sign in first'));
      case Ok(value: final cookie):
        final user = sessions.verify(cookie);
        return user == null
            ? const Err(Rejection.unauthorized('session is not valid'))
            : Ok(user);
    }
  }
}

/// Signs and verifies session cookies.
final class Sessions {
  /// Signs with [secret], for [lifetime].
  Sessions(this.secret, {this.lifetime = const Duration(days: 7)});

  /// The signing key. Never logged, never sent.
  final String secret;

  /// How long a session stays valid.
  final Duration lifetime;

  /// A `Set-Cookie` value carrying a signed session for [user].
  String cookieFor(String user) {
    final expires = DateTime.now().toUtc().add(lifetime);
    final payload = base64Url.encode(
      utf8.encode(jsonEncode({'u': user, 'e': expires.toIso8601String()})),
    );

    return _cookie('$payload.${_sign(payload)}', maxAge: lifetime);
  }

  /// A `Set-Cookie` value that clears it.
  ///
  /// HTTP has no delete: you send the same cookie with `Max-Age=0`, and the
  /// attributes have to match or the browser keeps the original.
  String expiredCookie() => _cookie('', maxAge: Duration.zero);

  /// The user a cookie names, or `null` when it is forged or expired.
  String? verify(String cookie) {
    final parts = cookie.split('.');
    if (parts.length != 2) return null;

    if (!_constantTimeEquals(parts[1], _sign(parts[0]))) return null;

    try {
      final claims = jsonDecode(utf8.decode(base64Url.decode(parts[0])))
          as Map<String, Object?>;
      final expires = DateTime.parse(claims['e']! as String);

      // Checked here, not left to the browser. `Max-Age` is a hint; a client can
      // keep sending an expired cookie for as long as it likes.
      if (expires.isBefore(DateTime.now().toUtc())) return null;

      return claims['u']! as String;
    } on Object {
      // A signature that verified but a payload that will not parse means the
      // format changed, not that a client misbehaved. Either way: no session.
      return null;
    }
  }

  String _sign(String payload) => base64Url.encode(
        Hmac(sha256, utf8.encode(secret)).convert(utf8.encode(payload)).bytes,
      );

  String _cookie(String value, {required Duration maxAge}) => [
        'session=$value',
        'Path=/',
        'Max-Age=${maxAge.inSeconds}',
        'SameSite=Lax',
        'HttpOnly',
        'Secure',
      ].join('; ');

  /// Compares without leaking where the first difference is.
  static bool _constantTimeEquals(String a, String b) {
    if (a.length != b.length) return false;

    var difference = 0;
    for (var index = 0; index < a.length; index++) {
      difference |= a.codeUnitAt(index) ^ b.codeUnitAt(index);
    }
    return difference == 0;
  }
}
