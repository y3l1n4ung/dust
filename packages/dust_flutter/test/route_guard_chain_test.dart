import 'package:dust_flutter/route.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('runs sync and async guards in declaration order', () async {
    final calls = <String>[];
    final chain = RouteGuardChain<_Route>([
      _SyncGuard('sync-1', calls),
      _AsyncGuard('async-2', calls),
      _SyncGuard('sync-3', calls),
    ]);

    final redirected = await chain.canActivate(const _Route('requested'));

    expect(redirected, isNull);
    expect(calls, ['sync-1', 'async-2', 'sync-3']);
  });

  test('stops at first async redirect in declaration order', () async {
    final calls = <String>[];
    final chain = RouteGuardChain<_Route>([
      _SyncGuard('sync-1', calls),
      _AsyncGuard('async-2', calls, const _Route('redirected')),
      _SyncGuard('sync-3', calls),
    ]);

    final redirected = await chain.canActivate(const _Route('requested'));

    expect(redirected?.name, 'redirected');
    expect(calls, ['sync-1', 'async-2']);
  });
}

final class _Route {
  const _Route(this.name);

  final String name;
}

final class _SyncGuard implements RouteGuard<_Route> {
  const _SyncGuard(this.name, this.calls);

  final String name;
  final List<String> calls;

  @override
  _Route? canActivate(_Route route) {
    calls.add(name);
    return null;
  }
}

final class _AsyncGuard implements AsyncRouteGuard<_Route> {
  const _AsyncGuard(this.name, this.calls, [this.redirect]);

  final String name;
  final List<String> calls;
  final _Route? redirect;

  @override
  Future<_Route?> canActivate(_Route route) async {
    calls.add(name);
    return redirect;
  }
}
