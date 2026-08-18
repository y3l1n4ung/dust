import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Streaming events to a browser.
///
/// Server-sent events are one-way and plain HTTP, which makes them the right
/// choice more often than WebSockets: a progress bar, a notification badge, a
/// build log. The browser reconnects on its own, and there is no framing to get
/// wrong.
///
/// The response opts out of the adapter's output buffering, and that is what
/// makes it a stream at all. `shelf` buffers a streamed body by default and
/// flushes when it ends — so every event arrived at once, and a stream that
/// never ends delivered nothing. Measured at the socket, five events emitted
/// 100ms apart all landed together at 536ms.
///
/// Three headers make it work, and `eventStream` sets all three:
///
/// * `content-type: text/event-stream` — what makes `EventSource` accept it.
/// * `cache-control: no-cache` — a cached stream is a stream that never updates.
/// * `x-accel-buffering: no` — tells nginx not to buffer. Without it the stream
///   sits invisible until the buffer fills, which looks exactly like a hung
///   server.
///
/// The keep-alive comment matters for the same reason: an idle stream looks dead
/// to a proxy, and a proxy that decides so closes it.
///
/// **`id` is what makes a reconnect resumable.** The browser sends the last id
/// back as `Last-Event-ID`, and a server that ignores it drops whatever happened
/// while the client was away.
///
/// Run it with `dart run example/sse.dart`:
///
/// ```bash
/// curl -N localhost:8080/ticks
/// curl -N localhost:8080/progress
/// curl -N localhost:8080/ticks -H 'last-event-id: 2'   # resumes from 3
/// ```
///
/// `-N` turns off curl's own buffering; without it you wait for the whole
/// stream.
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/ticks', get(ticks))
    ..route('/progress', get(progress));
}

/// `GET /ticks` — five numbered events, resuming from `Last-Event-ID`.
Future<Response> ticks(Request request) async {
  final resumeFrom = int.tryParse(await request.header('last-event-id') ?? '');
  final start = (resumeFrom ?? 0) + 1;

  return eventStream(
    Stream.fromIterable([
      for (var index = start; index < start + 5; index++)
        ServerSentEvent.json({'tick': index}, id: '$index'),
    ]),
    // Nothing to keep alive on a stream that ends immediately.
    keepAlive: null,
  );
}

/// `GET /progress` — named events, so the client can switch on the name.
///
/// A default `message` event is what `onmessage` receives; a named one needs
/// `addEventListener('step', ...)`. Naming them is what lets one stream carry
/// more than one kind of thing.
Response progress(Request request) {
  return eventStream(
    Stream.fromIterable([
      const ServerSentEvent(event: 'step', data: 'fetching'),
      const ServerSentEvent(event: 'step', data: 'building'),
      const ServerSentEvent(event: 'done', data: 'ok'),
    ]),
    keepAlive: null,
  );
}
