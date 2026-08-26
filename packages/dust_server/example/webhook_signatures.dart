import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:dust_server/server.dart';

/// Verifying a signed webhook.
///
/// A webhook endpoint is a public URL that performs privileged work — marking an
/// invoice paid, provisioning an account. The signature is the only thing
/// separating the real sender from anyone who found the URL.
///
/// Three mistakes, each of which silently defeats the check:
///
/// * **Verifying the re-serialized body.** Decode the JSON, encode it again, and
///   the bytes differ — key order, spacing, number formatting. The signature was
///   computed over what was *sent*, so verify `rawBody()` and parse afterwards.
/// * **Comparing with `==`.** It stops at the first wrong byte, and the timing
///   difference lets an attacker discover the signature one byte at a time.
/// * **Not checking the timestamp.** Without it a captured request can be
///   replayed forever. Signing the timestamp alongside the body is what makes
///   the window enforceable — a bare timestamp header can just be edited.
///
/// The 401 says nothing useful, deliberately. "Signature mismatch" versus
/// "timestamp too old" tells an attacker which half to work on.
///
/// Run it with `dart run example/webhook_signatures.dart`:
///
/// ```bash
/// BODY='{"event":"invoice.paid","id":"in_123"}'
/// TS=$(date +%s)
/// SIG=$(printf '%s.%s' "$TS" "$BODY" \
///   | openssl dgst -sha256 -hmac 'shh' -binary | base64)
///
/// curl -si -X POST localhost:8080/hooks -d "$BODY" \
///   -H "x-timestamp: $TS" -H "x-signature: $SIG"     # 200
/// curl -si -X POST localhost:8080/hooks -d "$BODY" \
///   -H "x-timestamp: $TS" -H 'x-signature: wrong'    # 401
/// ```
Future<void> main() async {
  final secret = Platform.environment['WEBHOOK_SECRET'];
  if (secret == null || secret.isEmpty) {
    stderr.writeln('set WEBHOOK_SECRET');
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
Router buildApp({required String secret, Duration? tolerance}) {
  return Router()
    ..route('/hooks', post(receive))
    ..withState(
      WebhookVerifier(
        secret,
        tolerance: tolerance ?? const Duration(minutes: 5),
      ),
    );
}

/// `POST /hooks`
Future<Result<Map<String, Object?>, Rejection>> receive(Request request) async {
  final verifier = await request.state<WebhookVerifier>();

  // The raw bytes, before anything parses them. Verifying a re-encoded body
  // compares a signature against something the sender never signed.
  final raw = await request.rawBody();

  if (!verifier.accepts(
    raw,
    signature: request.headers['x-signature'],
    timestamp: request.headers['x-timestamp'],
  )) {
    // Deliberately uninformative: which half failed is not the sender's
    // business, and telling an attacker narrows their search.
    return const Err(Rejection.unauthorized('invalid signature'));
  }

  final payload = jsonDecode(utf8.decode(raw)) as Map<String, Object?>;
  return Ok({'received': payload['event']});
}

/// Checks a webhook signature and its age.
final class WebhookVerifier {
  /// Verifies against [secret], accepting a timestamp within [tolerance].
  WebhookVerifier(this.secret,
      {required this.tolerance, DateTime Function()? now})
      : _now = now ?? DateTime.now;

  /// The shared secret.
  final String secret;

  /// How far out of date a request may be before it is treated as a replay.
  final Duration tolerance;

  final DateTime Function() _now;

  /// Whether [body] carries a valid, recent signature.
  bool accepts(List<int> body, {String? signature, String? timestamp}) {
    if (signature == null || timestamp == null) return false;

    final sent = int.tryParse(timestamp);
    if (sent == null) return false;

    final age = _now().difference(
      DateTime.fromMillisecondsSinceEpoch(sent * 1000),
    );
    // Both directions: a clock ahead of ours is as suspicious as one behind.
    if (age.abs() > tolerance) return false;

    // The timestamp is signed along with the body, so it cannot be edited to
    // widen the window.
    final signed = <int>[...utf8.encode('$timestamp.'), ...body];
    final expected =
        base64.encode(Hmac(sha256, utf8.encode(secret)).convert(signed).bytes);

    return _constantTimeEquals(signature, expected);
  }

  /// Compares without stopping at the first difference.
  static bool _constantTimeEquals(String a, String b) {
    if (a.length != b.length) return false;

    var difference = 0;
    for (var index = 0; index < a.length; index++) {
      difference |= a.codeUnitAt(index) ^ b.codeUnitAt(index);
    }
    return difference == 0;
  }
}
