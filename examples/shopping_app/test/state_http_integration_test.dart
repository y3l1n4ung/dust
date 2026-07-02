import 'dart:async';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/api/shopping_api.dart';
import 'package:shopping_app/features/integrations/models/external_integration_state.dart';
import 'package:shopping_app/features/integrations/services/external_integration_clients.dart';
import 'package:shopping_app/features/integrations/view_models/external_integration_view_model.dart';
import 'package:shopping_app/features/support/models/chat_socket.dart';

void main() {
  test('invalidated ViewModel ignores late Dust HTTP results', () async {
    final requestReceived = Completer<void>();
    final dio = Dio();
    dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) async {
          if (!requestReceived.isCompleted) requestReceived.complete();
          await Future<void>.delayed(const Duration(milliseconds: 20));
          handler.resolve(
            Response<dynamic>(
              requestOptions: options,
              data: [_productJson(1, 'Dust HTTP Backpack')],
            ),
          );
        },
      ),
    );
    final viewModel = ExternalIntegrationViewModel(
      ExternalIntegrationViewModelArgs(
        productsClient: DustHttpExternalProductsClient(ShoppingApi(dio)),
        platformClient: const _UnusedPlatformClient(),
        chatSocketFactory: const _UnusedChatSocketFactory(),
      ),
    );
    addTearDown(viewModel.dispose);

    final load = viewModel.loadProducts();
    await requestReceived.future.timeout(const Duration(seconds: 5));
    expect(viewModel.state.status, ExternalIntegrationStatus.loading);

    viewModel.invalidateSelf();
    await load;

    expect(viewModel.state.status, ExternalIntegrationStatus.initial);
    expect(viewModel.state.productTitles, isEmpty);
  });
}

final class _UnusedPlatformClient implements ExternalPlatformClient {
  const _UnusedPlatformClient();

  @override
  Future<String> loadMessage() {
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

Map<String, Object?> _productJson(int id, String title) {
  return {
    'id': id,
    'title': title,
    'price': 42.0,
    'description': 'A product loaded through Dust HTTP.',
    'category': 'bags',
    'image': 'https://example.com/product.png',
    'rating': {'rate': 4.8, 'count': 12},
  };
}
