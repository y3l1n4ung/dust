import 'chat_message.dart';

enum ChatStatus { idle, sending, error }

class ChatState {
  const ChatState({
    this.messages = const [],
    this.status = ChatStatus.idle,
    this.errorMessage,
  });

  final List<ChatMessage> messages;
  final ChatStatus status;
  final String? errorMessage;

  ChatState copyWith({
    List<ChatMessage>? messages,
    ChatStatus? status,
    String? errorMessage,
  }) {
    return ChatState(
      messages: messages ?? this.messages,
      status: status ?? this.status,
      errorMessage: errorMessage,
    );
  }
}
