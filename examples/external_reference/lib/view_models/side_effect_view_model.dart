import 'dart:async';
import 'dart:math';

import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

part 'side_effect_view_model.ease.dart';

/// Status of an async operation.
enum AsyncStatus { idle, loading, success, error }

/// A stock price update.
@immutable
class StockPrice {
  final String symbol;
  final double price;
  final double change;
  final DateTime timestamp;

  const StockPrice({
    required this.symbol,
    required this.price,
    required this.change,
    required this.timestamp,
  });

  double get changePercent => (change / (price - change)) * 100;
  bool get isPositive => change >= 0;
}

/// State for side effect demo.
@immutable
class SideEffectState {
  // Async operation state
  final AsyncStatus fetchStatus;
  final String? fetchResult;
  final String? fetchError;
  final int retryCount;

  // Debounced search
  final String searchQuery;
  final List<String> searchResults;
  final bool isSearching;

  // Stream state
  final List<StockPrice> stockPrices;
  final bool isStreamActive;
  final int tickCount;

  // Timer state
  final int timerSeconds;
  final bool isTimerRunning;

  const SideEffectState({
    this.fetchStatus = AsyncStatus.idle,
    this.fetchResult,
    this.fetchError,
    this.retryCount = 0,
    this.searchQuery = '',
    this.searchResults = const [],
    this.isSearching = false,
    this.stockPrices = const [],
    this.isStreamActive = false,
    this.tickCount = 0,
    this.timerSeconds = 0,
    this.isTimerRunning = false,
  });

  SideEffectState copyWith({
    AsyncStatus? fetchStatus,
    String? fetchResult,
    String? fetchError,
    int? retryCount,
    String? searchQuery,
    List<String>? searchResults,
    bool? isSearching,
    List<StockPrice>? stockPrices,
    bool? isStreamActive,
    int? tickCount,
    int? timerSeconds,
    bool? isTimerRunning,
    bool clearFetchError = false,
    bool clearFetchResult = false,
  }) {
    return SideEffectState(
      fetchStatus: fetchStatus ?? this.fetchStatus,
      fetchResult: clearFetchResult ? null : (fetchResult ?? this.fetchResult),
      fetchError: clearFetchError ? null : (fetchError ?? this.fetchError),
      retryCount: retryCount ?? this.retryCount,
      searchQuery: searchQuery ?? this.searchQuery,
      searchResults: searchResults ?? this.searchResults,
      isSearching: isSearching ?? this.isSearching,
      stockPrices: stockPrices ?? this.stockPrices,
      isStreamActive: isStreamActive ?? this.isStreamActive,
      tickCount: tickCount ?? this.tickCount,
      timerSeconds: timerSeconds ?? this.timerSeconds,
      isTimerRunning: isTimerRunning ?? this.isTimerRunning,
    );
  }
}

/// ViewModel demonstrating side effects and streaming.
@ease
class SideEffectViewModel extends StateNotifier<SideEffectState> {
  SideEffectViewModel() : super(const SideEffectState());

  // Active subscriptions
  StreamSubscription<StockPrice>? _stockSubscription;
  Timer? _debounceTimer;
  Timer? _countdownTimer;

  final _random = Random();

  // ============================================
  // Async Operations with Retry
  // ============================================

  /// Simulates an async API call with configurable failure rate.
  Future<void> fetchData({double failureRate = 0.3}) async {
    setState(
      state.copyWith(
        fetchStatus: AsyncStatus.loading,
        clearFetchError: true,
        clearFetchResult: true,
      ),
      action: 'fetchData:start',
    );

    try {
      // Simulate network delay
      await Future.delayed(const Duration(seconds: 1));

      if (!hasListeners) return;

      // Simulate random failure
      if (_random.nextDouble() < failureRate) {
        throw Exception('Network error: Connection timeout');
      }

      setState(
        state.copyWith(
          fetchStatus: AsyncStatus.success,
          fetchResult: 'Data fetched successfully at ${DateTime.now()}',
          retryCount: 0,
        ),
        action: 'fetchData:success',
      );
    } catch (e) {
      if (!hasListeners) return;

      setState(
        state.copyWith(
          fetchStatus: AsyncStatus.error,
          fetchError: e.toString(),
          retryCount: state.retryCount + 1,
        ),
        action: 'fetchData:error',
      );
    }
  }

  /// Retry with exponential backoff.
  Future<void> retryWithBackoff() async {
    final delay = Duration(milliseconds: 500 * (1 << state.retryCount));
    await Future.delayed(delay);
    if (hasListeners) {
      await fetchData();
    }
  }

