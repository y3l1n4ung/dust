import 'dart:io';

import 'package:dust_server/server.dart';

/// Serving over TLS.
///
/// `serve` takes a `SecurityContext`, so terminating TLS in the Dart
/// process is two extra lines. Whether you should is a separate question.
///
/// **Most deployments should not.** A reverse proxy or load balancer in front —
/// nginx, Caddy, an ALB, an ingress controller — already terminates TLS, renews
/// certificates, and staples OCSP. Doing it here as well means two places to
/// renew and one of them will lapse. Terminate in the process when there is
/// genuinely nothing in front: a sidecar-free container, an appliance, mutual TLS
/// where the handshake identity has to reach your code.
///
/// > **`Strict-Transport-Security` belongs with a real certificate, not this
/// > example.** Sent from a host whose certificate later lapses, it locks users
/// > out — the browser refuses to let them through. Turn it on once renewal is
/// > automated and proven.
///
/// Redirecting HTTP to HTTPS needs a second listener on port 80, because a
/// client speaking plain HTTP to the TLS port gets a handshake failure rather
/// than a redirect. The redirect uses **308**, so a `POST` is not silently turned
/// into a `GET`.
///
/// Run it with a self-signed pair, which browsers will warn about:
///
/// ```bash
/// openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
///   -subj '/CN=localhost' -keyout key.pem -out cert.pem
/// dart run example/tls.dart cert.pem key.pem
/// ```
///
/// ```bash
/// curl -sk https://localhost:8443/health   # -k accepts the self-signed cert
/// curl -si http://localhost:8080/health    # 308 to https
/// ```
Future<void> main(List<String> arguments) async {
  if (arguments.length < 2) {
    stderr.writeln('usage: dart run example/tls.dart <cert.pem> <key.pem>');
    exitCode = 64;
    return;
  }

  final context = SecurityContext()
    ..useCertificateChain(arguments[0])
    ..usePrivateKey(arguments[1]);

  final https = await serve(
    buildApp(),
    InternetAddress.anyIPv4,
    8443,
    securityContext: context,
  );
  final http = await serve(
    buildRedirectApp(port: 8443),
    InternetAddress.anyIPv4,
    8080,
  );
  stdout.writeln('https on ${https.port}, redirecting http on ${http.port}');

  await ProcessSignal.sigint.watch().first;
  await Future.wait([
    https.close(drain: const Duration(seconds: 15)),
    http.close(drain: const Duration(seconds: 5)),
  ]);
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    // No HSTS here: see the note above. A deployment with automated renewal
    // sets `strictTransportSecurity` and means it.
    ..layer(const SecurityHeaders())
    ..route('/health', get(health));
}

/// A second application whose only job is to send clients to HTTPS.
Router buildRedirectApp({required int port}) {
  return Router()..fallback((request) => toHttps(request, port).intoResponse());
}

/// `GET /health`
Map<String, Object?> health(Request request) => const {'status': 'ok'};

/// The HTTPS form of [request]'s URL.
///
/// The host comes from the `Host` header, which is client input — fine here,
/// because the value only goes back to the client that sent it, and a forged one
/// redirects that client to its own choice of host and nobody else. Anywhere the
/// value is stored or emailed, compare it against a configured list first.
Redirect toHttps(Request request, int port) {
  final host = (request.headers['host'] ?? 'localhost').split(':').first;
  final target = request.requestedUri.replace(
    scheme: 'https',
    host: host,
    port: port == 443 ? null : port,
  );

  // 308, so a POST stays a POST rather than losing its body.
  return Redirect.permanent(target.toString());
}
