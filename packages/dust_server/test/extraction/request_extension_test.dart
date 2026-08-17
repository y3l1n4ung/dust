import 'package:dust_dart/derive.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// The extension is the hand-written face of extraction: it reads the value and
/// throws the rejection, where the extractor classes return a `Result` for a
/// generator to inspect. Both have to agree, or the short spelling is a
/// different API wearing the same name.

final class _Repo {
  const _Repo(this.name);

  final String name;
}

final class _Payload implements Validatable {
  const _Payload(this.title);

  static _Payload deserialize(Map<String, Object?> json) =>
      _Payload(json['title']! as String);

  final String title;

  @override
  ValidationResult validate() => title.isEmpty
      ? const Invalid([ValidationError(field: 'title', message: 'required')])
      : const Valid();

  @override
  void validateOrThrow() {}
}

final class _Auth implements FromRequestParts<String> {
  const _Auth();

  @override
  Future<Result<String, Rejection>> extract(Request request) async {
    final token = RequestParts.of(request).headers['authorization'];
    return token == null
        ? const Err(Rejection.unauthorized('missing token'))
        : Ok(token);
  }
}

void main() {
  Request build({
    String path = '/',
    Map<String, String> headers = const {},
    Map<String, String> pathParameters = const {},
    Object? body,
  }) {
    return request(
      'POST',
      path,
      headers: headers,
      pathParameters: pathParameters,
      body: body,
      context: {stateKeyFor<_Repo>(): const _Repo('store')},
    );
  }

  group('reading a value', () {
    test('takes a path parameter', () async {
      expect(
        await build(pathParameters: const {'id': '7'}).path<String>('id'),
        '7',
      );
    });

    test('coerces a path parameter to the type asked for', () async {
      expect(await build(pathParameters: const {'id': '7'}).path<int>('id'), 7);
    });

    test('takes a query value', () async {
      expect(await build(path: '/?limit=20').query<int>('limit'), 20);
    });

    test('takes every value of a repeated query key', () async {
      expect(
        await build(path: '/?tag=a&tag=b').queryList<String>('tag'),
        ['a', 'b'],
      );
    });

    test('takes state by type', () async {
      expect((await build().state<_Repo>()).name, 'store');
    });

    test('takes a header', () async {
      expect(
        await build(headers: const {'x-trace': 't'}).header('X-Trace'),
        't',
      );
    });

    test('takes a JSON body', () async {
      final payload = await jsonRequest('POST', '/', '{"title":"buy milk"}')
          .body(_Payload.deserialize);

      expect(payload.title, 'buy milk');
    });

    test('takes a JSON array body', () async {
      final payloads =
          await jsonRequest('POST', '/', '[{"title":"a"},{"title":"b"}]')
              .bodyList(_Payload.deserialize);

      expect([for (final p in payloads) p.title], ['a', 'b']);
    });

    test('takes a text body', () async {
      expect(await build(body: 'hello').textBody(), 'hello');
    });

    test('takes a form body', () async {
      final form = await build(
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
        body: 'title=buy+milk',
      ).form();

      expect(form.fields['title'], 'buy milk');
    });

    test('takes every header at once', () async {
      final all = await build(headers: const {'x-trace': 't'}).headerMap();

      expect(all, containsPair('x-trace', 't'));
    });

    test('takes every query pair at once', () async {
      expect(await build(path: '/?a=1&b=2').queries(), {'a': '1', 'b': '2'});
    });

    test('takes the raw query string', () async {
      expect(await build(path: '/?a=b%20c').rawQuery(), 'a=b%20c');
    });

    test('takes the body as bytes', () async {
      expect(await build(body: 'hi').rawBody(), [104, 105]);
    });

    test('takes the body as an unread stream', () async {
      final stream = await build(body: 'hi').bodyStream();

      expect(await stream.expand((chunk) => chunk).toList(), [104, 105]);
    });

    test('takes a multipart body', () async {
      const boundary = 'X';
      final form = await build(
        headers: const {
          'content-type': 'multipart/form-data; boundary=$boundary',
        },
        body: '--$boundary\r\n'
            'content-disposition: form-data; name="title"\r\n'
            '\r\n'
            'buy milk\r\n'
            '--$boundary--\r\n',
      ).multipart();

      expect(expectOk(form.field<String>('title')), 'buy milk');
    });

    test('takes the connection information', () async {
      final info = await request(
        'GET',
        '/',
        context: const {
          PeerExtractable.contextKey: PeerInfo(
            remoteAddress: '10.0.0.1',
            remotePort: 4242,
            localPort: 8080,
          ),
        },
      ).peer();

      expect(info.remoteAddress, '10.0.0.1');
    });

    test('runs an extractor of its own', () async {
      expect(
        await build(headers: const {'authorization': 'token'})
            .extract(const _Auth()),
        'token',
      );
    });
  });

  group('validBody', () {
    test('returns a value that satisfies its constraints', () async {
      final payload = await jsonRequest('POST', '/', '{"title":"ok"}')
          .validBody(_Payload.deserialize);

      expect(payload.title, 'ok');
    });

    test('throws a 422 naming the field that failed', () async {
      try {
        await jsonRequest('POST', '/', '{"title":""}')
            .validBody(_Payload.deserialize);
        fail('expected a rejection');
      } on Rejection catch (rejection) {
        expect(rejection.status, 422);
        expect(rejection.fields['title'], ['required']);
      }
    });
  });

  group('a failure', () {
    test('throws the rejection instead of returning it', () async {
      expect(
        () => build().query<int>('missing'),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 400)),
      );
    });

    test('throws what a custom extractor rejected with', () async {
      expect(
        () => build().extract(const _Auth()),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 401)),
      );
    });

    test('is caught by the verb builder and becomes the response', () async {
      final app = Router()
        ..route('/{id}', get((request) async {
          final id = await request.path<int>('id');
          return {'id': id};
        }));

      final response =
          await app.handler(Request('GET', Uri.parse('http://x/abc')));

      expect(response.statusCode, 400);
    });
  });

  group('agreement with the extractor classes', () {
    test('reads the same value the class does', () async {
      final direct = await const PathExtractable<String>('id')
          .extract(build(pathParameters: const {'id': '7'}));

      expect(
        await build(pathParameters: const {'id': '7'}).path<String>('id'),
        expectOk(direct),
      );
    });

    test('throws the rejection the class returns', () async {
      final direct =
          await const QueryExtractable<int>('missing').extract(build());

      expect(
        () => build().query<int>('missing'),
        throwsA(
          isA<Rejection>().having(
            (r) => r.message,
            'message',
            expectErr(direct).message,
          ),
        ),
      );
    });
  });
}
