import 'package:flutter/widgets.dart';
import 'package:state_management_prototype/router/navigation_view_model.dart';

class NavigationViewModelScope extends InheritedWidget {
  const NavigationViewModelScope({
    super.key,
    required this.navigationViewModel,
    required super.child,
  });

  final NavigationViewModel navigationViewModel;

  static NavigationViewModel of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<NavigationViewModelScope>();
    if (scope == null) {
      throw StateError('No NavigationViewModelScope found in context.');
    }
    return scope.navigationViewModel;
  }

  static NavigationViewModel read(BuildContext context) {
    final scope =
        context
                .getElementForInheritedWidgetOfExactType<
                  NavigationViewModelScope
                >()
                ?.widget
            as NavigationViewModelScope?;
    if (scope == null) {
      throw StateError('No NavigationViewModelScope found in context.');
    }
    return scope.navigationViewModel;
  }

  @override
  bool updateShouldNotify(NavigationViewModelScope oldWidget) =>
      navigationViewModel != oldWidget.navigationViewModel;
}

extension NavigationBuildContext on BuildContext {
  NavigationViewModel get navigationViewModel =>
      NavigationViewModelScope.of(this);
  NavigationViewModel readNavigationViewModel() =>
      NavigationViewModelScope.read(this);
}
