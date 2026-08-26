import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

Future<String> up(Router app) async {
  final s = await serve(app, InternetAddress.loopbackIPv4, 0);
  addTearDown(() => s.close(drain: const Duration(seconds: 1)));
  return 'http://127.0.0.1:${s.port}';
}

void main() {
  group('path params', () {
    test('a percent-encoded slash stays one segment', () async {
      final seen = <String>[];
      final app = Router()
        ..route('/u/{name}', get((r) async {
          seen.add(await r.path<String>('name'));
          return 'ok';
        }));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/u/a%2Fb'));
      expect(res.statusCode, 200);
      expect(seen.single, 'a/b');
    });

    test('an encoded dot-segment does not reach another route', () async {
      final app = Router()
        ..route('/api/{id}', get((r) async => 'api'))
        ..route('/admin', get((r) async => 'admin'));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/api/..%2fadmin'));
      expect(res.body, isNot('admin'));
    });
  });

  group('coercion', () {
    test('an int beyond 64 bits is rejected, not wrapped', () async {
      final app = Router()
        ..route('/n', get((r) async => (await r.query<int>('v')).toString()));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/n?v=99999999999999999999'));
      expect(res.statusCode, 400);
    });

    test('whitespace is not trimmed into validity', () async {
      final app = Router()
        ..route('/n', get((r) async => (await r.query<int>('v')).toString()));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/n?v=%205'));
      expect(res.statusCode, 400);
    });

    test('trailing whitespace is rejected too', () async {
      final app = Router()
        ..route('/n', get((r) async => (await r.query<int>('v')).toString()));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/n?v=5%20'));
      expect(res.statusCode, 400);
    });

    test('a leading plus behaves consistently', () async {
      final app = Router()
        ..route('/n', get((r) async => (await r.query<int>('v')).toString()));
      final origin = await up(app);
      final res = await http.get(Uri.parse('$origin/n?v=%2B5'));
      expect([200, 400], contains(res.statusCode));
    });
  });

  group('content type', () {
    test('a charset parameter does not defeat the json match', () async {
      final app = Router()
        ..route('/j', post((r) async {
          final body = await r.body<Map<String, Object?>>((j) => j);
          return body['k'].toString();
        }));
      final origin = await up(app);
      final res = await http.post(Uri.parse('$origin/j'),
          headers: {'content-type': 'application/json; charset=utf-8'},
          body: jsonEncode({'k': 'v'}));
      expect(res.statusCode, 200);
      expect(res.body, 'v');
    });

    test('an uppercase media type still matches', () async {
      final app = Router()
        ..route('/j', post((r) async {
          final body = await r.body<Map<String, Object?>>((j) => j);
          return body['k'].toString();
        }));
      final origin = await up(app);
      final res = await http.post(Uri.parse('$origin/j'),
          headers: {'content-type': 'APPLICATION/JSON'},
          body: jsonEncode({'k': 'v'}));
      expect(res.statusCode, 200);
    });
  });

  group('body', () {
    test('an absent body on a json route is a client error', () async {
      final app = Router()
        ..route('/j', post((r) async {
          await r.body<Map<String, Object?>>((j) => j);
          return 'ok';
        }));
      final origin = await up(app);
      final res = await http.post(Uri.parse('$origin/j'),
          headers: {'content-type': 'application/json'});
      expect(res.statusCode, inInclusiveRange(400, 422));
    });

    test('a json array where an object is expected is not a 500', () async {
      final app = Router()
        ..route('/j', post((r) async {
          await r.body<Map<String, Object?>>((j) => j);
          return 'ok';
        }));
      final origin = await up(app);
      final res = await http.post(Uri.parse('$origin/j'),
          headers: {'content-type': 'application/json'}, body: '[1,2,3]');
      expect(res.statusCode, isNot(500));
    });
  });

  group('responses', () {
    test('CRLF in a header value answers 500 rather than hanging', () async {
      final errors = <Object>[];
      final app = Router(onError: (e, _) => errors.add(e))
        ..route(
            '/h',
            get((r) async => Response.ok('body',
                headers: {'x-echo': 'a\r\nx-injected: yes'})));
      final origin = await up(app);

      final res = await http
          .get(Uri.parse('$origin/h'))
          .timeout(const Duration(seconds: 3));

      expect(res.statusCode, 500);
      expect(res.headers.containsKey('x-injected'), isFalse);
      expect(errors, hasLength(1), reason: 'the log must name the handler');
    });

    test('a bare control character is refused too', () async {
      final app = Router()
        ..route(
            '/h',
            get((r) async =>
                Response.ok('body', headers: {'x-echo': 'a\x00b'})));
      final origin = await up(app);

      final res = await http
          .get(Uri.parse('$origin/h'))
          .timeout(const Duration(seconds: 3));
      expect(res.statusCode, 500);
    });

    test('a tab in a header value is still allowed', () async {
      final app = Router()
        ..route('/h',
            get((r) async => Response.ok('body', headers: {'x-echo': 'a\tb'})));
      final origin = await up(app);

      final res = await http
          .get(Uri.parse('$origin/h'))
          .timeout(const Duration(seconds: 3));
      expect(res.statusCode, 200, reason: 'tab is legal in a header value');
    });
  });

  group('shutdown', () {
    test('a request arriving after close is refused', () async {
      final app = Router()..route('/', get((r) async => 'ok'));
      final s = await serve(app, InternetAddress.loopbackIPv4, 0);
      final origin = 'http://127.0.0.1:${s.port}';

      unawaited(s.close(drain: const Duration(milliseconds: 200)));
      await Future<void>.delayed(const Duration(milliseconds: 100));

      await expectLater(
        http.get(Uri.parse('$origin/')).timeout(const Duration(seconds: 2)),
        throwsA(anything),
      );
    });
  });
}
