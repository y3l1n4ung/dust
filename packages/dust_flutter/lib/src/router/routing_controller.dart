part of 'routing_core.dart';

/// Imperative navigation controller exposed through generated extensions.
final class RouterController<T extends Object> {
  RouterController._(this._delegate);

  final GeneratedRouterDelegate<T> _delegate;

  /// Returns the nearest typed router controller.
  static RouterController<T> of<T extends Object>(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<RouterScope>();
    assert(scope != null, 'No generated router found in context.');
    return scope!.controller as RouterController<T>;
  }

  /// Top route on the current stack.
  T get currentRoute => _delegate.currentRoute;

  /// Immutable copy of the current stack.
  RouteStack<T> get stack => List.unmodifiable(_delegate.stack);

  /// Replaces the whole stack with [route].
  void go(T route) => _delegate.go(route);

  /// Pushes [route] on top of the current stack.
  void push(T route) => _delegate.push(route);

  /// Replaces the current top route with [route].
  void replace(T route) => _delegate.replace(route);

  /// Pops the top route if possible.
  void pop() => _delegate.popRoute();
}
