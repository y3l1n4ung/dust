import 'dart:io';

import 'package:dust_server/server.dart';

/// Finding out who is calling, and why the obvious answer is wrong.
///
/// `peer()` gives the address of the socket. Behind a load balancer, a CDN, or
/// an ingress controller that is **the proxy's** address, and every request
/// looks like it came from the same client — which breaks rate limiting, audit
/// logs, and geo rules all at once.
///
/// The real client address is in `X-Forwarded-For`. That header is a list, and
/// it is appended to by each hop:
///
/// ```http
/// X-Forwarded-For: 203.0.113.7, 70.41.3.18, 150.172.238.178
/// ```
///
/// > **Every entry a client could have written is a lie.** Anyone may send
/// > `X-Forwarded-For: 1.2.3.4`, and a proxy *appends* rather than replaces — so
/// > the leftmost entry is whatever the client claimed. Trusting it hands an
/// > attacker the ability to spoof any address: to evade an IP ban, to poison an
/// > audit log, or to frame someone else's address for abuse.
///
/// The rule: count hops **from the right**, and trust exactly as many as you
/// have proxies. With one proxy in front, the client is the last entry. With
/// none, ignore the header entirely — an unproxied server has no reason to
/// believe it at all.
///
/// Run it with `dart run example/client_ip.dart`:
///
/// ```bash
/// curl -s localhost:8080/whoami
/// curl -s localhost:8080/whoami -H 'x-forwarded-for: 1.2.3.4'
/// curl -s localhost:8080/whoami -H 'x-forwarded-for: 9.9.9.9, 203.0.113.7'
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
///
/// [trustedProxies] is how many hops sit in front. **Zero** is the default,
/// because a server that is not behind a proxy must not believe the header, and
/// a default that trusts one is a spoofing hole for everyone who never
/// configured it.
Router buildApp({int trustedProxies = 0}) {
  return Router()
    ..route('/whoami', get(whoAmI))
    ..withState(ClientAddress(trustedProxies: trustedProxies));
}

/// `GET /whoami`
Future<Map<String, Object?>> whoAmI(Request request) async {
  final resolver = await request.state<ClientAddress>();
  final peer = await request.peer();

  return {
    'client': resolver.of(request, peer),
    'socket': peer.remoteAddress,
  };
}

/// Works out the client address for a known number of proxies.
final class ClientAddress {
  /// Trusts [trustedProxies] hops in front of this server.
  const ClientAddress({required this.trustedProxies});

  /// How many proxies sit between the client and here.
  final int trustedProxies;

  /// The client address, as far as it can be established.
  String of(Request request, PeerInfo peer) {
    if (trustedProxies < 1) return peer.remoteAddress;

    final forwarded = request.headers['x-forwarded-for'];
    if (forwarded == null) return peer.remoteAddress;

    final hops = forwarded.split(',').map((entry) => entry.trim()).toList();

    // Counted from the right. The rightmost entry was written by the proxy
    // nearest us and is the only one we can vouch for; each step left is one
    // more hop we are choosing to believe. Anything beyond the proxies we
    // actually have is client-written text.
    final index = hops.length - trustedProxies;
    return index >= 0 && index < hops.length ? hops[index] : peer.remoteAddress;
  }
}
