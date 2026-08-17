import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('AccessLog', () {
    test('records method, path, and status', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(AccessLog(records.add))
        ..route('/a/{id}', get((request) async => textResponse('ok')));

      await app.handler(request('GET', '/a/7'));

      expect(records.single.method, 'GET');
      expect(records.single.path, '/a/7');
      expect(records.single.status, 200);
    });

    test('records a failure as readily as a success', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(AccessLog(records.add))
        ..route('/a', get((request) async => noContent()));

      await app.handler(request('DELETE', '/a'));

      expect(records.single.status, 405);
    });

    test('records requests nothing matched', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(AccessLog(records.add))
        ..route('/a', get((request) async => noContent()));

      await app.handler(request('GET', '/nope'));

      expect(records.single.status, 404);
    });

    test('measures the handler', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(AccessLog(records.add))
        ..route('/slow', get((request) async {
          await Future<void>.delayed(const Duration(milliseconds: 20));
          return noContent();
        }));

      await app.handler(request('GET', '/slow'));

      expect(records.single.duration.inMilliseconds, greaterThanOrEqualTo(15));
    });

    test('picks up the request id when that layer ran first', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(const RequestId())
        ..layer(AccessLog(records.add))
        ..route('/a', get((request) async => noContent()));

      await app.handler(request('GET', '/a', headers: {'x-request-id': 'abc'}));

      expect(records.single.requestId, 'abc');
    });

    test('leaves the id null without that layer', () async {
      final records = <AccessRecord>[];
      final app = Router()
        ..layer(AccessLog(records.add))
        ..route('/a', get((request) async => noContent()));

      await app.handler(request('GET', '/a'));

      expect(records.single.requestId, isNull);
    });

    test('describes a record readably', () {
      const record = AccessRecord(
        method: 'GET',
        path: '/a',
        status: 200,
        duration: Duration(milliseconds: 12),
        requestId: 'abc',
      );

      expect(record.toString(), 'GET /a 200 12ms [abc]');
    });
  });
}
