import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/chat_message.dart';
import '../models/chat_socket.dart';
import '../models/chat_state.dart';

part 'shopping_chat_view_model.g.dart';

final class ShoppingChatViewModelArgs extends ViewModelArgs {
  const ShoppingChatViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(state: ChatState, args: ShoppingChatViewModelArgs)
class ShoppingChatViewModel extends $ShoppingChatViewModel {
  ShoppingChatViewModel(super.args);

  ShoppingChatSocket? _socket;
  StreamSubscription<ChatResponse>? _socketSub;

  @override
  void onInit() {
    _connectSocket();
    if (state.messages.isEmpty) {
      emit(
        ChatState(
          messages: [
            ChatMessage(
              id: 'assistant-welcome',
              role: ChatRole.assistant,
              text:
                  'Ask about products, coupons, order tracking, or which parts use Dust codegen.',
              createdAt: DateTime.now(),
            ),
          ],
        ),
      );
    }
  }

  Future<void> send(String text) async {
    final message = text.trim();
    if (message.isEmpty || state.status == ChatStatus.sending) return;

    _connectSocket();
    final socket = _socket;
    if (socket == null) {
      emit(
        state.copyWith(
          status: ChatStatus.error,
          errorMessage: 'Support chat socket is not connected.',
        ),
      );
      return;
    }

    final userMessage = ChatMessage(
      id: 'user-${DateTime.now().microsecondsSinceEpoch}',
      role: ChatRole.user,
      text: message,
      createdAt: DateTime.now(),
    );
    final history = [...state.messages, userMessage];
    emit(
      state.copyWith(
        messages: history,
        status: ChatStatus.sending,
        errorMessage: null,
      ),
    );

    socket.send(ChatRequest(message: message, history: history));
  }

  void _connectSocket() {
    if (_socket != null) return;
    final socket = args.repository.openChatSocket();
    _socket = socket;
    _socketSub = socket.responses.listen(
      _handleSocketResponse,
      onError: (Object error) {
        emit(
          state.copyWith(
            status: ChatStatus.error,
            errorMessage: error.toString(),
          ),
        );
      },
    );
  }

  void _handleSocketResponse(ChatResponse response) {
    emit(
      state.copyWith(
        messages: [...state.messages, response.message],
        status: ChatStatus.idle,
        errorMessage: null,
      ),
    );
  }

  @override
  void dispose() {
    _socketSub?.cancel();
    _socketSub = null;
    _socket?.close();
    _socket = null;
    super.dispose();
  }
}
