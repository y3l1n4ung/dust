import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/theme.dart';

part 'theme_view_model.ease.dart';

/// Theme ViewModel - demonstrates app-wide state management
@ease
class ThemeViewModel extends StateNotifier<AppTheme> {
  ThemeViewModel() : super(const AppTheme());

  void setMode(AppThemeMode mode) {
    state = state.copyWith(mode: mode);
  }

  void toggleMode() {
    final nextMode = switch (state.mode) {
      AppThemeMode.light => AppThemeMode.dark,
      AppThemeMode.dark => AppThemeMode.system,
      AppThemeMode.system => AppThemeMode.light,
    };
    state = state.copyWith(mode: nextMode);
  }

  void setSeedColor(Color color) {
    state = state.copyWith(seedColor: color);
  }
}
