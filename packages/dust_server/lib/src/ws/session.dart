import 'dart:async';
import 'dart:typed_data';

import 'package:shelf/shelf.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import '../request/request_parts.dart';

/// Close codes this package uses.
///
/// RFC 6455 reserves everything below 3000 for the protocol itself, and
/// `web_socket_channel` enforces that: an application may only send 1000 or a
/// code in 3000-4999. That rules out 1011, the code that would otherwise say
/// "the server broke", so a failure is reported as [handlerFailed].
abstract final class WebSocketClose {
  /// The conversation finished normally.
  static const normal = 1000;

  /// The handler threw, so the connection could not continue.
  static const handlerFailed = 4500;

  /// The server is shutting down.
  static const goingAway = 4501;
}

/// One accepted WebSocket connection.
///
/// The upgrade already happened, so everything a handler needs about the
/// request that opened it, path parameters and state included, is still here.
final class WebSocketSession {
  /// Wraps [channel], opened by [request].
  WebSocketSession(this.channel, this.request);

  /// The channel underneath, for anything this class does not wrap.
  final WebSocketChannel channel;

  /// The request that upgraded, kept so extractors still work.
  final Request request;

  /// Path parameters captured by the route that accepted the upgrade.
  Map<String, String> get pathParameters => pathParametersOf(request);

  /// The subprotocol agreed during the handshake, if any.
  String? get protocol => channel.protocol;

  /// Everything the peer sends, as `String` or `Uint8List`.
  Stream<Object?> get messages => channel.stream;

  /// Text messages only, which is what most applications want.
  Stream<String> get textMessages =>
      messages.where((message) => message is String).cast<String>();

  /// Binary messages only.
  Stream<Uint8List> get binaryMessages => messages
      .where((message) => message is List<int>)
      .map((message) => Uint8List.fromList(message! as List<int>));

  /// Sends [message] to the peer.
  void send(String message) => channel.sink.add(message);

  /// Sends bytes to the peer.
  void sendBytes(List<int> bytes) => channel.sink.add(bytes);

  /// Closes the connection.
  ///
  /// [code] has to be 1000 or in the application range 3000-4999; see
  /// [WebSocketClose] for the ones this package sends.
  Future<void> close([int code = WebSocketClose.normal, String? reason]) =>
      channel.sink.close(code, reason);

  /// Resolves when the connection is finished, however it ended.
  Future<void> get done => channel.sink.done;

  /// The close code the peer sent, once [done] completes.
  int? get closeCode => channel.closeCode;

  /// The close reason the peer sent, once [done] completes.
  String? get closeReason => channel.closeReason;
}
