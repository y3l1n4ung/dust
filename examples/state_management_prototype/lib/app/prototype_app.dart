import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/features/theme/view_models/theme_view_model.dart';
import 'package:state_management_prototype/router/app_router_delegate.dart';
import 'package:state_management_prototype/router/app_route_information_parser.dart';
import 'package:state_management_prototype/router/navigation_view_model.dart';
import 'package:state_management_prototype/router/navigation_view_model_scope.dart';
import 'package:state_management_prototype/shared/api/prototype_api.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/data/repository_scope.dart';
import 'package:state_management_prototype/shared/data/state_observer.dart';

class PrototypeApp extends StatelessWidget {
  const PrototypeApp({
    super.key,
    this.repository,
    this.observer = const LoggingStateObserver(),
  });

  final PrototypeRepository? repository;
  final StateObserver observer;

  @override
  Widget build(BuildContext context) {
    return RepositoryScope(
      repository: repository ?? LivePrototypeRepository(PrototypeApi(Dio())),
      observer: observer,
      child: SessionViewModelScope(
        args: (context) => SessionViewModelArgs(
          repository: context.repository,
          observer: context.observer,
        ),
        create: (context, args) => SessionViewModel(args),
        child: ThemeViewModelScope(
          args: (context) => ThemeViewModelArgs(observer: context.observer),
          create: (context, args) => ThemeViewModel(args),
          child: ShellViewModelScope(
            args: (context) => ShellViewModelArgs(observer: context.observer),
            create: (context, args) => ShellViewModel(args),
            child: NavigationViewModelScope(
              navigationViewModel: NavigationViewModel(),
              child: TaskBoardViewModelScope(
                args: (context) => TaskBoardViewModelArgs(
                  repository: context.repository,
                  observer: context.observer,
                ),
                create: (context, args) => TaskBoardViewModel(args),
                child: const _AppView(),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _AppView extends StatefulWidget {
  const _AppView();

  @override
  State<_AppView> createState() => _AppViewState();
}

class _AppViewState extends State<_AppView> {
  late final AppRouterDelegate _routerDelegate;
  late final AppRouteInformationParser _routeInformationParser;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final navigationViewModel = NavigationViewModelScope.read(context);
    _routerDelegate = AppRouterDelegate(navigationViewModel);
    _routeInformationParser = AppRouteInformationParser();
  }

  @override
  Widget build(BuildContext context) {
    final themeViewModel = context.watchThemeViewModel();

    return MaterialApp.router(
      title: 'Dust Prototype',
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF0B7285),
          brightness: themeViewModel.value == ThemeMode.dark
              ? Brightness.dark
              : Brightness.light,
        ),
      ),
      routerDelegate: _routerDelegate,
      routeInformationParser: _routeInformationParser,
      debugShowCheckedModeBanner: false,
    );
  }
}

class PrototypeShell extends StatelessWidget {
  const PrototypeShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final shellViewModel = context.watchShellViewModel();

    return Scaffold(
      body: child,
      bottomNavigationBar: NavigationBar(
        selectedIndex: shellViewModel.value.index,
        onDestinationSelected: (index) {
          final tab = ShellTab.values[index];
          context.readShellViewModel().selectTab(tab);
          NavigationViewModelScope.read(context).handleShellTab(tab);
        },
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.dashboard_outlined),
            selectedIcon: Icon(Icons.dashboard),
            label: 'Dashboard',
          ),
          NavigationDestination(
            icon: Icon(Icons.task_alt_outlined),
            selectedIcon: Icon(Icons.task_alt),
            label: 'Tasks',
          ),
          NavigationDestination(
            icon: Icon(Icons.person_outline),
            selectedIcon: Icon(Icons.person),
            label: 'Profile',
          ),
        ],
      ),
    );
  }
}
