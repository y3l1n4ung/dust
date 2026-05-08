import 'package:flutter/widgets.dart';
import 'package:state_management_prototype/shared/annotations.dart';

part 'shell_view_model.g.dart';

enum ShellTab { dashboard, tasks, profile }

@ViewModel()
class ShellViewModel extends ValueNotifier<ShellTab> {
  ShellViewModel() : super(ShellTab.dashboard);

  void selectTab(ShellTab tab) {
    if (value == tab) {
      return;
    }
    value = tab;
  }
}
