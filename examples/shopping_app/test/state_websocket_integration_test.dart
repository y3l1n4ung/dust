import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/integrations/services/external_integration_clients.dart';
import 'package:shopping_app/features/integrations/view_models/external_integration_view_model.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/features/support/models/chat_message.dart';
import 'package:shopping_app/features/support/models/chat_socket.dart';

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

  test('concurrent sends share one WebSocket open', () async {
    final factory = _DeferredChatSocketFactory();
    final viewModel = _viewModelWith(factory);
    addTearDown(viewModel.dispose);

    final first = viewModel.sendChat('one');
    final second = viewModel.sendChat('two');

    expect(factory.openCalls, 1);

    final socket = _FakeChatSocket();
    factory.complete(socket);
    await Future.wait([first, second]);

    expect(socket.sent.map((request) => request.message), ['one', 'two']);
  });

  test('socket opened after dispose is closed without listening', () async {
    final factory = _DeferredChatSocketFactory();
    final viewModel = _viewModelWith(factory);

    final connect = viewModel.connectChat();
    expect(factory.openCalls, 1);

    viewModel.dispose();
    final socket = _FakeChatSocket();
    factory.complete(socket);
    await connect;

    expect(socket.listenCalls, 0);
    expect(socket.closeCalls, 1);
  });
}

ExternalIntegrationViewModel _viewModelWith(
  ExternalChatSocketFactory chatSocketFactory,
) {
  return ExternalIntegrationViewModel(
    ExternalIntegrationViewModelArgs(
      productsClient: const _UnusedProductsClient(),
      platformClient: const _UnusedPlatformClient(),
      chatSocketFactory: chatSocketFactory,
    ),
  );
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

final class _DeferredChatSocketFactory implements ExternalChatSocketFactory {
  final List<Completer<ShoppingChatSocket>> _pending = [];

  int get openCalls => _pending.length;

  @override
  Future<ShoppingChatSocket> open() {
    final completer = Completer<ShoppingChatSocket>();
    _pending.add(completer);
    return completer.future;
  }

  void complete(ShoppingChatSocket socket) {
    _pending.last.complete(socket);
  }
}

final class _FakeChatSocket implements ShoppingChatSocket {
  final StreamController<ChatResponse> _responses =
      StreamController<ChatResponse>.broadcast();

  final List<ChatRequest> sent = [];
  var closeCalls = 0;
  var listenCalls = 0;

  @override
  Stream<ChatResponse> get responses {
    listenCalls += 1;
    return _responses.stream;
  }

  @override
  void send(ChatRequest request) {
    sent.add(request);
  }

  @override
  Future<void> close() async {
    closeCalls += 1;
    await _responses.close();
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
