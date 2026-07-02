import 'package:flutter/material.dart';

import '../../../route.dart';

/// Admin dashboard screen.
@AppRoute('/admin', name: 'admin', guards: [AdminGuard])
class AdminDashboardScreen extends StatelessWidget {
  /// Creates an [AdminDashboardScreen].
  const AdminDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('Admin dashboard')),
    );
  }
}
