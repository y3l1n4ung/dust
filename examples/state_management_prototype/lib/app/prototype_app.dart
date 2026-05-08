import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/dashboard/views/dashboard_page.dart';
import 'package:state_management_prototype/features/profile/views/profile_page.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/views/tasks_page.dart';
import 'package:state_management_prototype/features/theme/view_models/theme_view_model.dart';

class PrototypeApp extends StatelessWidget {
  const PrototypeApp({super.key});

  @override
  Widget build(BuildContext context) {
    final themeViewModel = context.themeViewModel;

    return ValueListenableBuilder<ThemeMode>(
      valueListenable: themeViewModel,
      builder: (context, themeMode, child) {
        return MaterialApp(
          debugShowCheckedModeBanner: false,
          title: 'Dust State Prototype',
          themeMode: themeMode,
          theme: _buildTheme(Brightness.light),
          darkTheme: _buildTheme(Brightness.dark),
          home: const _PrototypeShell(),
        );
      },
    );
  }
}

ThemeData _buildTheme(Brightness brightness) {
  final seed = brightness == Brightness.light
      ? const Color(0xFF0B7285)
      : const Color(0xFF80EDC7);
  final scheme = ColorScheme.fromSeed(seedColor: seed, brightness: brightness);

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: scheme.surface,
    appBarTheme: AppBarTheme(
      backgroundColor: Colors.transparent,
      foregroundColor: scheme.onSurface,
      elevation: 0,
    ),
    cardTheme: CardThemeData(
      elevation: 0,
      color: scheme.surfaceContainerHighest.withValues(alpha: 0.75),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(24)),
    ),
    navigationBarTheme: NavigationBarThemeData(
      indicatorColor: scheme.primaryContainer,
      backgroundColor: scheme.surface,
      labelTextStyle: WidgetStateProperty.all(
        const TextStyle(fontWeight: FontWeight.w600),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: scheme.surfaceContainerHigh,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(18),
        borderSide: BorderSide.none,
      ),
    ),
    chipTheme: ChipThemeData(
      selectedColor: scheme.primaryContainer,
      backgroundColor: scheme.surfaceContainerHigh,
      labelStyle: TextStyle(color: scheme.onSurface),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
    ),
  );
}

class _PrototypeShell extends StatelessWidget {
  const _PrototypeShell();

  @override
  Widget build(BuildContext context) {
    final shellViewModel = context.shellViewModel;

    return ValueListenableBuilder<ShellTab>(
      valueListenable: shellViewModel,
      builder: (context, selectedTab, child) {
        return Scaffold(
          body: DecoratedBox(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  Theme.of(context)
                      .colorScheme
                      .primaryContainer
                      .withValues(alpha: 0.55),
                  Theme.of(context).colorScheme.surface,
                  Theme.of(context)
                      .colorScheme
                      .tertiaryContainer
                      .withValues(alpha: 0.35),
                ],
              ),
            ),
            child: SafeArea(
              child: IndexedStack(
                index: selectedTab.index,
                children: const [
                  DashboardPage(),
                  TasksPage(),
                  ProfilePage(),
                ],
              ),
            ),
          ),
          bottomNavigationBar: NavigationBar(
            selectedIndex: selectedTab.index,
            onDestinationSelected: (index) =>
                shellViewModel.selectTab(ShellTab.values[index]),
            destinations: const [
              NavigationDestination(
                icon: Icon(Icons.dashboard_outlined),
                selectedIcon: Icon(Icons.dashboard),
                label: 'Overview',
              ),
              NavigationDestination(
                icon: Icon(Icons.checklist_rtl_outlined),
                selectedIcon: Icon(Icons.checklist_rtl),
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
      },
    );
  }
}
