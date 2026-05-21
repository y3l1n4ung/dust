import 'package:flutter/widgets.dart';
import 'package:state_management_prototype/router/app_route_path.dart';

class AppRouteInformationParser extends RouteInformationParser<AppRoutePath> {
  @override
  Future<AppRoutePath> parseRouteInformation(
    RouteInformation routeInformation,
  ) async {
    final uri = routeInformation.uri;

    if (uri.pathSegments.isEmpty) {
      return const DashboardPath();
    }

    if (uri.pathSegments.length == 1) {
      if (uri.pathSegments[0] == 'tasks') return const TasksPath();
      if (uri.pathSegments[0] == 'profile') return const ProfilePath();
    }

    if (uri.pathSegments.length == 2) {
      if (uri.pathSegments[0] == 'posts') {
        final id = int.tryParse(uri.pathSegments[1]);
        if (id != null) return PostDetailPath(id);
      }
    }

    return const DashboardPath();
  }

  @override
  RouteInformation? restoreRouteInformation(AppRoutePath configuration) {
    return switch (configuration) {
      DashboardPath() => RouteInformation(uri: Uri.parse('/')),
      TasksPath() => RouteInformation(uri: Uri.parse('/tasks')),
      ProfilePath() => RouteInformation(uri: Uri.parse('/profile')),
      PostDetailPath(id: final id) => RouteInformation(
        uri: Uri.parse('/posts/$id'),
      ),
    };
  }
}