  // ============================================
  // Debounced Search
  // ============================================

  /// Search with debouncing to avoid excessive API calls.
  void search(String query) {
    setState(
      state.copyWith(searchQuery: query),
      action: 'search:query',
    );

    // Cancel previous timer
    _debounceTimer?.cancel();

    if (query.isEmpty) {
      setState(
        state.copyWith(searchResults: [], isSearching: false),
        action: 'search:clear',
      );
      return;
    }

    setState(
      state.copyWith(isSearching: true),
      action: 'search:debounce',
    );

    // Debounce: wait 300ms before searching
    _debounceTimer = Timer(const Duration(milliseconds: 300), () {
      _performSearch(query);
    });
  }

  Future<void> _performSearch(String query) async {
    // Simulate API call
    await Future.delayed(const Duration(milliseconds: 500));

    if (!hasListeners) return;

    // Mock search results
    final results = List.generate(
      5,
      (i) => '${query.toUpperCase()} Result ${i + 1}',
    );

    setState(
      state.copyWith(
        searchResults: results,
        isSearching: false,
      ),
      action: 'search:results',
    );
  }

  // ============================================
  // Stream Handling
  // ============================================

  /// Start streaming stock prices.
  void startStockStream() {
    if (state.isStreamActive) return;

    setState(
      state.copyWith(isStreamActive: true, tickCount: 0),
      action: 'stream:start',
    );

    _stockSubscription = _createStockStream().listen(
      (price) {
        if (!hasListeners) return;
        setState(
          state.copyWith(
            stockPrices: [price, ...state.stockPrices.take(9)].toList(),
            tickCount: state.tickCount + 1,
          ),
          action: 'stream:tick',
        );
      },
      onError: (error) {
        if (!hasListeners) return;
        setState(
          state.copyWith(isStreamActive: false),
          action: 'stream:error',
        );
      },
      onDone: () {
        if (!hasListeners) return;
        setState(
          state.copyWith(isStreamActive: false),
          action: 'stream:done',
        );
      },
    );
  }

  /// Stop streaming.
  void stopStockStream() {
    _stockSubscription?.cancel();
    _stockSubscription = null;

    setState(
      state.copyWith(isStreamActive: false),
      action: 'stream:stop',
    );
  }

  /// Create a mock stock price stream.
  Stream<StockPrice> _createStockStream() async* {
    final symbols = ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'META'];
    final basePrices = [175.0, 140.0, 380.0, 185.0, 500.0];

    while (true) {
      await Future.delayed(const Duration(seconds: 1));

      final index = _random.nextInt(symbols.length);
      final change = (_random.nextDouble() - 0.5) * 5;

      yield StockPrice(
        symbol: symbols[index],
        price: basePrices[index] + change,
        change: change,
        timestamp: DateTime.now(),
      );
    }
  }

  // ============================================
  // Timer (Countdown)
  // ============================================

  /// Start countdown timer.
  void startTimer(int seconds) {
    _countdownTimer?.cancel();

    setState(
      state.copyWith(timerSeconds: seconds, isTimerRunning: true),
      action: 'timer:start',
    );

    _countdownTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (!hasListeners) {
        timer.cancel();
        return;
      }

      final remaining = state.timerSeconds - 1;
      if (remaining <= 0) {
        timer.cancel();
        setState(
          state.copyWith(timerSeconds: 0, isTimerRunning: false),
          action: 'timer:done',
        );
      } else {
        setState(
          state.copyWith(timerSeconds: remaining),
          action: 'timer:tick',
        );
      }
    });
  }

  /// Pause timer.
  void pauseTimer() {
    _countdownTimer?.cancel();
    setState(
      state.copyWith(isTimerRunning: false),
      action: 'timer:pause',
    );
  }

  /// Resume timer.
  void resumeTimer() {
    if (state.timerSeconds > 0) {
      startTimer(state.timerSeconds);
    }
  }

  /// Reset timer.
  void resetTimer() {
    _countdownTimer?.cancel();
    setState(
      state.copyWith(timerSeconds: 0, isTimerRunning: false),
      action: 'timer:reset',
    );
  }

  // ============================================
  // Cleanup
  // ============================================

  @override
  void dispose() {
    _stockSubscription?.cancel();
    _debounceTimer?.cancel();
    _countdownTimer?.cancel();
    super.dispose();
  }
}
