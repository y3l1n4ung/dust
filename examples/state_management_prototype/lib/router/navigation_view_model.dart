import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/router/app_route_path.dart';

class NavigationViewModel extends ValueNotifier<List<AppRoutePath>> {
  NavigationViewModel() : super([const DashboardPath()]);

  void push(AppRoutePath path) {
    value = [...value, path];
  }

  void pop() {
    if (value.length > 1) {
      value = value.sublist(0, value.length - 1);
    }
  }

  void setPath(AppRoutePath path) {
    value = [path];
  }

  AppRoutePath get currentPath => value.last;

  void handleShellTab(ShellTab tab) {
    switch (tab) {
      case ShellTab.dashboard:
        setPath(const DashboardPath());
      case ShellTab.tasks:
        setPath(const TasksPath());
      case ShellTab.profile:
        setPath(const ProfilePath());
    }
  }
}
