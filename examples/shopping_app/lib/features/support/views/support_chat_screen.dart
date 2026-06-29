import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';
import '../models/chat_message.dart';
import '../models/chat_state.dart';
import '../view_models/shopping_chat_view_model.dart';

@AppRoute('/support/chat', name: 'supportChat', guards: [])
class SupportChatScreen extends StatefulWidget {
  const SupportChatScreen({super.key});

  @override
  State<SupportChatScreen> createState() => _SupportChatScreenState();
}

class _SupportChatScreenState extends State<SupportChatScreen> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watchShoppingChatViewModel().value;

    return Scaffold(
      appBar: AppBar(
        title: const TranslatedText(
          'shop_shopping_assistant',
          defaultText: 'Shopping Assistant',
        ),
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.all(16),
              itemCount: state.messages.length,
              itemBuilder: (context, index) =>
                  _ChatBubble(message: state.messages[index]),
            ),
          ),
          if (state.status == ChatStatus.error)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                state.errorMessage ??
                    context.tr('shop_chat_failed', defaultText: 'Chat failed'),
                style: const TextStyle(color: Colors.red),
              ),
            ),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      decoration: InputDecoration(
                        hintText: context.tr(
                          'shop_chat_hint',
                          defaultText: 'Ask about coupons, orders, API...',
                        ),
                        border: const OutlineInputBorder(),
                      ),
                      onSubmitted: (_) => _send(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton.filled(
                    onPressed:
                        state.status == ChatStatus.sending ? null : _send,
                    icon: state.status == ChatStatus.sending
                        ? const SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.send),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _send() {
    final text = _controller.text;
    _controller.clear();
    context.readShoppingChatViewModel().send(text);
  }
}

class _ChatBubble extends StatelessWidget {
  const _ChatBubble({required this.message});

  final ChatMessage message;

  @override
  Widget build(BuildContext context) {
    final isUser = message.role == ChatRole.user;
    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 320),
        margin: const EdgeInsets.only(bottom: 10),
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: isUser
              ? Theme.of(context).colorScheme.primary
              : Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(16),
        ),
        child: Text(
          message.text,
          style: TextStyle(color: isUser ? Colors.white : null),
        ),
      ),
    );
  }
}
