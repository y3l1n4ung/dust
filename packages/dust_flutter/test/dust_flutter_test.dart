import 'package:dust_flutter/dust_flutter.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('exports routing annotations', () {
    const route = Route('/', name: 'home');
    const router = Router(initial: '/', notFound: '/404');

    expect(route.path, '/');
    expect(route.name, 'home');
    expect(router.initial, '/');
    expect(router.notFound, '/404');
  });

  test('exports state annotations', () {
    const annotation = ViewModel(state: _State, args: _Args);
    expect(annotation.state, _State);
    expect(annotation.args, _Args);
  });
}

final class _State {
  const _State();
}

final class _Args extends ViewModelArgs {
  const _Args();
}
