import 'package:flutter/material.dart';

import '../view_models/chat_view_model.dart';

/// Chat View - demonstrates WebSocket-like streaming.
class ChatView extends StatefulWidget {
  const ChatView({super.key});

  @override
  State<ChatView> createState() => _ChatViewState();
}

class _ChatViewState extends State<ChatView> {
  final _scrollController = ScrollController();
  final _textController = TextEditingController();
  final _focusNode = FocusNode();

  static const _hints = [
    'Hello!',
    'How are you?',
    'Thanks!',
    'I love Flutter',
    'What\'s new?',
    'LOL',
    'Bye!',
  ];
  int _hintIndex = 0;

  @override
  void initState() {
    super.initState();
    // Setup listener after first frame when context is available
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _setupListener();
    });
  }

  void _setupListener() {
    context.listenOnChatViewModel(
      (previous, current) {
        // Auto-scroll when new message arrives
        if (current.hasNewMessage &&
            current.messages.length > previous.messages.length) {
          _scrollToBottom();
        }

        // Show snackbar on connection status change
        if (previous.connectionStatus != current.connectionStatus) {
          _showConnectionStatus(current.connectionStatus);
        }
      },
    );
  }

  void _showConnectionStatus(ConnectionStatus status) {
    if (!mounted) return;

    final (message, color, icon) = switch (status) {
      ConnectionStatus.connected => (
          'Connected to chat',
          Colors.green,
          Icons.wifi
        ),
      ConnectionStatus.connecting => (
          'Connecting...',
          Colors.orange,
          Icons.wifi_find
        ),
      ConnectionStatus.disconnected => (
          'Disconnected',
          Colors.red,
          Icons.wifi_off
        ),
    };

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(icon, color: Colors.white, size: 18),
            const SizedBox(width: 8),
            Text(message),
          ],
        ),
        backgroundColor: color,
        behavior: SnackBarBehavior.floating,
        duration: const Duration(seconds: 2),
      ),
    );
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _textController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final messages = context.selectChatViewModel((s) => s.messages);
    final isTyping = context.selectChatViewModel((s) => s.isTyping);
    final typingUser = context.selectChatViewModel((s) => s.typingUser);
    final connectionStatus =
        context.selectChatViewModel((s) => s.connectionStatus);
    final isConnected = connectionStatus == ConnectionStatus.connected;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Chat Example'),
        actions: [
          // Connection status indicator
          _ConnectionIndicator(status: connectionStatus),
          const SizedBox(width: 8),
          // Connect/Disconnect button
          IconButton(
            icon: Icon(isConnected ? Icons.link_off : Icons.link),
            onPressed: () {
              final vm = context.readChatViewModel();
              if (isConnected) {
                vm.disconnect();
              } else {
                vm.connect();
              }
            },
            tooltip: isConnected ? 'Disconnect' : 'Connect',
          ),
          IconButton(
            icon: const Icon(Icons.delete_outline),
            onPressed: context.readChatViewModel().clearMessages,
            tooltip: 'Clear messages',
          ),
        ],
      ),
      body: Column(
        children: [
          // Messages
          Expanded(
            child: !isConnected && messages.isEmpty
                ? _DisconnectedPlaceholder(
                    status: connectionStatus,
                    onConnect: context.readChatViewModel().connect,
                  )
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.all(16),
                    itemCount: messages.length + (isTyping ? 1 : 0),
                    itemBuilder: (context, index) {
                      if (index == messages.length && isTyping) {
                        return _TypingIndicator(userName: typingUser);
                      }
                      return _MessageBubble(message: messages[index]);
                    },
                  ),
          ),
          // Input
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.1),
                  blurRadius: 4,
                  offset: const Offset(0, -2),
                ),
              ],
            ),
            child: SafeArea(
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _textController,
                      focusNode: _focusNode,
                      enabled: isConnected,
                      decoration: InputDecoration(
                        hintText: isConnected
                            ? 'Type a message...'
                            : 'Connect to send messages',
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(24),
                        ),
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                          vertical: 8,
                        ),
                        suffixIcon: isConnected
                            ? IconButton(
                                icon: const Icon(Icons.lightbulb_outline,
                                    size: 20),
                                tooltip: 'Add suggestion',
                                onPressed: () {
                                  _textController.text = _hints[_hintIndex];
                                  _textController.selection =
                                      TextSelection.collapsed(
                                    offset: _textController.text.length,
                                  );
                                  setState(() {
                                    _hintIndex =
                                        (_hintIndex + 1) % _hints.length;
                                  });
                                },
                              )
                            : null,
                      ),
                      textInputAction: TextInputAction.send,
                      onSubmitted: (_) {
                        if (_textController.text.isNotEmpty) {
                          context
                              .readChatViewModel()
                              .sendMessage(_textController.text);
                          _textController.clear();
                        }
                      },
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton.filled(
                    onPressed: isConnected
                        ? () {
                            if (_textController.text.isNotEmpty) {
                              context
                                  .readChatViewModel()
                                  .sendMessage(_textController.text);
                              _textController.clear();
                            }
                          }
                        : null,
                    icon: const Icon(Icons.send),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Connection status indicator.
class _ConnectionIndicator extends StatelessWidget {
  final ConnectionStatus status;

  const _ConnectionIndicator({required this.status});

  @override
  Widget build(BuildContext context) {
    final (color, label) = switch (status) {
      ConnectionStatus.connected => (Colors.green, 'LIVE'),
      ConnectionStatus.connecting => (Colors.orange, '...'),
      ConnectionStatus.disconnected => (Colors.red, 'OFF'),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.bold,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}

/// Placeholder when disconnected.
class _DisconnectedPlaceholder extends StatelessWidget {
  final ConnectionStatus status;
  final VoidCallback onConnect;

  const _DisconnectedPlaceholder({
    required this.status,
    required this.onConnect,
  });

  @override
  Widget build(BuildContext context) {
    final isConnecting = status == ConnectionStatus.connecting;

    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            isConnecting ? Icons.wifi_find : Icons.wifi_off,
            size: 64,
            color: Colors.grey.withValues(alpha: 0.5),
          ),
          const SizedBox(height: 16),
          Text(
            isConnecting ? 'Connecting...' : 'Not connected',
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                  color: Colors.grey,
                ),
          ),
          const SizedBox(height: 8),
          Text(
            isConnecting
                ? 'Please wait...'
                : 'Connect to start receiving messages',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 24),
          if (!isConnecting)
            ElevatedButton.icon(
              onPressed: onConnect,
              icon: const Icon(Icons.wifi),
              label: const Text('Connect'),
            )
          else
            const CircularProgressIndicator(),
        ],
      ),
    );
  }
}

