// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format width=80

part of 'chat_view_model.dart';

// **************************************************************************
// EaseGenerator
// **************************************************************************

class _ChatViewModelAspect<T> {
  final T Function(ChatState state) selector;
  final T value;
  final bool Function(T a, T b)? equals;

  const _ChatViewModelAspect(this.selector, this.value, [this.equals]);

  bool hasChanged(T newValue) {
    if (equals != null) return !equals!(value, newValue);
    return value != newValue;
  }
}

class ChatViewModelProvider extends StatefulWidget {
  final Widget child;
  const ChatViewModelProvider({super.key, required this.child});

  @override
  State<ChatViewModelProvider> createState() => _ChatViewModelProviderState();
}

class _ChatViewModelProviderState extends State<ChatViewModelProvider> {
  late final ChatViewModel _notifier = ChatViewModel();

  @override
  void initState() {
    super.initState();
    _notifier.addListener(_onStateChange);
  }

  @override
  void dispose() {
    _notifier.removeListener(_onStateChange);
    _notifier.dispose();
    super.dispose();
  }

  void _onStateChange() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return _ChatViewModelInherited(notifier: _notifier, child: widget.child);
  }
}

class _ChatViewModelInherited extends InheritedModel<_ChatViewModelAspect> {
  final ChatViewModel notifier;

  const _ChatViewModelInherited({required this.notifier, required super.child});

  @override
  bool updateShouldNotify(_ChatViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(
    _ChatViewModelInherited oldWidget,
    Set<_ChatViewModelAspect> dependencies,
  ) {
    if (dependencies.isEmpty) return true;
    for (final aspect in dependencies) {
      if (aspect.hasChanged(aspect.selector(notifier.state))) return true;
    }
    return false;
  }
}

extension ChatViewModelContext on BuildContext {
  ChatViewModel get chatViewModel {
    final inherited = InheritedModel.inheritFrom<_ChatViewModelInherited>(this);
    if (inherited == null) {
      throw StateError(
        'No ChatViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ChatViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  ChatViewModel readChatViewModel() {
    final inherited = getInheritedWidgetOfExactType<_ChatViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No ChatViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ChatViewModelProvider to your providers list',
      );
    }
    return inherited.notifier;
  }

  T selectChatViewModel<T>(
    T Function(ChatState state) selector, {
    bool Function(T a, T b)? equals,
  }) {
    final inherited = getInheritedWidgetOfExactType<_ChatViewModelInherited>();
    if (inherited == null) {
      throw StateError(
        'No ChatViewModel found in widget tree.\n'
        'Make sure you:\n'
        '1. Wrapped your app with EaseScope widget: EaseScope(providers: [...], child: MyApp())\n'
        '2. Added ChatViewModelProvider to your providers list',
      );
    }
    final currentValue = selector(inherited.notifier.state);
    InheritedModel.inheritFrom<_ChatViewModelInherited>(
      this,
      aspect: _ChatViewModelAspect<T>(selector, currentValue, equals),
    );
    return currentValue;
  }

  EaseSubscription listenOnChatViewModel(
    void Function(ChatState previous, ChatState current) listener, {
    bool fireImmediately = false,
  }) {
    return readChatViewModel()
        .listenInContext(this, listener, fireImmediately: fireImmediately);
  }
}
