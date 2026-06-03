import 'package:dust_flutter/dust_flutter.dart';
import 'package:flutter/widgets.dart' as widgets;
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('exports routing annotations', () {
    const route = Route('/', name: 'home');
    const router = Router(initial: _Page, notFound: _Page);

    expect(route.path, '/');
    expect(route.name, 'home');
    expect(router.initial, _Page);
    expect(router.notFound, _Page);
  });

  test('exports state annotations', () {
    const annotation = ViewModel(state: _State, args: _Args);
    expect(annotation.state, _State);
    expect(annotation.args, _Args);
  });
}

final class _Page extends widgets.StatelessWidget {
  const _Page();

  @override
  widgets.Widget build(widgets.BuildContext context) {
    return const widgets.SizedBox.shrink();
  }
}

final class _State {
  const _State();
}

final class _Args extends ViewModelArgs {
  const _Args();
}
