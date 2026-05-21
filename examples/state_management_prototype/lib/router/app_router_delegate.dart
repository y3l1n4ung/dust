import 'package:flutter/material.dart';
import 'package:state_management_prototype/app/prototype_app.dart';
import 'package:state_management_prototype/features/dashboard/views/dashboard_page.dart';
import 'package:state_management_prototype/features/posts/view_models/post_detail_view_model.dart';
import 'package:state_management_prototype/features/posts/views/post_detail_page.dart';
import 'package:state_management_prototype/features/profile/views/profile_page.dart';
import 'package:state_management_prototype/features/tasks/views/tasks_page.dart';
import 'package:state_management_prototype/router/app_route_path.dart';
import 'package:state_management_prototype/router/navigation_view_model.dart';
import 'package:state_management_prototype/shared/data/repository_scope.dart';

class AppRouterDelegate extends RouterDelegate<AppRoutePath>
    with ChangeNotifier, PopNavigatorRouterDelegateMixin<AppRoutePath> {
  AppRouterDelegate(this.navigationViewModel) {
    navigationViewModel.addListener(notifyListeners);
  }

  final NavigationViewModel navigationViewModel;

  @override
  final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();

  @override
  AppRoutePath? get currentConfiguration => navigationViewModel.currentPath;

  @override
  Widget build(BuildContext context) {
    final stack = navigationViewModel.value;

    return Navigator(
      key: navigatorKey,
      pages: [for (final path in stack) _buildPage(context, path)],
      onDidRemovePage: (page) {
        navigationViewModel.pop();
      },
    );
  }

  Page<void> _buildPage(BuildContext context, AppRoutePath path) {
    return switch (path) {
      DashboardPath() => const MaterialPage(
        key: ValueKey('Dashboard'),
        child: PrototypeShell(child: DashboardPage()),
      ),
      TasksPath() => const MaterialPage(
        key: ValueKey('Tasks'),
        child: PrototypeShell(child: TasksPage()),
      ),
      ProfilePath() => const MaterialPage(
        key: ValueKey('Profile'),
        child: PrototypeShell(child: ProfilePage()),
      ),
      PostDetailPath(id: final id) => MaterialPage(
        key: ValueKey('PostDetail-$id'),
        child: PostDetailViewModelScope(
          args: (context) => PostDetailViewModelArgs(
            repository: context.repository,
            postId: id,
            observer: context.observer,
          ),
          create: (context, args) => PostDetailViewModel(args),
          child: const PostDetailPage(),
        ),
      ),
    };
  }

  @override
  Future<void> setNewRoutePath(AppRoutePath configuration) async {
    navigationViewModel.setPath(configuration);
  }
}
