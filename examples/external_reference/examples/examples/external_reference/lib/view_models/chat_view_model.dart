import 'dart:async';
import 'dart:math';

import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

part 'chat_view_model.ease.dart';

/// Connection status for WebSocket.
enum ConnectionStatus { disconnected, connecting, connected }

/// A chat message.
@immutable
class ChatMessage {
  final String id;
  final String text;
  final String sender;
  final bool isMe;
  final DateTime timestamp;

  const ChatMessage({
    required this.id,
    required this.text,
    required this.sender,
    required this.isMe,
    required this.timestamp,
  });
}

/// Chat state.
@immutable
class ChatState {
  final List<ChatMessage> messages;
  final bool isTyping;
  final bool hasNewMessage;
  final ConnectionStatus connectionStatus;
  final String? typingUser;

  const ChatState({
    this.messages = const [],
    this.isTyping = false,
    this.hasNewMessage = false,
    this.connectionStatus = ConnectionStatus.disconnected,
    this.typingUser,
  });

  bool get isConnected => connectionStatus == ConnectionStatus.connected;

  ChatState copyWith({
    List<ChatMessage>? messages,
    bool? isTyping,
    bool? hasNewMessage,
    ConnectionStatus? connectionStatus,
    String? typingUser,
    bool clearTypingUser = false,
  }) {
    return ChatState(
      messages: messages ?? this.messages,
      isTyping: isTyping ?? this.isTyping,
      hasNewMessage: hasNewMessage ?? this.hasNewMessage,
      connectionStatus: connectionStatus ?? this.connectionStatus,
      typingUser: clearTypingUser ? null : (typingUser ?? this.typingUser),
    );
  }
}

/// ViewModel for chat demo with WebSocket-like streaming.
@ease
class ChatViewModel extends StateNotifier<ChatState> {
  ChatViewModel() : super(const ChatState());

  final _random = Random();

  // Stream subscription for incoming messages
  StreamSubscription<ChatMessage>? _messageSubscription;
  Timer? _typingTimer;
  Timer? _reconnectTimer;

  // Simulated users in the chat
  static const _users = ['Alice', 'Bob', 'Charlie', 'Diana'];

  // Keyword-based responses for conversation simulation
  static const _conversationResponses = <String, List<String>>{
    'hello': ['Hey there!', 'Hi! How are you?', 'Hello! Nice to meet you!'],
    'hi': ['Hey!', 'Hi there!', 'Hello!'],
    'how are you': [
      'I\'m doing great, thanks!',
      'Pretty good! You?',
      'Fantastic! How about you?'
    ],
    'good': ['That\'s great to hear!', 'Awesome!', 'Nice!'],
    'thanks': ['You\'re welcome!', 'No problem!', 'Anytime!'],
    'help': [
      'Sure, what do you need?',
      'I\'m here to help!',
      'How can I assist you?'
    ],
    'bye': ['Goodbye!', 'See you later!', 'Take care!'],
    'weather': [
      'It\'s sunny here!',
      'A bit cloudy today',
      'Perfect weather for coding!'
    ],
    'food': ['I love pizza!', 'Have you tried sushi?', 'I\'m hungry now 😄'],
    'work': [
      'How\'s your project going?',
      'Busy day?',
      'Keep up the good work!'
    ],
    'flutter': [
      'Flutter is awesome!',
      'Love building apps with Flutter!',
      'Dart is great too!'
    ],
    'code': ['What are you building?', 'Coding is fun!', 'Any bugs today?'],
    'yes': ['Great!', 'Awesome!', 'Perfect!'],
    'no': ['Oh okay', 'I see', 'No worries!'],
    'lol': ['😂', 'Haha!', '🤣'],
    '?': ['Good question!', 'Let me think...', 'Hmm, interesting!'],
  };

  static const _defaultResponses = [
    'That\'s interesting!',
    'Tell me more about that.',
    'I see what you mean.',
    'Good point!',
    'Thanks for sharing!',
    'That makes sense.',
    'Hmm, let me think about that...',
    'I agree!',
    'Interesting perspective!',
    'Oh really?',
  ];

  /// Connect to the chat stream (simulates WebSocket connection).
  Future<void> connect() async {
    if (state.connectionStatus == ConnectionStatus.connecting ||
        state.connectionStatus == ConnectionStatus.connected) {
      return;
    }

    setState(
      state.copyWith(connectionStatus: ConnectionStatus.connecting),
      action: 'ws:connecting',
    );

    // Simulate connection delay
    await Future.delayed(const Duration(milliseconds: 800));

    if (!hasListeners) return;

    setState(
      state.copyWith(connectionStatus: ConnectionStatus.connected),
      action: 'ws:connected',
    );

    // Start listening to the message stream
    _messageSubscription = _createMessageStream().listen(
      _onMessageReceived,
      onError: (error) {
        if (!hasListeners) return;
        _handleDisconnect();
      },
    );
  }

