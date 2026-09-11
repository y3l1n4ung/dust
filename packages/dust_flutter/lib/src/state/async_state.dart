/// The lifecycle states a generated async view model moves through.
///
/// Each state compares by value. A generated view model only notifies when its
/// next state differs from the current one, so a state that wraps data without
/// comparing it makes that check useless: an async view model re-emitting equal
/// data rebuilt every listener, while a synchronous one holding the same type
/// did not. The type's own `==` decides, as it already did outside async mode.
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

  @override
  bool operator ==(Object other) => other is AsyncInitial<T>;

  @override
  int get hashCode => (AsyncInitial<T>).hashCode;

  @override
  String toString() => 'AsyncInitial<$T>()';
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

  @override
  bool operator ==(Object other) =>
      other is AsyncLoading<T> &&
      other.previousData == previousData &&
      other.hasPreviousData == hasPreviousData;

  @override
  int get hashCode =>
      Object.hash(AsyncLoading<T>, previousData, hasPreviousData);

  @override
  String toString() => hasPreviousData
      ? 'AsyncLoading<$T>(refreshing: $previousData)'
      : 'AsyncLoading<$T>()';
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

  @override
  bool operator ==(Object other) => other is AsyncData<T> && other.data == data;

  @override
  int get hashCode => Object.hash(AsyncData<T>, data);

  @override
  String toString() => 'AsyncData<$T>($data)';
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

  /// Two failures are the same only when their stack traces match too: a retry
  /// that fails the same way is a new failure, and the UI should see it.
  @override
  bool operator ==(Object other) =>
      other is AsyncFailure<T> &&
      other.error == error &&
      other.stackTrace == stackTrace &&
      other.previousData == previousData &&
      other.hasPreviousData == hasPreviousData;

  @override
  int get hashCode => Object.hash(
      AsyncFailure<T>, error, stackTrace, previousData, hasPreviousData);

  @override
  String toString() => hasPreviousData
      ? 'AsyncFailure<$T>($error, stale: $previousData)'
      : 'AsyncFailure<$T>($error)';
}
