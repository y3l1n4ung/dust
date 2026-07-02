import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/integrations/services/external_integration_clients.dart';
import 'package:shopping_app/features/integrations/view_models/external_integration_view_model.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/features/support/models/chat_message.dart';

void main() {
  test('ViewModel uses WebSocket factory and closes socket on dispose',
      () async {
    final server = await _ChatWebSocketServer.start();
    addTearDown(server.close);
    final viewModel = ExternalIntegrationViewModel(
      ExternalIntegrationViewModelArgs(
        productsClient: const _UnusedProductsClient(),
        platformClient: const _UnusedPlatformClient(),
        chatSocketFactory: WebSocketExternalChatSocketFactory(server.uri),
      ),
    );
    addTearDown(viewModel.dispose);

    final responseObserved = Completer<void>();
    viewModel.addListener(() {
      if (viewModel.state.socketReply == 'websocket:coupon' &&
          !responseObserved.isCompleted) {
        responseObserved.complete();
      }
    });

    await viewModel.sendChat('coupon');
    await server.waitForRequest();
    await server.waitForResponse();
    await responseObserved.future.timeout(const Duration(seconds: 5));

    expect(viewModel.state.socketReply, 'websocket:coupon');

    viewModel.dispose();
    await server.waitForClose();
  });
}

final class _UnusedProductsClient implements ExternalProductsClient {
  const _UnusedProductsClient();

  @override
  Future<List<Product>> fetchProducts() {
    throw UnimplementedError();
  }
}

final class _UnusedPlatformClient implements ExternalPlatformClient {
  const _UnusedPlatformClient();

  @override
  Future<String> loadMessage() {
    throw UnimplementedError();
  }
}

final class _ChatWebSocketServer {
  _ChatWebSocketServer(
    this._server,
    this._requestReceived,
    this._responseSent,
    this._socketClosed,
  );

  final HttpServer _server;
  final Completer<void> _requestReceived;
  final Completer<void> _responseSent;
  final Completer<void> _socketClosed;

  Uri get uri => Uri.parse(
        'ws://${_server.address.address}:${_server.port}/support',
      );

  static Future<_ChatWebSocketServer> start() async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final requestReceived = Completer<void>();
    final responseSent = Completer<void>();
    final socketClosed = Completer<void>();
    server.transform(WebSocketTransformer()).listen((socket) {
      void completeClose() {
        if (!socketClosed.isCompleted) socketClosed.complete();
      }

      socket.done.then((_) => completeClose());
      socket.listen(
        (data) {
          if (!requestReceived.isCompleted) requestReceived.complete();
          final request = ChatRequest.fromJson(
            Map<String, Object?>.from(jsonDecode(data as String) as Map),
          );
          final response = ChatResponse(
            message: ChatMessage(
              id: 'ws-${request.message}',
              role: ChatRole.assistant,
              text: 'websocket:${request.message}',
              createdAt: DateTime(2026, 1, 1),
            ),
            escalated: false,
          );
          socket.add(jsonEncode(response.toJson()));
          if (!responseSent.isCompleted) responseSent.complete();
        },
        onDone: completeClose,
      );
    });
    return _ChatWebSocketServer(
      server,
      requestReceived,
      responseSent,
      socketClosed,
    );
  }

  Future<void> waitForRequest() =>
      _requestReceived.future.timeout(const Duration(seconds: 5));

  Future<void> waitForResponse() =>
      _responseSent.future.timeout(const Duration(seconds: 5));

  Future<void> waitForClose() =>
      _socketClosed.future.timeout(const Duration(seconds: 5));

  Future<void> close() => _server.close(force: true);
}
