import 'package:flutter/material.dart';

/// Theme mode options
enum AppThemeMode { light, dark, system }

/// App theme state
class AppTheme {
  final AppThemeMode mode;
  final Color seedColor;

  const AppTheme({
    this.mode = AppThemeMode.system,
    this.seedColor = Colors.blue,
  });

  AppTheme copyWith({AppThemeMode? mode, Color? seedColor}) {
    return AppTheme(
      mode: mode ?? this.mode,
      seedColor: seedColor ?? this.seedColor,
    );
  }

  ThemeMode get themeMode {
    switch (mode) {
      case AppThemeMode.light:
        return ThemeMode.light;
      case AppThemeMode.dark:
        return ThemeMode.dark;
      case AppThemeMode.system:
        return ThemeMode.system;
    }
  }

  ThemeData get lightTheme => ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: seedColor,
          brightness: Brightness.light,
        ),
        useMaterial3: true,
      );

  ThemeData get darkTheme => ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: seedColor,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      );
}

/// Preset colors for theme
const themeColors = [
  Colors.blue,
  Colors.purple,
  Colors.teal,
  Colors.orange,
  Colors.pink,
  Colors.green,
  Colors.red,
  Colors.indigo,
];
