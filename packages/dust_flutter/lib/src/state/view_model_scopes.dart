import 'package:flutter/widgets.dart';

/// Builds one ViewModel scope around [child].
typedef ViewModelScopeBuilder = Widget Function(Widget child);

/// Groups generated ViewModel scopes without manual nesting.
///
/// Scopes are nested in list order. The first scope is the outermost scope.
class ViewModelScopes extends StatelessWidget {
  /// Creates a scope group.
  const ViewModelScopes({
    required this.child,
    super.key,
    this.scopes = const [],
  });

  /// Scope builders to wrap around [child].
  final List<ViewModelScopeBuilder> scopes;

  /// Widget wrapped by all [scopes].
  final Widget child;

  @override
  Widget build(BuildContext context) {
    var current = child;
    for (final scope in scopes.reversed) {
      current = scope(current);
    }
    return current;
  }
}
