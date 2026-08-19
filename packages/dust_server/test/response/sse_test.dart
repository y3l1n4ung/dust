import 'dart:async';
import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// `text/event-stream` is line-oriented, so every framing mistake is silent on
/// the server and shows up as a browser that receives half an event or nothing
/// at all. These pin the framing rather than the plumbing.

void main() {
  Future<String> read(Response response) async =>
      utf8.decode(await response.read().expand((chunk) => chunk).toList());

  group('encoding one event', () {
    test('writes data and ends with a blank line', () {
      expect(
        const ServerSentEvent(data: 'hello').encode(),
        'data:hello\n\n',
      );
    });

    test('names an event when it was given one', () {
      expect(
        const ServerSentEvent(data: 'x', event: 'tick').encode(),
        'event:tick\ndata:x\n\n',
      );
    });

    test('carries an id a browser can resume from', () {
      expect(
        const ServerSentEvent(data: 'x', id: '42').encode(),
        'id:42\ndata:x\n\n',
      );
    });

    test('writes the retry hint in milliseconds', () {
      expect(
        const ServerSentEvent(retry: Duration(seconds: 3)).encode(),
        'retry:3000\n\n',
      );
    });

    test('splits multi-line data into one field per line', () {
      // A raw newline would end the event early and the second line would be
      // parsed as a new field.
      expect(
        const ServerSentEvent(data: 'first\nsecond').encode(),
        'data:first\ndata:second\n\n',
      );
    });

    test('keeps an empty data line rather than dropping it', () {
      expect(const ServerSentEvent(data: '').encode(), 'data:\n\n');
    });

    test('writes a comment, which listeners ignore', () {
      expect(
        const ServerSentEvent(comment: 'keep-alive').encode(),
        ':keep-alive\n\n',
      );
    });

    test('splits a multi-line comment too', () {
      expect(
        const ServerSentEvent(comment: 'a\nb').encode(),
        ':a\n:b\n\n',
      );
    });

    test('strips a newline out of an event name', () {
      // The name is one line by definition; a newline would forge a field.
      expect(
        const ServerSentEvent(data: 'x', event: 'tick\ndata:forged').encode(),
        'event:tickdata:forged\ndata:x\n\n',
      );
    });

    test('strips a carriage return out of an id', () {
      expect(
        const ServerSentEvent(data: 'x', id: '1\r\n2').encode(),
        'id:12\ndata:x\n\n',
      );
    });

    test('writes the fields in the order a parser expects', () {
      final encoded = const ServerSentEvent(
        comment: 'c',
        event: 'e',
        id: 'i',
        retry: Duration(milliseconds: 500),
        data: 'd',
      ).encode();

      expect(encoded, ':c\nevent:e\nid:i\nretry:500\ndata:d\n\n');
    });

    test('sends nothing but the terminator when it carries nothing', () {
      expect(const ServerSentEvent().encode(), '\n');
    });

    test('encodes JSON for the common case', () {
      expect(
        ServerSentEvent.json({'a': 1}, event: 'update').encode(),
        'event:update\ndata:{"a":1}\n\n',
      );
    });

    test('describes itself by event name and data', () {
      expect(
        const ServerSentEvent(data: 'x', event: 'tick').toString(),
        'ServerSentEvent(tick, x)',
      );
      expect(
        const ServerSentEvent(data: 'x').toString(),
        'ServerSentEvent(message, x)',
      );
    });
  });

  group('the response', () {
    Response stream(List<ServerSentEvent> events) =>
        eventStream(Stream.fromIterable(events), keepAlive: null);

    test('is text/event-stream with a charset', () {
      expect(
        stream(const []).headers['content-type'],
        'text/event-stream; charset=utf-8',
      );
    });

    test('refuses caching, so a proxy cannot share one stream', () {
      expect(stream(const []).headers['cache-control'], 'no-cache');
    });

    test('tells nginx not to buffer, which would hide the stream', () {
      expect(stream(const []).headers['x-accel-buffering'], 'no');
    });

    test('writes each event in order', () async {
      final response = stream(const [
        ServerSentEvent(data: 'one'),
        ServerSentEvent(data: 'two'),
      ]);

      expect(await read(response), 'data:one\n\ndata:two\n\n');
    });

    test('ends when the stream does', () async {
      expect(await read(stream(const [])), isEmpty);
    });

    test('can answer with a status of its own', () {
      expect(
        eventStream(const Stream.empty(), status: 201, keepAlive: null)
            .statusCode,
        201,
      );
    });
  });

  group('keep-alive', () {
    test('slips a comment into an idle stream', () async {
      // Nothing is emitted for long enough that an intermediary might give up.
      final response = eventStream(
        Stream.periodic(const Duration(milliseconds: 120)).take(1).map(
              (_) => const ServerSentEvent(data: 'late'),
            ),
        keepAlive: const Duration(milliseconds: 30),
      );

      final body = await read(response);

      expect(body, contains(':keep-alive'));
      expect(body, contains('data:late'));
    });

    test('sends none when it was turned off', () async {
      final response = eventStream(
        Stream.periodic(const Duration(milliseconds: 120)).take(1).map(
              (_) => const ServerSentEvent(data: 'late'),
            ),
        keepAlive: null,
      );

      expect(await read(response), 'data:late\n\n');
    });

    test('stops once the stream is done', () async {
      final response = eventStream(
        Stream.fromIterable(const [ServerSentEvent(data: 'x')]),
        keepAlive: const Duration(milliseconds: 20),
      );

      final body = await read(response);
      await Future<void>.delayed(const Duration(milliseconds: 80));

      // Reading finished, so no keep-alive can have been appended after.
      expect(body, 'data:x\n\n');
    });

    test('carries an error from the source through', () async {
      final response = eventStream(
        Stream<ServerSentEvent>.error(StateError('boom')),
        keepAlive: const Duration(milliseconds: 20),
      );

      expect(read(response), throwsA(isA<StateError>()));
    });

    test('stops its timer when the client goes away', () async {
      final response = eventStream(
        Stream.periodic(
          const Duration(milliseconds: 200),
          (_) => const ServerSentEvent(data: 'x'),
        ),
        keepAlive: const Duration(milliseconds: 20),
      );

      final subscription = response.read().listen(null);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      await subscription.cancel();

      // Nothing to assert but the absence of a pending timer keeping the test
      // isolate alive; cancelling must not throw either.
      expect(true, isTrue);
    });
  });

  group('from a handler', () {
    test('is returned like any other value', () async {
      final app = Router()
        ..route('/events', get((request) async {
          return eventStream(
            Stream.fromIterable(const [ServerSentEvent(data: 'tick')]),
            keepAlive: null,
          );
        }));

      final response = await app.handler(request('GET', '/events'));

      expect(response.headers['content-type'], startsWith('text/event-stream'));
      expect(await read(response), 'data:tick\n\n');
    });
  });
}
