import 'chat_message.dart';

/// Chat status values for the shopping app example.
enum ChatStatus {
  /// Idle chat status.
  idle,

  /// Sending chat status.
  sending,

  /// Error chat status.
  error,
}

/// Chat state for the shopping app example.
class ChatState {
  /// Creates a [ChatState].
  const ChatState({
    this.messages = const [],
    this.status = ChatStatus.idle,
    this.errorMessage,
  });

  /// Messages.
  final List<ChatMessage> messages;

  /// Status.
  final ChatStatus status;

  /// Error message.
  final String? errorMessage;

  /// Copy with chat status.
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
