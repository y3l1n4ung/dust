import 'dart:async';
import 'dart:convert';

import 'package:shelf/shelf.dart';

/// One server-sent event.
///
/// The wire format is line-oriented and unforgiving: fields are `name: value`,
/// an event ends at a blank line, and a newline inside a value would end the
/// event early. Multi-line data is therefore sent as one `data:` line per
/// line, which is what the specification says and what browsers parse back
/// into a single string.
final class ServerSentEvent {
  /// Creates an event.
  const ServerSentEvent({
    this.data,
    this.event,
    this.id,
    this.retry,
    this.comment,
  });

  /// The payload, encoded as JSON.
  ///
  /// A convenience for the common case; anything else can use [data].
  factory ServerSentEvent.json(
    Object? value, {
    String? event,
    String? id,
  }) =>
      ServerSentEvent(data: jsonEncode(value), event: event, id: id);

  /// What the event carries.
  final String? data;

  /// The event name a listener can subscribe to by, rather than `message`.
  final String? event;

  /// The id a browser echoes back as `Last-Event-ID` when it reconnects.
  final String? id;

  /// How long a browser should wait before reconnecting.
  final Duration? retry;

  /// A comment line, which listeners ignore.
  ///
  /// The usual reason to send one is as a keep-alive: proxies close idle
  /// connections, and a comment is traffic that no listener has to handle.
  final String? comment;

  /// This event in the `text/event-stream` format, blank line included.
  String encode() {
    final buffer = StringBuffer();

    if (comment case final comment?) {
      for (final line in comment.split('\n')) {
        buffer.writeln(':$line');
      }
    }
    if (event case final event?) buffer.writeln('event:${_oneLine(event)}');
    if (id case final id?) buffer.writeln('id:${_oneLine(id)}');
    if (retry case final retry?) {
      buffer.writeln('retry:${retry.inMilliseconds}');
    }
    if (data case final data?) {
      // A newline inside the value would end the event, so each line is its
      // own `data:` field. The browser rejoins them with `\n`.
      for (final line in data.split('\n')) {
        buffer.writeln('data:$line');
      }
    }

    buffer.writeln();
    return buffer.toString();
  }

  /// Strips what would end a field early.
  static String _oneLine(String value) =>
      value.replaceAll('\r', '').replaceAll('\n', '');

  @override
  String toString() => 'ServerSentEvent(${event ?? 'message'}, $data)';
}

/// Streams [events] to the client as `text/event-stream`.
///
/// Return it from a handler like any other value:
///
/// ```dart
/// Future<Object?> ticks(Request request) async => eventStream(
///       Stream.periodic(
///         const Duration(seconds: 1),
///         (count) => ServerSentEvent.json({'tick': count}),
///       ),
///     );
/// ```
///
/// The connection stays open until the stream closes or the client goes away.
/// Compared with a WebSocket this is one-way and rides on plain HTTP, which is
/// why it survives proxies that mangle upgrades — and why it is usually the
/// cheaper choice for a live feed nobody talks back on.
///
/// The headers matter as much as the body. `Cache-Control: no-cache` stops a
/// proxy serving one client's stream to another, and `X-Accel-Buffering: no`
/// tells nginx not to buffer — without it a stream can sit invisible until the
/// buffer fills, which looks exactly like a server that has hung.
///
/// [keepAlive] sends a comment on an idle stream so an intermediary does not
/// decide the connection is dead. Set it to `null` to send nothing.
Response eventStream(
  Stream<ServerSentEvent> events, {
  Duration? keepAlive = const Duration(seconds: 15),
  int status = 200,
}) {
  final body = keepAlive == null ? events : _withKeepAlive(events, keepAlive);

  return Response(
    status,
    body: body.map((event) => utf8.encode(event.encode())),
    headers: const {
      'content-type': 'text/event-stream; charset=utf-8',
      'cache-control': 'no-cache',
      'x-accel-buffering': 'no',
    },
  );
}

/// Emits [events], slipping in a comment whenever nothing has been sent for
/// [every].
Stream<ServerSentEvent> _withKeepAlive(
  Stream<ServerSentEvent> events,
  Duration every,
) {
  late StreamController<ServerSentEvent> controller;
  StreamSubscription<ServerSentEvent>? subscription;
  Timer? timer;

  void schedule() {
    timer?.cancel();
    timer = Timer(every, () {
      if (!controller.isClosed) {
        controller.add(const ServerSentEvent(comment: 'keep-alive'));
        schedule();
      }
    });
  }

  controller = StreamController<ServerSentEvent>(
    onListen: () {
      schedule();
      subscription = events.listen(
        (event) {
          controller.add(event);
          schedule();
        },
        onError: controller.addError,
        onDone: controller.close,
      );
    },
    onCancel: () async {
      timer?.cancel();
      await subscription?.cancel();
    },
  );

  return controller.stream;
}