/// Message bubble widget.
class _MessageBubble extends StatelessWidget {
  final ChatMessage message;

  const _MessageBubble({required this.message});

  @override
  Widget build(BuildContext context) {
    final isMe = message.isMe;

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisAlignment:
            isMe ? MainAxisAlignment.end : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          if (!isMe) ...[
            CircleAvatar(
              radius: 16,
              backgroundColor: _getAvatarColor(message.sender),
              child: Text(
                message.sender[0],
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              decoration: BoxDecoration(
                color: isMe
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.only(
                  topLeft: const Radius.circular(16),
                  topRight: const Radius.circular(16),
                  bottomLeft: Radius.circular(isMe ? 16 : 4),
                  bottomRight: Radius.circular(isMe ? 4 : 16),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (!isMe) ...[
                    Text(
                      message.sender,
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        color: _getAvatarColor(message.sender),
                      ),
                    ),
                    const SizedBox(height: 2),
                  ],
                  Text(
                    message.text,
                    style: TextStyle(
                      color: isMe
                          ? Theme.of(context).colorScheme.onPrimary
                          : Theme.of(context).colorScheme.onSurface,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    _formatTime(message.timestamp),
                    style: TextStyle(
                      fontSize: 10,
                      color: isMe
                          ? Theme.of(context)
                              .colorScheme
                              .onPrimary
                              .withValues(alpha: 0.7)
                          : Theme.of(context)
                              .colorScheme
                              .onSurface
                              .withValues(alpha: 0.5),
                    ),
                  ),
                ],
              ),
            ),
          ),
          if (isMe) const SizedBox(width: 8),
        ],
      ),
    );
  }

  Color _getAvatarColor(String name) {
    final colors = [
      Colors.blue,
      Colors.purple,
      Colors.orange,
      Colors.teal,
      Colors.pink,
    ];
    return colors[name.hashCode % colors.length];
  }

  String _formatTime(DateTime time) {
    return '${time.hour.toString().padLeft(2, '0')}:${time.minute.toString().padLeft(2, '0')}';
  }
}

/// Typing indicator widget.
class _TypingIndicator extends StatelessWidget {
  final String? userName;

  const _TypingIndicator({this.userName});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        children: [
          CircleAvatar(
            radius: 16,
            backgroundColor: Colors.grey.withValues(alpha: 0.3),
            child: userName != null
                ? Text(
                    userName![0],
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                    ),
                  )
                : const Icon(Icons.person, size: 18),
          ),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(16),
                topRight: Radius.circular(16),
                bottomLeft: Radius.circular(4),
                bottomRight: Radius.circular(16),
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (userName != null) ...[
                  Text(
                    userName!,
                    style: TextStyle(
                      fontSize: 11,
                      color: Theme.of(context)
                          .colorScheme
                          .onSurface
                          .withValues(alpha: 0.7),
                    ),
                  ),
                  const SizedBox(width: 8),
                ],
                _Dot(delay: 0),
                const SizedBox(width: 4),
                _Dot(delay: 150),
                const SizedBox(width: 4),
                _Dot(delay: 300),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Animated dot for typing indicator.
class _Dot extends StatefulWidget {
  final int delay;

  const _Dot({required this.delay});

  @override
  State<_Dot> createState() => _DotState();
}

class _DotState extends State<_Dot> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 600),
      vsync: this,
    );
    _animation = Tween<double>(begin: 0, end: 1).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );

    Future.delayed(Duration(milliseconds: widget.delay), () {
      if (mounted) {
        _controller.repeat(reverse: true);
      }
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _animation,
      builder: (context, child) {
        return Transform.translate(
          offset: Offset(0, -4 * _animation.value),
          child: Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: Colors.grey.withValues(alpha: 0.6),
              shape: BoxShape.circle,
            ),
          ),
        );
      },
    );
  }
}
