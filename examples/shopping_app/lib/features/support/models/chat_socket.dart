import 'dart:async';

import 'chat_message.dart';

abstract interface class ShoppingChatSocket {
  Stream<ChatResponse> get responses;
  void send(ChatRequest request);
  Future<void> close();
}
