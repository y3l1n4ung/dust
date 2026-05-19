import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:routing_prototype/app_state.dart';
import 'package:routing_prototype/layout/app_shell.dart';
import 'package:routing_prototype/main.dart';
import 'package:routing_prototype/route.dart';

void main() {
  setUp(() {
    appSession.logOut();
  });

  testWidgets(
    'redirects protected deep link to login and returns after login',
    (tester) async {
      final parser = AppRouter(
        session: appSession,
      ).config.routeInformationParser!;
      final route = await parser.parseRouteInformation(
        RouteInformation(uri: Uri.parse('/projects/712?tab=activity')),
      );

      await tester.pumpWidget(const RoutingPrototypeApp());
      await tester.pumpAndSettle();

      final router = DustRouter.of(tester.element(find.byType(Navigator)));
      router.go(route);
      await tester.pumpAndSettle();

      expect(find.text('Login'), findsOneWidget);
      expect(
        find.text('Redirected from: /projects/712?tab=activity'),
        findsOneWidget,
      );

      appSession.logIn();
      await tester.pumpAndSettle();

      expect(find.text('Project 712'), findsOneWidget);
      expect(find.text('Selected tab: activity'), findsOneWidget);
    },
  );

  testWidgets('route-level admin guard redirects non-admin users', (
    tester,
  ) async {
    appSession.logIn();
    await tester.pumpWidget(const RoutingPrototypeApp());
    await tester.pumpAndSettle();

    final router = DustRouter.of(tester.element(find.byType(Navigator)));
    router.go(const AdminRoute());
    await tester.pumpAndSettle();

    expect(find.text('Forbidden'), findsOneWidget);
  });

  testWidgets('feature guard protects invoice route', (tester) async {
    appSession.logIn();
    await tester.pumpWidget(const RoutingPrototypeApp());
    await tester.pumpAndSettle();

    final router = DustRouter.of(tester.element(find.byType(Navigator)));
    router.go(const InvoiceRoute(invoiceId: 'INV-2026-001', download: true));
    await tester.pumpAndSettle();

    expect(find.text('Invoice INV-2026-001'), findsOneWidget);
    expect(find.text('Download requested: true'), findsOneWidget);
  });

  testWidgets('public modal checkout route does not require auth', (
    tester,
  ) async {
    await tester.pumpWidget(const RoutingPrototypeApp());
    await tester.pumpAndSettle();

    final router = DustRouter.of(tester.element(find.byType(Navigator)));
    router.push(const CheckoutRoute(plan: 'team', annual: true));
    await tester.pumpAndSettle();

    expect(find.text('Checkout'), findsOneWidget);
    expect(find.text('Plan: team'), findsOneWidget);
    expect(find.text('Billing: annual'), findsOneWidget);
  });

  testWidgets('shell stays mounted across nested shell navigation', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1200, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    appSession.logIn();
    await tester.pumpWidget(const RoutingPrototypeApp());
    await tester.pumpAndSettle();

    final router = DustRouter.of<AppRoutePath>(
      tester.element(find.byType(Navigator)),
    );
    router.go(const ProjectRoute(projectId: 712, tab: 'activity'));
    await tester.pumpAndSettle();

    expect(find.byType(AppShell), findsOneWidget);
    expect(find.text('Project 712'), findsOneWidget);

    router.go(const ProjectSettingsRoute(projectId: 712, section: 'members'));
    await tester.pumpAndSettle();

    expect(find.byType(AppShell), findsOneWidget);
    expect(find.text('Project 712 Settings'), findsOneWidget);
    expect(find.text('Settings section: members'), findsOneWidget);
  });

  testWidgets('back pops from nested shell child to parent route', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1200, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    appSession.logIn();
    await tester.pumpWidget(const RoutingPrototypeApp());
    await tester.pumpAndSettle();

    final router = DustRouter.of<AppRoutePath>(
      tester.element(find.byType(Navigator)),
    );
    router.go(const ProjectRoute(projectId: 712, tab: 'activity'));
    await tester.pumpAndSettle();
    router.push(const ProjectSettingsRoute(projectId: 712, section: 'members'));
    await tester.pumpAndSettle();

    expect(find.text('Project 712 Settings'), findsOneWidget);

    router.pop();
    await tester.pumpAndSettle();

    expect(find.text('Project 712'), findsOneWidget);
    expect(find.byType(AppShell), findsOneWidget);
  });

  test(
    'parser supports nested path, query, public and compact links',
    () async {
      final parser = AppRouter(
        session: appSession,
      ).config.routeInformationParser!;

      expect(
        await parser.parseRouteInformation(
          RouteInformation(uri: Uri.parse('/invite/acme-token')),
        ),
        isA<InviteRoute>(),
      );
      expect(
        await parser.parseRouteInformation(
          RouteInformation(
            uri: Uri.parse('/projects/42/settings?section=members'),
          ),
        ),
        isA<ProjectSettingsRoute>(),
      );
      expect(
        await parser.parseRouteInformation(
          RouteInformation(
            uri: Uri.parse('/billing/invoices/INV-1?download=true'),
          ),
        ),
        isA<InvoiceRoute>(),
      );
      final compactBool = await parser.parseRouteInformation(
        RouteInformation(uri: Uri.parse('/billing/invoices/INV-1?download=1')),
      );
      expect(compactBool, isA<InvoiceRoute>());
      expect((compactBool as InvoiceRoute).download, isTrue);
      final notFound = await parser.parseRouteInformation(
        RouteInformation(uri: Uri.parse('/projects/99/settings/extra')),
      );
      expect(notFound, isA<NotFoundRoute>());
      expect(Uri.parse(notFound.location).pathSegments, ['404']);
      expect(
        Uri.parse(notFound.location).queryParameters['path'],
        '/projects/99/settings/extra',
      );
    },
  );

  test('constructor params restore into production deep link URLs', () {
    final parser = AppRouter(
      session: appSession,
    ).config.routeInformationParser!;

    expect(
      parser
          .restoreRouteInformation(const PostDetailRoute(id: 42))!
          .uri
          .toString(),
      '/posts/42',
    );
    expect(
      parser
          .restoreRouteInformation(
            const ProjectRoute(projectId: 712, tab: 'activity'),
          )!
          .uri
          .toString(),
      '/projects/712?tab=activity',
    );
    expect(
      parser
          .restoreRouteInformation(
            const InvoiceRoute(invoiceId: 'INV-2026-001', download: true),
          )!
          .uri
          .toString(),
      '/billing/invoices/INV-2026-001?download=true',
    );
    final encodedInvoiceRoute = parseAppRoute(
      Uri.parse(
        parser
            .restoreRouteInformation(const InvoiceRoute(invoiceId: 'A%2FZ'))!
            .uri
            .toString(),
      ),
    );
    expect(encodedInvoiceRoute, isA<InvoiceRoute>());
    expect((encodedInvoiceRoute as InvoiceRoute).invoiceId, 'A%2FZ');
    final encodedInvite = parseAppRoute(
      Uri.parse(const InviteRoute(token: 'hello world').location),
    );
    expect(encodedInvite, isA<InviteRoute>());
    expect((encodedInvite as InviteRoute).token, 'hello world');
    expect(
      const SearchRoute(q: 'design docs / routing', page: 2).location,
      '/search?q=design+docs+%2F+routing&page=2',
    );
  });

  test('malformed params fall back safely', () {
    expect(parseAppRoute(Uri.parse('/posts/not-an-int')), isA<NotFoundRoute>());
    expect(
      () => parseAppRoute(Uri.parse('/billing/invoices/INV-1?download=maybe')),
      throwsA(isA<AssertionError>()),
    );
  });

  test('generated pages cover configured transition builders', () {
    expect(
      buildAppRoutePage(const DashboardRoute()),
      isA<MaterialPage<void>>(),
    );
    expect(
      buildAppRoutePage(const CheckoutRoute(plan: 'team')),
      isA<MaterialPage<void>>(),
    );
    expect(buildAppRoutePage(const LoginRoute()), isA<MaterialPage<void>>());
    expect(buildAppRoutePage(const SearchRoute()), isA<MaterialPage<void>>());
  });

  test('login redirect only accepts safe local from routes', () {
    appSession.logIn();
    final router = AppRouter(session: appSession);

    expect(
      router.redirect(
        DustRouteState<AppRoutePath>(
          uri: Uri.parse('/login?from=https://evil.example/projects/1'),
          route: const LoginRoute(from: 'https://evil.example/projects/1'),
          stack: const [],
          isInitial: false,
        ),
      ),
      isA<DashboardRoute>(),
    );
    expect(
      router.redirect(
        DustRouteState<AppRoutePath>(
          uri: Uri.parse('/login?from=/projects/712?tab=activity'),
          route: const LoginRoute(from: '/projects/712?tab=activity'),
          stack: const [],
          isInitial: false,
        ),
      ),
      isA<ProjectRoute>(),
    );
  });
}
