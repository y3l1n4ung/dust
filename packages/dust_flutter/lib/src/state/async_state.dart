/// The lifecycle states a generated async view model moves through.
library;

/// Lifecycle state for generated async ViewModels.
sealed class AsyncState<T> {
  const AsyncState();

  /// Whether a load or refresh is in progress.
  bool get isLoading;

  /// Whether current visible data is available.
  bool get hasData => false;

  /// Whether previous visible data is available.
  bool get hasPreviousData => hasData;

  /// Whether visible data is being refreshed.
  bool get isRefreshing => false;

  /// Current visible data, when available.
  T? get data => null;

  /// Data preserved from the previous successful load, when available.
  T? get previousData => data;

  /// Current load error, when available.
  Object? get error => null;

  /// Stack trace for the current load failure, when available.
  StackTrace? get stackTrace => null;
}

/// No async load has started yet.
final class AsyncInitial<T> extends AsyncState<T> {
  /// Creates initial async state.
  const AsyncInitial();

  @override
  bool get isLoading => false;
}

/// Async data is loading.
final class AsyncLoading<T> extends AsyncState<T> {
  /// Creates loading async state.
  const AsyncLoading({this.previousData, this.hasPreviousData = false});

  @override
  final T? previousData;

  @override
  final bool hasPreviousData;

  @override
  T? get data => previousData;

  @override
  bool get hasData => hasPreviousData;

  @override
  bool get isLoading => true;

  @override
  bool get isRefreshing => hasPreviousData;
}

/// Async data loaded successfully.
final class AsyncData<T> extends AsyncState<T> {
  /// Creates data async state.
  const AsyncData(this.data);

  @override
  final T data;

  @override
  bool get hasData => true;

  @override
  bool get isLoading => false;
}

/// Async load failed.
final class AsyncFailure<T> extends AsyncState<T> {
  /// Creates failed async state.
  const AsyncFailure(
    this.error,
    this.stackTrace, {
    this.previousData,
    this.hasPreviousData = false,
  });

  @override
  final Object error;

  @override
  final StackTrace stackTrace;

  @override
  final T? previousData;

  @override
  final bool hasPreviousData;

  @override
  T? get data => previousData;

  @override
  bool get hasData => hasPreviousData;

  @override
  bool get isLoading => false;
}
