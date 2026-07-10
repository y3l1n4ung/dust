part of 'routing_core.dart';

/// Builds a generated Flutter page for one typed route.
Page<R> generatedPage<R>({
  required LocalKey key,
  required String location,
  required String name,
  required Widget child,
  PopInvokedWithResultCallback<Object?>? onPopInvoked,
  PageTransitionsBuilder? transition,
  bool fullscreenDialog = false,
  bool maintainState = true,
}) {
  if (transition != null) {
    return _GeneratedTransitionPage<R>(
      key: key,
      name: name,
      onPopInvoked: (didPop, result) => onPopInvoked?.call(didPop, result),
      fullscreenDialog: fullscreenDialog,
      maintainState: maintainState,
      transition: transition,
      child: child,
    );
  }

  return MaterialPage<R>(
    key: key,
    name: name,
    onPopInvoked: (didPop, result) => onPopInvoked?.call(didPop, result),
    fullscreenDialog: fullscreenDialog,
    maintainState: maintainState,
    child: child,
  );
}

final class _GeneratedTransitionPage<R> extends Page<R> {
  const _GeneratedTransitionPage({
    required this.child,
    required this.transition,
    required this.fullscreenDialog,
    required this.maintainState,
    required super.key,
    required super.name,
    required super.onPopInvoked,
  });

  final Widget child;
  final PageTransitionsBuilder transition;
  final bool fullscreenDialog;
  final bool maintainState;

  @override
  Route<R> createRoute(BuildContext context) {
    return _GeneratedTransitionRoute<R>(page: this);
  }
}

final class _GeneratedTransitionRoute<R> extends PageRoute<R> {
  _GeneratedTransitionRoute({
    required _GeneratedTransitionPage<R> page,
  })  : _page = page,
        super(
          settings: page,
          fullscreenDialog: page.fullscreenDialog,
        );

  final _GeneratedTransitionPage<R> _page;

  @override
  bool get maintainState => _page.maintainState;

  @override
  Duration get transitionDuration => _page.transition.transitionDuration;

  @override
  Duration get reverseTransitionDuration =>
      _page.transition.reverseTransitionDuration;

  @override
  Color? get barrierColor => null;

  @override
  String? get barrierLabel => null;

  @override
  DelegatedTransitionBuilder? get delegatedTransition =>
      _page.transition.delegatedTransition;

  @override
  Widget buildPage(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
  ) {
    return Semantics(
      scopesRoute: true,
      explicitChildNodes: true,
      child: _page.child,
    );
  }

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    return _page.transition.buildTransitions<R>(
      this,
      context,
      animation,
      secondaryAnimation,
      child,
    );
  }
}

/// Bottom-to-top page transition for fullscreen dialog routes.
class BottomToTopPageTransitionsBuilder extends PageTransitionsBuilder {
  /// Creates a bottom-to-top page transition builder.
  const BottomToTopPageTransitionsBuilder();

  @override
  Widget buildTransitions<T>(
    PageRoute<T> route,
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    if (MediaQuery.of(context).disableAnimations) return child;
    return SlideTransition(
      position: Tween<Offset>(
        begin: const Offset(0, 1),
        end: Offset.zero,
      ).animate(CurvedAnimation(parent: animation, curve: Curves.easeOutCubic)),
      child: child,
    );
  }
}
