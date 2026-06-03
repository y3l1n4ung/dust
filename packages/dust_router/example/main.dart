import 'package:dust_flutter/route.dart';

@Router(initial: DashboardPage, notFound: NotFoundPage)
final class AppRouter {}

@Route('/', name: 'dashboard', shell: AppShell)
final class DashboardPage {
  const DashboardPage();
}

@Route('/404/:path', name: 'notFound')
final class NotFoundPage {
  const NotFoundPage({required this.path});

  final String path;
}

final class AppShell {
  const AppShell();
}

const metadata = GeneratedRoute(
  '/',
  page: DashboardPage,
  name: 'dashboard',
  shell: AppShell,
);

void main() {}
