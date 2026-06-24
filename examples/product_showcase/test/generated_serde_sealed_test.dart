import 'package:test/test.dart';

import 'package:product_showcase/product_showcase.dart';

void main() {
  test(
    'sealed serde sample keeps variant metadata and concrete JSON working',
    () {
      final event = JsonPaymentEvent.success(
        id: 'pay-1',
        cents: 4200,
        currency: 'USD',
      );

      expect(event, isA<JsonPaymentSuccess>());
      final success = event as JsonPaymentSuccess;
      expect(success.toJson(), {
        'id': 'pay-1',
        'cents': 4200,
        'currency': 'USD',
      });

      final decoded = JsonPaymentSuccess.fromJson({
        'id': 'pay-1',
        'cents': 4200,
        'currency': 'USD',
      });
      expect(decoded.id, success.id);
      expect(decoded.cents, success.cents);
      expect(decoded.currency, success.currency);

      final failed = JsonPaymentEvent.failed(
        id: 'pay-2',
        reason: 'insufficient_funds',
        retryable: true,
      );
      expect(failed, isA<JsonPaymentFailed>());
      expect((failed as JsonPaymentFailed).toJson(), {
        'id': 'pay-2',
        'reason': 'insufficient_funds',
        'retryable': true,
      });
    },
  );
}
