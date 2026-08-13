import 'dart:async';

import 'package:dust_flutter/route.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/router_runtime_support.dart';

void main() {
  test('controller pop completes push future with typed result', () async {
    final delegate = GeneratedRouterDelegate<TestRoute>(runtimeConfig());
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<bool>(const TestRoute('/detail'));
    await Future<void>.delayed(Duration.zero);

    expect(delegate.pop<bool>(true), isTrue);
    await expectLater(result, completion(isTrue));
  });

  test('route stack observer records committed stack changes once', () async {
    final router = RecordingRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(router: router),
    );
    await delegate.debugWaitForScheduledRefresh();

    expect(router.stackChanges, isEmpty);

    final result = delegate.push<void>(const TestRoute('/detail'));
    delegate.replace(const TestRoute('/detail'));
    await Future<void>.delayed(Duration.zero);
    await delegate.popRoute();
    await expectLater(result, completion(isNull));
    await delegate.setNewRoutePath(const TestRoute('/private'));

    expect(router.stackChanges, [
      '[/safe] => [/safe, /detail]',
      '[/safe, /detail] => [/safe]',
      '[/safe] => [/private]',
    ]);
  });

  test('push future completes when delegate pops the route', () async {
    final delegate = GeneratedRouterDelegate<TestRoute>(runtimeConfig());
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const TestRoute('/detail'));

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/detail',
    ]);

    await delegate.popRoute();

    await expectLater(result, completion(isNull));
  });

  test('delegate pop revalidates the exposed route', () async {
    final router = AuthRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(router: router),
    );
    await delegate.debugWaitForScheduledRefresh();

    final privateResult = delegate.push<void>(const TestRoute('/private'));
    await Future<void>.delayed(Duration.zero);
    final detailResult = delegate.push<void>(const TestRoute('/detail'));
    await Future<void>.delayed(Duration.zero);
    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/private',
      '/detail',
    ]);

    router.isAuthenticated = false;
    await delegate.popRoute();
    await Future<void>.delayed(Duration.zero);

    expect(delegate.stack.map((route) => route.location), [
      '/safe',
      '/login',
    ]);
    await expectLater(privateResult, completion(isNull));
    await expectLater(detailResult, completion(isNull));
  });

  test('unawaited navigation exceptions are routed to the router hook',
      () async {
    final router = ExceptionRecordingRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(router: router),
    );
    await delegate.debugWaitForScheduledRefresh();

    delegate.go(const TestRoute('/one'));
    await Future<void>.delayed(Duration.zero);

    expect(router.errors.single, isA<StateError>());
    expect(router.stackTraces.single, isA<StackTrace>());
  });

  test('disposing the delegate completes pending push futures with null',
      () async {
    final delegate = GeneratedRouterDelegate<TestRoute>(runtimeConfig());
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const TestRoute('/detail'));
    delegate.dispose();

    await expectLater(
      result.timeout(const Duration(seconds: 1)),
      completion(isNull),
    );
  });

  test('async guard completion after disposal cannot commit a route', () async {
    final guardResult = Completer<TestRoute?>();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(
        resolveGuards: (route) => route.location == '/detail'
            ? [BlockingGuard(guardResult.future)]
            : const [],
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    final result = delegate.push<void>(const TestRoute('/detail'));
    delegate.dispose();
    guardResult.complete(null);

    await expectLater(
      result.timeout(const Duration(seconds: 1)),
      completion(isNull),
    );
    expect(delegate.stack.map((route) => route.location), ['/safe']);
  });

  test('scheduled refresh after disposal does not notify listeners', () async {
    final router = RefreshRouter();
    final delegate = GeneratedRouterDelegate<TestRoute>(
      runtimeConfig(router: router),
    );
    await delegate.debugWaitForScheduledRefresh();

    var notifications = 0;
    delegate.addListener(() => notifications += 1);
    router.refreshNotifier.notifyListeners();
    delegate.dispose();
    await delegate.debugWaitForScheduledRefresh();

    expect(notifications, 0);
  });
}
