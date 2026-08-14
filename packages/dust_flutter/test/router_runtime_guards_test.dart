import 'package:dust_flutter/route.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/router_runtime_support.dart';

void main() {
  test('router redirects throw StateError after the redirect cap', () async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(router: RouterRedirectCycle()),
    );
    await delegate.debugWaitForScheduledRefresh();

    await expectLater(
      delegate.setNewRoutePath(const TestRoute('/one')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          'Router hit the redirect cap (8) navigating to "/one". '
              'Redirect chain: /one -> /two -> /one -> /two -> /one -> '
              '/two -> /one -> /two -> /one. '
              'Check redirect() for a cycle or return null to allow navigation.',
        ),
      ),
    );
  });

  test('guard redirects throw StateError after the redirect cap', () async {
    final guard = GuardRedirectCycle();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        resolveGuards: (route) {
          if (!route.location.startsWith('/guard')) return const [];
          return [guard];
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await expectLater(
      delegate.setNewRoutePath(const TestRoute('/guard-one')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          'Router guard redirects hit the redirect cap (8) navigating to '
              '"/guard-one". Guard redirect chain: /guard-one -> '
              '/guard-two -> /guard-one -> /guard-two -> /guard-one -> '
              '/guard-two -> /guard-one -> /guard-two -> /guard-one. '
              'Check guards that return one of these routes repeatedly.',
        ),
      ),
    );
  });

  test('deep-link restoration guards restored ancestors in stack order',
      () async {
    final calls = <String>[];
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        restoreStack: (route) => route.location == '/child'
            ? [
                const TestRoute('/safe'),
                const TestRoute('/parent'),
                route,
              ]
            : [route],
        resolveGuards: (route) => switch (route.location) {
          '/parent' || '/child' => [RecordingGuard(route.location, calls)],
          _ => const [],
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const TestRoute('/child'));

    expect(calls, ['/parent', '/child']);
    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/parent',
      '/child',
    ]);
  });

  test('direct navigation does not re-run guards for restored ancestors',
      () async {
    final calls = <String>[];
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        restoreStack: (route) => [
          const TestRoute('/safe'),
          const TestRoute('/parent'),
          route,
        ],
        resolveGuards: (route) => switch (route.location) {
          '/parent' || '/child' => [RecordingGuard(route.location, calls)],
          _ => const [],
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const TestRoute('/child'));
    await Future<void>.delayed(Duration.zero);
    expect(calls, ['/child']);
    await delegate.popRoute();
    await expectLater(result, completion(isNull));
  });

  test('ancestor guard redirect prevents unauthorized child restoration',
      () async {
    final calls = <String>[];
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        restoreStack: (route) => route.location == '/child'
            ? [
                const TestRoute('/safe'),
                const TestRoute('/parent'),
                route,
              ]
            : [route],
        resolveGuards: (route) => switch (route.location) {
          '/parent' => [
              RedirectRecordingGuard(
                route.location,
                calls,
                const TestRoute('/login'),
              ),
            ],
          '/child' => [RecordingGuard(route.location, calls)],
          _ => const [],
        },
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const TestRoute('/child'));

    expect(calls, ['/parent']);
    expect(delegate.stack.map((route) => route.location), ['/login']);
  });

  test('route stack observer records final guard redirect stack once',
      () async {
    final router = RecordingRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        router: router,
        resolveGuards: (route) => route.location == '/guard-private'
            ? [const LoginGuard()]
            : const [],
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const TestRoute('/guard-private'));

    expect(delegate.stack.map((route) => route.location), ['/login']);
    expect(router.stackChanges, ['[/safe] => [/login]']);
  });

  test('navigation fails closed when a guard implements neither interface',
      () async {
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        resolveGuards: (route) => route.location == '/guard-private'
            ? [const UnrecognizedGuard()]
            : const [],
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await expectLater(
      delegate.setNewRoutePath(const TestRoute('/guard-private')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          contains('implements neither RouteGuard'),
        ),
      ),
    );
    expect(delegate.stack.map((route) => route.location), ['/safe']);
  });
}
