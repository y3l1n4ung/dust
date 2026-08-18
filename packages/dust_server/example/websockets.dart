import 'dart:io';

import 'package:dust_server/server.dart';

/// A WebSocket on the same router as the HTTP routes.
///
/// `ws(handler)` is a `MethodRouter` like `get` or `post`, so an upgrade route
/// sits in the same table as everything else and gets the same layers.
///
/// The handler runs for the life of the connection. Returning from it does not
/// close the socket — `session.done` is how you wait, and `session.close()` is
/// how you end it.
///
/// An upgrade succeeds by **throwing** `HijackException` — the one place here
/// where a throw is not a failure. Layers have to know: tracing and the access
/// log both record it as **101**. Treated as an error instead, a working chat
/// server reads as 100% failure on the endpoint that works.
///
/// > **Every message is untrusted input, and the connection is long-lived.**
/// > Two things follow. Authenticate at the upgrade, where you still have
/// > headers and can answer 401; after that there is no status code to send. And
/// > check the `Origin` header, because the same-origin policy does **not**
/// > apply to WebSockets — any page on the internet may open one to your server
/// > carrying the user's cookies. That is cross-site WebSocket hijacking, and
/// > the check below is the whole defence.
///
/// Run it with `dart run example/websockets.dart`, then use any WebSocket
/// client:
///
/// ```bash
/// # The upgrade is refused without a credential, while it is still HTTP
/// curl -si localhost:8080/echo \
///   -H 'connection: Upgrade' -H 'upgrade: websocket' \
///   -H 'sec-websocket-version: 13' -H 'sec-websocket-key: dGhlIHNhbXBsZQ=='
///
/// # And refused for a foreign origin
/// curl -si 'localhost:8080/echo?token=t-ada' \
///   -H 'origin: https://evil.example' \
///   -H 'connection: Upgrade' -H 'upgrade: websocket' \
///   -H 'sec-websocket-version: 13' -H 'sec-websocket-key: dGhlIHNhbXBsZQ=='
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Origins allowed to open a socket.
const allowedOrigins = {'http://localhost:3000'};

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..routeLayer(guardUpgrade)
    ..route('/echo', ws(echo))
    ..route('/greeter', ws(greeter, protocols: ['greeting.v2', 'greeting.v1']));
}

/// Echoes text back, and stops when the client goes away.
Future<void> echo(WebSocketSession session) async {
  await for (final message in session.textMessages) {
    session.send('echo: $message');
  }
}

/// Answers according to the subprotocol that was negotiated.
///
/// `protocols` is offered in preference order and the client picks; the result is
/// `session.protocol`. It is how one endpoint serves two client versions without
/// a second URL.
Future<void> greeter(WebSocketSession session) async {
  session.send(switch (session.protocol) {
    'greeting.v2' => '{"hello":true}',
    _ => 'hello',
  });
  await session.close();
}

/// Refuses an upgrade that is unauthenticated or cross-origin.
///
/// A `routeLayer`, so it runs before the upgrade while a status code can still
/// be sent, and only for routes that exist.
Handler guardUpgrade(Handler inner) {
  return (Request request) async {
    final origin = request.headers['origin'];
    // A non-browser client sends no Origin, which is why absent is allowed here
    // and a *wrong* one is not. If only browsers should connect, require it.
    if (origin != null && !allowedOrigins.contains(origin)) {
      return const Rejection.forbidden('origin not allowed').intoResponse();
    }

    // The token rides in the query because a browser cannot set headers on a
    // WebSocket handshake. It therefore lands in access logs — use a short-lived
    // ticket fetched over HTTPS, not the session token.
    if (request.url.queryParameters['token'] != 't-ada') {
      return const Rejection.unauthorized('a ticket is required')
          .intoResponse();
    }

    return inner(request);
  };
}