  /// Disconnect from the chat stream.
  void disconnect() {
    _messageSubscription?.cancel();
    _messageSubscription = null;
    _typingTimer?.cancel();
    _reconnectTimer?.cancel();

    setState(
      state.copyWith(
        connectionStatus: ConnectionStatus.disconnected,
        isTyping: false,
        clearTypingUser: true,
      ),
      action: 'ws:disconnected',
    );
  }

  void _handleDisconnect() {
    disconnect();

    // Auto-reconnect after 3 seconds
    _reconnectTimer = Timer(const Duration(seconds: 3), () {
      if (hasListeners) {
        connect();
      }
    });
  }

  // Background chatter messages
  static const _chatMessages = [
    'Hey everyone!',
    'How\'s it going?',
    'Anyone working on something cool?',
    'Just joined the chat',
    'What\'s new?',
    'Hi all!',
    'Good to be here',
    'Any news?',
  ];

  /// Create a mock WebSocket message stream (background chatter).
  Stream<ChatMessage> _createMessageStream() async* {
    while (true) {
      // Random interval between 8-15 seconds (less frequent background chatter)
      await Future.delayed(
        Duration(milliseconds: 8000 + _random.nextInt(7000)),
      );

      // Show typing indicator first
      final typingUser = _users[_random.nextInt(_users.length)];
      if (hasListeners) {
        setState(
          state.copyWith(isTyping: true, typingUser: typingUser),
          action: 'ws:typing',
        );
      }

      // Typing delay
      await Future.delayed(
        Duration(milliseconds: 500 + _random.nextInt(1500)),
      );

      yield ChatMessage(
        id: DateTime.now().millisecondsSinceEpoch.toString(),
        text: _chatMessages[_random.nextInt(_chatMessages.length)],
        sender: typingUser,
        isMe: false,
        timestamp: DateTime.now(),
      );
    }
  }

  void _onMessageReceived(ChatMessage message) {
    if (!hasListeners) return;

    setState(
      state.copyWith(
        messages: [...state.messages, message],
        isTyping: false,
        hasNewMessage: true,
        clearTypingUser: true,
      ),
      action: 'ws:message',
    );

    // Clear the new message flag
    Future.microtask(() {
      if (hasListeners) {
        setState(
          state.copyWith(hasNewMessage: false),
          action: 'ws:clearNewMessage',
        );
      }
    });
  }

  /// Send a message.
  void sendMessage(String text) {
    if (text.trim().isEmpty) return;
    if (!state.isConnected) return;

    final message = ChatMessage(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      text: text.trim(),
      sender: 'Me',
      isMe: true,
      timestamp: DateTime.now(),
    );

    setState(
      state.copyWith(
        messages: [...state.messages, message],
        hasNewMessage: true,
      ),
      action: 'ws:send',
    );

    // Clear the new message flag
    Future.microtask(() {
      if (hasListeners) {
        setState(
          state.copyWith(hasNewMessage: false),
          action: 'ws:clearNewMessage',
        );
      }
    });

    // Simulate conversation reply
    _simulateReply(text.trim());
  }

  /// Generate a contextual reply based on user message.
  void _simulateReply(String userMessage) {
    final lowerMessage = userMessage.toLowerCase();

    // Find matching response
    String? responseText;
    for (final entry in _conversationResponses.entries) {
      if (lowerMessage.contains(entry.key)) {
        final responses = entry.value;
        responseText = responses[_random.nextInt(responses.length)];
        break;
      }
    }

    // Fallback to default response
    responseText ??=
        _defaultResponses[_random.nextInt(_defaultResponses.length)];

    // Pick a random user to reply
    final replyUser = _users[_random.nextInt(_users.length)];

    // Show typing indicator
    _typingTimer?.cancel();
    _typingTimer = Timer(
      Duration(milliseconds: 300 + _random.nextInt(500)),
      () {
        if (!hasListeners || !state.isConnected) return;

        setState(
          state.copyWith(isTyping: true, typingUser: replyUser),
          action: 'ws:typing',
        );

        // Send the reply after typing delay
        _typingTimer = Timer(
          Duration(milliseconds: 800 + _random.nextInt(1200)),
          () {
            if (!hasListeners || !state.isConnected) return;

            final reply = ChatMessage(
              id: DateTime.now().millisecondsSinceEpoch.toString(),
              text: responseText!,
              sender: replyUser,
              isMe: false,
              timestamp: DateTime.now(),
            );

            setState(
              state.copyWith(
                messages: [...state.messages, reply],
                isTyping: false,
                hasNewMessage: true,
                clearTypingUser: true,
              ),
              action: 'ws:reply',
            );

            // Clear the new message flag
            Future.microtask(() {
              if (hasListeners) {
                setState(
                  state.copyWith(hasNewMessage: false),
                  action: 'ws:clearNewMessage',
                );
              }
            });
          },
        );
      },
    );
  }

  /// Clear all messages.
  void clearMessages() {
    setState(
      state.copyWith(messages: []),
      action: 'chat:clear',
    );
  }

  @override
  void dispose() {
    _messageSubscription?.cancel();
    _typingTimer?.cancel();
    _reconnectTimer?.cancel();
    super.dispose();
  }
}
