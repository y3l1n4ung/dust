import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';

import 'support.dart';

/// The streaming export.
///
/// A CSV is unbounded by nature, so it is written out as it is produced. The
/// test that matters reads a raw socket and checks the bytes arrive before the
/// work has finished — no status code can show that.

void main() {
  Future<({TestApp app, String token})> shopWith(int orders) async {
    final app = await testApp();
    await app.createAccount('ada@example.com', 'correct horse battery');
    final token = await app.signIn('ada@example.com', 'correct horse battery');

    for (var index = 0; index < orders; index++) {
      await app.send('POST', '/orders',
          body: {'item': 'item $index', 'quantity': 1}, token: token);
    }

    return (app: app, token: token);
  }

  group('GET /exports/orders.csv', () {
    test('has a header and one row per order', () async {
      final shop = await shopWith(3);

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);
      final lines = const LineSplitter().convert(response.body);

      expect(response.statusCode, 200);
      expect(lines.first, 'id,item,quantity,placed_at');
      expect(lines, hasLength(4));
    });

    test('names the download rather than letting the URL decide', () async {
      final shop = await shopWith(1);

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);

      expect(response.headers['content-type'], startsWith('text/csv'));
      expect(
        response.headers['content-disposition'],
        'attachment; filename="orders.csv"',
      );
    });

    test('sets no content-length, because the length is not known yet',
        () async {
      final shop = await shopWith(2);

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);

      expect(response.headers['content-length'], isNull);
    });

    test('quotes an item containing a comma', () async {
      // Unquoted, one value would become two columns.
      final shop = await shopWith(0);
      await shop.app.send('POST', '/orders',
          body: const {'item': 'shirt, blue', 'quantity': 1},
          token: shop.token);

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);

      expect(response.body, contains('"shirt, blue"'));
    });

    test('doubles a quote inside an item, so the row stays valid', () async {
      final shop = await shopWith(0);
      await shop.app.send('POST', '/orders',
          body: const {'item': 'the "good" one', 'quantity': 1},
          token: shop.token);

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);

      expect(response.body, contains('"the ""good"" one"'));
    });

    test('exports only this account rows', () async {
      final shop = await shopWith(2);
      await shop.app
          .createAccount('bob@example.com', 'a different long password');
      final bob =
          await shop.app.signIn('bob@example.com', 'a different long password');

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: bob);

      expect(const LineSplitter().convert(response.body), hasLength(1));
    });

    test('pages through more rows than one page holds', () async {
      // The page size is 100; this crosses it, so the loop has to run twice.
      final shop = await shopWith(0);
      for (var index = 0; index < 120; index++) {
        await shop.app.send('POST', '/orders',
            body: {'item': 'item $index', 'quantity': 1}, token: shop.token);
      }

      final response =
          await shop.app.send('GET', '/exports/orders.csv', token: shop.token);

      expect(const LineSplitter().convert(response.body), hasLength(121));
    });

    test('is sent chunked, not buffered and measured first', () async {
      // The observable difference. A buffered response carries a
      // content-length, which means the whole export was assembled before a
      // byte went out; a streamed one is chunked.
      //
      // Counting arrivals would be the more direct test and is not reliable
      // here: 300 rows are produced with no delay between them, so the kernel
      // coalesces them into one segment and a correct stream looks identical to
      // a buffered one. The SSE tests can count arrivals because their source
      // is genuinely slow.
      final shop = await shopWith(0);
      for (var index = 0; index < 200; index++) {
        await shop.app.send('POST', '/orders',
            body: {'item': 'item $index', 'quantity': 1}, token: shop.token);
      }

      final socket = await Socket.connect(
        shop.app.server.address.host,
        shop.app.server.port,
      );
      socket.write('GET /exports/orders.csv HTTP/1.1\r\n'
          'Host: localhost\r\n'
          'authorization: Bearer ${shop.token}\r\n'
          'accept-encoding: identity\r\n'
          'Connection: close\r\n\r\n');
      await socket.flush();

      final received = StringBuffer();
      await for (final chunk in socket) {
        received.write(utf8.decode(chunk, allowMalformed: true));
      }
      await socket.close();

      final headers = received.toString().split('\r\n\r\n').first;
      expect(headers, contains('transfer-encoding: chunked'));
      expect(headers, isNot(contains('content-length')));
    });

    test('no token is 401', () async {
      final app = await testApp();

      expect((await app.send('GET', '/exports/orders.csv')).statusCode, 401);
    });
  });
}
