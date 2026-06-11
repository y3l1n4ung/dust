part of 'routing_core.dart';

/// Builds a generated Flutter page for one typed route.
Page<R> generatedPage<R>({
  required String location,
  required String name,
  required Widget child,
  PageTransitionsBuilder? transition,
  bool fullscreenDialog = false,
  bool maintainState = true,
}) {
  return MaterialPage<R>(
    key: ValueKey(location),
    name: name,
    fullscreenDialog: fullscreenDialog,
    maintainState: maintainState,
    child: transition == null
        ? child
        : _RouteTransitionOverride(builder: transition, child: child),
  );
}

class _RouteTransitionOverride extends StatelessWidget {
  const _RouteTransitionOverride({required this.builder, required this.child});

  final PageTransitionsBuilder builder;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: Theme.of(context).copyWith(
        pageTransitionsTheme: PageTransitionsTheme(
          builders: {
            for (final platform in TargetPlatform.values) platform: builder,
          },
        ),
      ),
      child: child,
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
