/// WebSockets, served by the same router as HTTP.
///
/// ```dart
/// import 'package:dust_server/ws.dart';
///
/// final app = Router()..route('/chat/{room}', ws(joinRoom));
///
/// Future<void> joinRoom(WebSocketSession session) async {
///   await for (final message in session.textMessages) {
///     session.send(message);
///   }
/// }
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'package:web_socket_channel/web_socket_channel.dart'
    show WebSocketChannel;

export 'src/ws/ws.dart';
