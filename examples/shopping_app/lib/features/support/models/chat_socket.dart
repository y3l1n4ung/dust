import 'dart:async';

import 'chat_message.dart';

/// Shopping chat socket.
abstract interface class ShoppingChatSocket {
  /// Responses.
  Stream<ChatResponse> get responses;

  /// Sends.
  void send(ChatRequest request);

  /// Closes.
  Future<void> close();
}
