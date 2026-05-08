import 'package:flutter/material.dart';
import 'package:state_management_prototype/shared/annotations.dart';

part 'theme_view_model.g.dart';

@ViewModel()
class ThemeViewModel extends ValueNotifier<ThemeMode> {
  ThemeViewModel() : super(ThemeMode.dark);

  void toggle() {
    value = value == ThemeMode.dark ? ThemeMode.light : ThemeMode.dark;
  }
}
