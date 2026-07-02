import 'package:flutter/material.dart';

import '../../../route.dart';

/// Staff dashboard screen.
@AppRoute('/staff', name: 'staff', guards: [StaffGuard])
class StaffDashboardScreen extends StatelessWidget {
  /// Creates a [StaffDashboardScreen].
  const StaffDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('Staff dashboard')),
    );
  }
}
