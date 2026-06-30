import 'package:flutter/material.dart';

import '../../../route.dart';

@AppRoute('/staff', name: 'staff', guards: [StaffGuard])
class StaffDashboardScreen extends StatelessWidget {
  const StaffDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('Staff dashboard')),
    );
  }
}
