import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';

import '../../../core/api/shopping_api.dart';
import '../../products/models/product.dart';
import '../../support/models/chat_message.dart';
import '../../support/models/chat_socket.dart';

/// Product client used by the external integration ViewModel fixture.
abstract interface class ExternalProductsClient {
  /// Loads products from an external source.
  Future<List<Product>> fetchProducts();
}

/// Platform client used by the external integration ViewModel fixture.
abstract interface class ExternalPlatformClient {
  /// Loads a platform-provided message.
  Future<String> loadMessage();
}

/// Chat socket factory used by the external integration ViewModel fixture.
abstract interface class ExternalChatSocketFactory {
  /// Opens a chat socket.
  Future<ShoppingChatSocket> open();
}

/// HTTP implementation for [ExternalProductsClient].
final class DustHttpExternalProductsClient implements ExternalProductsClient {
  /// Creates a [DustHttpExternalProductsClient].
  const DustHttpExternalProductsClient(this.api);

  /// Injected generated Dust HTTP client.
  final ShoppingApi api;

  @override
  Future<List<Product>> fetchProducts() => api.getProducts();
}

/// Method-channel implementation for [ExternalPlatformClient].
final class MethodChannelExternalPlatformClient
    implements ExternalPlatformClient {
  /// Creates a [MethodChannelExternalPlatformClient].
  const MethodChannelExternalPlatformClient(this.channel);

  /// Injected Flutter method channel.
  final MethodChannel channel;

  @override
  Future<String> loadMessage() async {
    return await channel.invokeMethod<String>('message') ?? '';
  }
}

/// WebSocket implementation for [ExternalChatSocketFactory].
final class WebSocketExternalChatSocketFactory
    implements ExternalChatSocketFactory {
  /// Creates a [WebSocketExternalChatSocketFactory].
  const WebSocketExternalChatSocketFactory(this.uri);

  /// WebSocket endpoint URI.
  final Uri uri;

  @override
  Future<ShoppingChatSocket> open() async {
    final socket = await WebSocket.connect(uri.toString());
    return WebSocketShoppingChatSocket(socket);
  }
}

/// WebSocket-backed shopping chat socket.
final class WebSocketShoppingChatSocket implements ShoppingChatSocket {
  /// Creates a [WebSocketShoppingChatSocket].
  const WebSocketShoppingChatSocket(this.socket);

  /// Injected Dart WebSocket.
  final WebSocket socket;

  @override
  Stream<ChatResponse> get responses => socket.map((data) {
        return ChatResponse.fromJson(
          Map<String, Object?>.from(jsonDecode(data as String) as Map),
        );
      });

  @override
  void send(ChatRequest request) {
    socket.add(jsonEncode(request.toJson()));
  }

  @override
  Future<void> close() =>
      socket.close(WebSocketStatus.normalClosure, 'disposed');
}
