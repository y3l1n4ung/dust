import 'package:dust_flutter/state.dart';

import '../../support/models/chat_message.dart';
import '../../support/models/chat_socket.dart';
import '../models/external_integration_state.dart';
import '../services/external_integration_clients.dart';

part 'external_integration_view_model.g.dart';

/// Args for the external integration ViewModel fixture.
final class ExternalIntegrationViewModelArgs extends ViewModelArgs {
  /// Creates [ExternalIntegrationViewModelArgs].
  const ExternalIntegrationViewModelArgs({
    required this.productsClient,
    required this.platformClient,
    required this.chatSocketFactory,
    super.observer,
  });

  /// HTTP-backed product client.
  final ExternalProductsClient productsClient;

  /// Platform-channel client.
  final ExternalPlatformClient platformClient;

  /// WebSocket-backed chat socket factory.
  final ExternalChatSocketFactory chatSocketFactory;
}

/// ViewModel fixture for realistic external state-management integrations.
@ViewModel(
  state: ExternalIntegrationState,
  args: ExternalIntegrationViewModelArgs,
)
class ExternalIntegrationViewModel extends $ExternalIntegrationViewModel {
  /// Creates an [ExternalIntegrationViewModel].
  ExternalIntegrationViewModel(super.args);

  static const Object _loadProductsAction = Object();
  static const Object _connectChatAction = Object();

  ShoppingChatSocket? _socket;
  StreamSubscription<ChatResponse>? _socketSub;
  Future<void>? _connectChatFuture;

  /// Loads product titles through the injected HTTP client.
  Future<void> loadProducts() async {
    final token = beginAction(_loadProductsAction);
    emit(
      state.copyWith(
        status: ExternalIntegrationStatus.loading,
        errorMessage: null,
      ),
    );

    try {
      final products = await args.productsClient.fetchProducts();
      if (!isCurrentAction(token)) return;
      emit(
        state.copyWith(
          status: ExternalIntegrationStatus.success,
          productTitles: products.map((product) => product.title).toList(),
          errorMessage: null,
        ),
      );
    } catch (error) {
      if (!isCurrentAction(token)) return;
      emit(
        state.copyWith(
          status: ExternalIntegrationStatus.error,
          errorMessage: error.toString(),
        ),
      );
    }
  }

  /// Loads a message through the injected platform channel.
  Future<void> loadPlatformMessage() async {
    final message = await args.platformClient.loadMessage();
    emit(state.copyWith(platformMessage: message));
  }

  /// Connects the injected WebSocket-backed chat socket.
  Future<void> connectChat() {
    if (_socket != null) return Future<void>.value();
    return _connectChatFuture ??= _openChat();
  }

  Future<void> _openChat() async {
    final token = beginAction(_connectChatAction);
    try {
      final socket = await args.chatSocketFactory.open();
      if (!isCurrentAction(token)) {
        await socket.close();
        return;
      }
      _socket = socket;
      _socketSub = socket.responses.listen((response) {
        emit(state.copyWith(socketReply: response.message.text));
      });
    } finally {
      _connectChatFuture = null;
    }
  }

  /// Sends a chat request through the injected WebSocket.
  Future<void> sendChat(String message) async {
    await connectChat();
    _socket?.send(ChatRequest(message: message, history: const []));
  }

  @override
  void dispose() {
    cancelAction(_connectChatAction);
    _connectChatFuture = null;
    _socketSub?.cancel();
    _socketSub = null;
    _socket?.close();
    _socket = null;
    super.dispose();
  }
}
