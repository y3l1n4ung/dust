# WebSockets

An upgrade is a `GET`, so it registers like any other route and lives on the
same router as the HTTP API.

```dart
final app = Router()
  ..route('/chat/{room}', ws(joinRoom))
  ..route('/health', get(health))
  ..withState(hub);

Future<void> joinRoom(WebSocketSession session) async {
  final room = session.pathParameters['room']!;
  await for (final message in session.textMessages) {
    hub.broadcast(room, message);
  }
}
```

The handshake is `shelf_web_socket`'s; this package adds the session and the
route.

## The session

`WebSocketSession` keeps the `Request` that upgraded, so everything about it is
still reachable after the handshake:

| Member | Purpose |
| :--- | :--- |
| `pathParameters` | what the route captured |
| `request` | the upgrading request, for extractors such as `StateExtractable` |
| `messages` | everything, `String` or `Uint8List` |
| `textMessages` / `binaryMessages` | one kind only |
| `send` / `sendBytes` | write to the peer |
| `close([code, reason])` | end it |
| `done`, `closeCode`, `closeReason` | how it ended |
| `protocol` | the subprotocol agreed at handshake |

## Close codes

RFC 6455 reserves everything under 3000 for the protocol, and
`web_socket_channel` enforces it: an application may send 1000 or 3000-4999 and
nothing else. That rules out 1011, the code that would otherwise mean "the
server broke", so this package defines its own:

| Constant | Code | Meaning |
| :--- | :--- | :--- |
| `WebSocketClose.normal` | 1000 | the conversation finished |
| `WebSocketClose.handlerFailed` | 4500 | the handler threw |
| `WebSocketClose.goingAway` | 4501 | the server is shutting down |

A handler that throws closes with `handlerFailed` and reports to `onError`,
rather than leaving the socket open.

## What a plain GET sees

A request to a WebSocket path that is not an upgrade is answered by
`shelf_web_socket` with **404**, not 400. The route exists; the handshake did
not.

## Trying it

```bash
dart run example/chat_server.dart
```

The upgrade and the page it belongs to come from one route table:

```bash
# the rendered page
curl -s localhost:8081/rooms/general

# the same room over JSON
curl -s localhost:8081/api/rooms
curl -s localhost:8081/api/rooms/general/messages

# the upgrade handshake, by hand
curl -i --http1.1 \
  -H 'connection: Upgrade' -H 'upgrade: websocket' \
  -H 'sec-websocket-version: 13' \
  -H 'sec-websocket-key: dGhlIHNhbXBsZSBub25jZQ==' \
  localhost:8081/ws/general
```

The last one answers `101 Switching Protocols`. For an actual conversation use
a WebSocket client — `websocat ws://localhost:8081/ws/general?as=ada` — and
watch the message appear in the rendered page.
