import 'package:dust_state/dust_state.dart';

part 'shell_view_model.g.dart';

enum ShellTab { dashboard, tasks, profile }

final class ShellViewModelArgs extends ViewModelArgs {
  const ShellViewModelArgs({super.observer});
}

@ViewModel(
  state: ShellTab,
  args: ShellViewModelArgs,
  initial: ShellTab.dashboard,
)
class ShellViewModel extends $ShellViewModel {
  ShellViewModel(super.args);

  void selectTab(ShellTab tab) {
    if (state == tab) {
      return;
    }
    emit(tab);
  }
}
