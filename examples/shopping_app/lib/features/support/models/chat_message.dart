import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'chat_message.g.dart';

@Derive([Serialize(), Deserialize()])
enum ChatRole { user, assistant }

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatMessage with _$ChatMessage {
  const ChatMessage({
    required this.id,
    required this.role,
    required this.text,
    required this.createdAt,
  });

  final String id;
  final ChatRole role;
  final String text;
  final DateTime createdAt;

  factory ChatMessage.fromJson(Map<String, Object?> json) =>
      _$ChatMessageFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatRequest with _$ChatRequest {
  const ChatRequest({required this.message, required this.history});

  final String message;
  final List<ChatMessage> history;

  factory ChatRequest.fromJson(Map<String, Object?> json) =>
      _$ChatRequestFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ChatResponse with _$ChatResponse {
  const ChatResponse({required this.message, required this.escalated});

  final ChatMessage message;
  final bool escalated;

  factory ChatResponse.fromJson(Map<String, Object?> json) =>
      _$ChatResponseFromJson(json);
}
