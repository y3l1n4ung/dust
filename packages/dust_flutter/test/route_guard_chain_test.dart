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

  test('propagates synchronous guard exceptions', () async {
    final chain = RouteGuardChain<_Route>([
      _ThrowingSyncGuard(StateError('sync failed')),
    ]);

    await expectLater(
      chain.canActivate(const _Route('requested')),
      throwsA(isA<StateError>()),
    );
  });

  test('propagates asynchronous guard exceptions', () async {
    final chain = RouteGuardChain<_Route>([
      _ThrowingAsyncGuard(StateError('async failed')),
    ]);

    await expectLater(
      chain.canActivate(const _Route('requested')),
      throwsA(isA<StateError>()),
    );
  });

  test('allows navigation when there are no guards', () async {
    final chain = RouteGuardChain<_Route>(const []);

    expect(await chain.canActivate(const _Route('requested')), isNull);
  });

  test('stops at first sync redirect in declaration order', () async {
    final calls = <String>[];
    final chain = RouteGuardChain<_Route>([
      _SyncGuard('sync-1', calls),
      _SyncGuard('sync-2', calls, const _Route('redirected')),
      _AsyncGuard('async-3', calls),
    ]);

    final redirected = await chain.canActivate(const _Route('requested'));

    expect(redirected?.name, 'redirected');
    expect(calls, ['sync-1', 'sync-2']);
  });

  test(
      'throws instead of skipping a guard that implements neither '
      'guard interface', () async {
    final chain = RouteGuardChain<_Route>([const _UntypedGuard()]);

    await expectLater(
      chain.canActivate(const _Route('requested')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          allOf(
            contains('_UntypedGuard'),
            contains('implements neither RouteGuard'),
          ),
        ),
      ),
    );
  });

  test('throws for a guard that only looks like a guard', () async {
    final chain = RouteGuardChain<_Route>([const _DuckTypedGuard()]);

    await expectLater(
      chain.canActivate(const _Route('requested')),
      throwsA(isA<StateError>()),
    );
  });

  test('does not run later guards after an unrecognized guard throws',
      () async {
    final calls = <String>[];
    final chain = RouteGuardChain<_Route>([
      const _UntypedGuard(),
      _SyncGuard('sync-after', calls),
    ]);

    await expectLater(
      chain.canActivate(const _Route('requested')),
      throwsA(isA<StateError>()),
    );
    expect(calls, isEmpty);
  });
}

final class _Route {
  const _Route(this.name);

  final String name;
}

final class _SyncGuard implements RouteGuard<_Route> {
  const _SyncGuard(this.name, this.calls, [this.redirect]);

  final String name;
  final List<String> calls;
  final _Route? redirect;

  @override
  _Route? canActivate(_Route route) {
    calls.add(name);
    return redirect;
  }
}

/// Satisfies the generated guard list type without implementing a guard.
final class _UntypedGuard implements RouteGuardBase<_Route> {
  const _UntypedGuard();
}

/// Declares `canActivate` but never implements a guard contract, which is what
/// happens when the `implements RouteGuard<...>` clause is dropped.
final class _DuckTypedGuard implements RouteGuardBase<_Route> {
  const _DuckTypedGuard();

  _Route? canActivate(_Route route) => const _Route('blocked');
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

final class _ThrowingSyncGuard implements RouteGuard<_Route> {
  const _ThrowingSyncGuard(this.error);

  final Object error;

  @override
  _Route? canActivate(_Route route) {
    throw error;
  }
}

final class _ThrowingAsyncGuard implements AsyncRouteGuard<_Route> {
  const _ThrowingAsyncGuard(this.error);

  final Object error;

  @override
  Future<_Route?> canActivate(_Route route) async {
    throw error;
  }
}
