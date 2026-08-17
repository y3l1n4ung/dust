import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Which redirect a handler returns is a question about method and caching,
/// and both failure modes are quiet: a `POST` silently becomes a `GET`, or a
/// browser caches a move that was meant to be temporary and never asks again.

void main() {
  group('the status each one sends', () {
    test('to is 303, which asks for a GET next', () {
      expect(const Redirect.to('/next').intoResponse().statusCode, 303);
    });

    test('temporary is 307', () {
      expect(const Redirect.temporary('/next').intoResponse().statusCode, 307);
    });

    test('permanent is 308', () {
      expect(const Redirect.permanent('/next').intoResponse().statusCode, 308);
    });

    test('found is 302', () {
      expect(const Redirect.found('/next').intoResponse().statusCode, 302);
    });

    test('movedPermanently is 301', () {
      expect(
        const Redirect.movedPermanently('/next').intoResponse().statusCode,
        301,
      );
    });
  });

  group('what each one promises', () {
    test('only 301 and 308 may be cached forever', () {
      expect(
        [
          const Redirect.movedPermanently('/a').isPermanent,
          const Redirect.permanent('/a').isPermanent,
          const Redirect.to('/a').isPermanent,
          const Redirect.temporary('/a').isPermanent,
          const Redirect.found('/a').isPermanent,
        ],
        [true, true, false, false, false],
      );
    });

    test('only 307 and 308 keep the method', () {
      expect(
        [
          const Redirect.temporary('/a').preservesMethod,
          const Redirect.permanent('/a').preservesMethod,
          const Redirect.to('/a').preservesMethod,
          const Redirect.found('/a').preservesMethod,
          const Redirect.movedPermanently('/a').preservesMethod,
        ],
        [true, true, false, false, false],
      );
    });
  });

  group('the Location header', () {
    test('carries where the client is being sent', () {
      final response = const Redirect.to('/next').intoResponse();

      expect(response.headers['location'], '/next');
    });

    test('carries an absolute URL unchanged', () {
      final response = const Redirect.permanent('https://example.test/a?b=1#c')
          .intoResponse();

      expect(response.headers['location'], 'https://example.test/a?b=1#c');
    });

    test('cannot start a second header with a newline', () {
      // A redirect target is exactly the kind of value built from user input.
      final response =
          const Redirect.to('/next\r\nX-Injected: yes').intoResponse();

      expect(response.headers['location'], '/nextX-Injected: yes');
      expect(response.headers, isNot(contains('x-injected')));
    });

    test('drops a bare newline as well', () {
      final response = const Redirect.to('/a\nb').intoResponse();

      expect(response.headers['location'], isNot(contains('\n')));
    });

    test('keeps a query string that merely looks odd', () {
      final response = const Redirect.to('/search?q=a+b&r=%20c').intoResponse();

      expect(response.headers['location'], '/search?q=a+b&r=%20c');
    });
  });

  group('from a handler', () {
    Future<Response> serve(Object? Function(Request) handler) {
      final app = Router()
        ..route('/old', get((request) async => handler(request)));
      return Future.sync(() => app.handler(request('GET', '/old')));
    }

    test('is written as the redirect it is', () async {
      final response = await serve((request) => const Redirect.to('/new'));

      expect(response.statusCode, 303);
      expect(response.headers['location'], '/new');
    });

    test('sends no body, because there is nothing to read', () async {
      final response =
          await serve((request) => const Redirect.permanent('/new'));

      expect(await response.readAsString(), isEmpty);
    });

    test('works through Ok, like any other value', () async {
      final response = await serve(
        (request) => const Ok<Redirect, Rejection>(Redirect.to('/new')),
      );

      expect(response.statusCode, 303);
    });
  });

  group('describing itself', () {
    test('names the status and the target', () {
      expect(
        const Redirect.temporary('/next').toString(),
        'Redirect(307, /next)',
      );
    });
  });
}
