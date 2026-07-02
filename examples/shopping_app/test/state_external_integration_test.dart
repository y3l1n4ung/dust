import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/integrations/services/external_integration_clients.dart';
import 'package:shopping_app/features/integrations/view_models/external_integration_view_model.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/features/support/models/chat_socket.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('state scope loads platform message through MethodChannel', (
    tester,
  ) async {
    const channel = MethodChannel('dust.shopping/integrations');
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    messenger.setMockMethodCallHandler(channel, (call) async {
      expect(call.method, 'message');
      return 'Channel ready';
    });

    await tester.pumpWidget(
      MaterialApp(
        home: ExternalIntegrationViewModelScope(
          args: (_) => ExternalIntegrationViewModelArgs(
            productsClient: const _UnusedProductsClient(),
            platformClient: const MethodChannelExternalPlatformClient(channel),
            chatSocketFactory: const _UnusedChatSocketFactory(),
          ),
          create: (_, args) => ExternalIntegrationViewModel(args),
          child: Builder(
            builder: (context) {
              final message = context
                  .watchExternalIntegrationViewModel()
                  .value
                  .platformMessage;
              return Text(message ?? 'pending');
            },
          ),
        ),
      ),
    );

    await tester.pump();
    await _contextFrom(tester)
        .readExternalIntegrationViewModel()
        .loadPlatformMessage();
    await tester.pump();

    expect(find.text('Channel ready'), findsOneWidget);
  });
}

BuildContext _contextFrom(WidgetTester tester) {
  return tester.element(find.text('pending'));
}

final class _UnusedProductsClient implements ExternalProductsClient {
  const _UnusedProductsClient();

  @override
  Future<List<Product>> fetchProducts() {
    throw UnimplementedError();
  }
}

final class _UnusedChatSocketFactory implements ExternalChatSocketFactory {
  const _UnusedChatSocketFactory();

  @override
  Future<ShoppingChatSocket> open() {
    throw UnimplementedError();
  }
}
