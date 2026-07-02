import 'package:dust_dart/serde.dart';

part 'chat_message.g.dart';

/// Chat role values for the shopping app example.
@Derive([Serialize(), Deserialize()])
enum ChatRole {
  /// User chat role.
  user,

  /// Assistant chat role.
  assistant,
}

/// Chat message model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatMessage with _$ChatMessage {
  /// Creates a [ChatMessage].
  const ChatMessage({
    required this.id,
    required this.role,
    required this.text,
    required this.createdAt,
  });

  /// Unique identifier.
  final String id;

  /// Role.
  final ChatRole role;

  /// Text.
  final String text;

  /// Created at.
  final DateTime createdAt;

  /// Creates a [ChatMessage] from JSON.
  factory ChatMessage.fromJson(Map<String, Object?> json) =>
      _$ChatMessageFromJson(json);
}

/// Chat request model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatRequest with _$ChatRequest {
  /// Creates a [ChatRequest].
  const ChatRequest({required this.message, required this.history});

  /// Message.
  final String message;

  /// History.
  final List<ChatMessage> history;

  /// Creates a [ChatRequest] from JSON.
  factory ChatRequest.fromJson(Map<String, Object?> json) =>
      _$ChatRequestFromJson(json);
}

/// Chat response model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatResponse with _$ChatResponse {
  /// Creates a [ChatResponse].
  const ChatResponse({required this.message, required this.escalated});

  /// Message.
  final ChatMessage message;

  /// Escalated.
  final bool escalated;

  /// Creates a [ChatResponse] from JSON.
  factory ChatResponse.fromJson(Map<String, Object?> json) =>
      _$ChatResponseFromJson(json);
}
