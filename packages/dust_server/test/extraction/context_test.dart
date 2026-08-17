import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('ContextExtractable', () {
    test('reads a value middleware stored', () async {
      final outcome = await const ContextExtractable<String>('tenant')
          .extract(request('GET', '/', context: {'tenant': 'acme'}));

      expect(expectOk(outcome), 'acme');
    });

    test('returns null for an absent optional value', () async {
      final outcome = await const ContextExtractable<String?>('tenant')
          .extract(request('GET', '/'));

      expect(expectOk(outcome), isNull);
    });

    test('reports an absent required value as 500', () async {
      final outcome = await const ContextExtractable<String>('tenant')
          .extract(request('GET', '/'));

      expectStatus(outcome, 500);
    });

    test('reports a wrongly typed value as 500', () async {
      final outcome = await const ContextExtractable<int>('tenant')
          .extract(request('GET', '/', context: {'tenant': 'acme'}));

      expect(expectStatus(outcome, 500).message, contains('is not a int'));
    });
  });

  group('PeerExtractable', () {
    test('reads connection information', () async {
      const info = PeerInfo(
        remoteAddress: '10.0.0.1',
        remotePort: 51234,
        localPort: 8080,
      );
      final outcome = await const PeerExtractable().extract(
        request('GET', '/', context: {PeerExtractable.contextKey: info}),
      );

      expect(expectOk(outcome).remoteAddress, '10.0.0.1');
      expect(expectOk(outcome).toString(), 'PeerInfo(10.0.0.1:51234)');
    });

    test('reports 500 when the adapter supplied nothing', () async {
      final outcome =
          await const PeerExtractable().extract(request('GET', '/'));

      expectStatus(outcome, 500);
    });
  });
}
