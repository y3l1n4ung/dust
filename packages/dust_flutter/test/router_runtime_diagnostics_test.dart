import 'package:dust_flutter/route.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/router_runtime_support.dart';

void main() {
  test('router diagnostics are disabled by default', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<TestRoute>(runtimeConfig());
      await delegate.debugWaitForScheduledRefresh();
      await delegate.setNewRoutePath(const TestRoute('/private'));

      expect(messages, isEmpty);
    } finally {
      debugPrint = previousDebugPrint;
    }
  });

  test('router diagnostics log redirects guards and commits', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<TestRoute>(
        runtimeConfig(
          router: DebugRouter(),
          resolveGuards: (route) {
            if (route.location != '/guard-private') return const [];
            return [const LoginGuard()];
          },
        ),
      );
      await delegate.debugWaitForScheduledRefresh();
      await delegate.setNewRoutePath(const TestRoute('/private'));
      await delegate.setNewRoutePath(const TestRoute('/guard-private'));

      const fullPathsLog = 'AppRouter: Full paths for routes:\n'
          '           => /safe\n'
          '           => /private\n'
          '           => /branch\n'
          '           => /guard-private\n'
          '           => /login\n'
          '           => /nested/:id';
      const namedPathsLog = 'AppRouter: known full paths for route names:\n'
          '           safe => /safe\n'
          '           private => /private\n'
          '           branch => /branch\n'
          '           guardPrivate => /guard-private\n'
          '           login => /login\n'
          '           nestedDetail => /nested/:id';

      expect(
        messages,
        [
          fullPathsLog,
          namedPathsLog,
          'AppRouter: setting initial route /safe',
          'AppRouter: refreshing /safe',
          'AppRouter: replace /safe',
          'AppRouter: route /safe name=safe shell=- branch=-',
          'AppRouter: stack [/safe]',
          'AppRouter: restoring /private',
          'AppRouter: route /private name=private shell=- branch=-',
          'AppRouter: redirecting /private => /login',
          'AppRouter: redirect target /login name=login shell=- branch=-',
          'AppRouter: stack [/login]',
          'AppRouter: restoring /guard-private',
          'AppRouter: route /guard-private name=guardPrivate shell=- branch=-',
          'AppRouter: guards 1 for /guard-private',
          'AppRouter: guard LoginGuard for /guard-private',
          'AppRouter: guard LoginGuard redirect /guard-private => /login',
          'AppRouter: guard redirect /guard-private => /login',
          'AppRouter: guard target /login name=login shell=- branch=-',
          'AppRouter: restoring /login',
          'AppRouter: route /login name=login shell=- branch=-',
          'AppRouter: stack [/login]',
        ],
      );
    } finally {
      debugPrint = previousDebugPrint;
    }
  });

  test('router diagnostics log guard allow results', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final calls = <String>[];
      final delegate = GeneratedRouterDelegate<TestRoute>(
        runtimeConfig(
          router: DebugRouter(),
          resolveGuards: (route) => route.location == '/guard-private'
              ? [RecordingGuard(route.location, calls)]
              : const [],
        ),
      );
      await delegate.debugWaitForScheduledRefresh();
      messages.clear();

      await delegate.setNewRoutePath(const TestRoute('/guard-private'));

      expect(calls, ['/guard-private']);
      expect(messages, [
        'AppRouter: restoring /guard-private',
        'AppRouter: route /guard-private name=guardPrivate shell=- branch=-',
        'AppRouter: guards 1 for /guard-private',
        'AppRouter: guard RecordingGuard for /guard-private',
        'AppRouter: guard RecordingGuard allow /guard-private',
        'AppRouter: stack [/guard-private]',
      ]);
    } finally {
      debugPrint = previousDebugPrint;
    }
  });

  test('router diagnostics include shell and branch decisions', () async {
    final messages = <String>[];
    final previousDebugPrint = debugPrint;
    debugPrint = (message, {wrapWidth}) {
      if (message != null) messages.add(message);
    };

    try {
      final delegate = GeneratedRouterDelegate<TestRoute>(
        runtimeConfig(router: DebugRouter()),
      );
      await delegate.debugWaitForScheduledRefresh();
      messages.clear();
      await delegate.setNewRoutePath(const TestRoute('/branch'));

      expect(
        messages,
        [
          'AppRouter: restoring /branch',
          'AppRouter: route /branch name=branch shell=DebugShell branch=mainTabs',
          'AppRouter: branch - => mainTabs',
          'AppRouter: stack [/branch]',
        ],
      );
    } finally {
      debugPrint = previousDebugPrint;
    }
  });
}
