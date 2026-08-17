import 'dart:async';

import 'package:shelf_web_socket/shelf_web_socket.dart';

import '../response/error_reporting.dart';
import '../router/method_router.dart';
import 'session.dart';

/// What a WebSocket route runs once the connection is open.
typedef WebSocketHandler = FutureOr<void> Function(WebSocketSession session);

/// Serves a WebSocket upgrade at this path.
///
/// The handshake is a `GET`, so this registers one, and the same router serves
/// it beside ordinary routes:
///
/// ```dart
/// final app = Router()
///   ..route('/chat/{room}', ws(joinRoom))
///   ..route('/health', get(health));
/// ```
///
/// A handler that throws closes the connection with
/// [WebSocketClose.handlerFailed] rather than leaving it open, and the error
/// reaches the router's `onError`.
MethodRouter ws(
  WebSocketHandler handler, {
  Iterable<String>? protocols,
  Duration? pingInterval,
}) {
  return get(
    (request) {
      final upgrade = webSocketHandler(
        (channel, protocol) async {
          final session = WebSocketSession(
            channel,
            request,
            negotiatedProtocol: protocol,
          );
          try {
            await handler(session);
          } on Object catch (error, stack) {
            ServerErrors.report(error, stack);
            await channel.sink
                .close(WebSocketClose.handlerFailed, 'handler failed');
          }
        },
        protocols: protocols,
        pingInterval: pingInterval,
      );
      return upgrade(request);
    },
  );
}
