import 'package:flutter/material.dart';
import 'package:dust_state/dust_state.dart';

part 'theme_view_model.g.dart';

final class ThemeViewModelArgs extends ViewModelArgs {
  const ThemeViewModelArgs({super.observer});
}

@ViewModel(state: ThemeMode, args: ThemeViewModelArgs, initial: ThemeMode.dark)
class ThemeViewModel extends $ThemeViewModel {
  ThemeViewModel(super.args);

  void toggle() {
    emit(state == ThemeMode.dark ? ThemeMode.light : ThemeMode.dark);
  }
}
