import 'package:flutter/material.dart';

import '../../../route.dart';

/// Staff dashboard screen.
@AppRoute('/staff', name: 'staff', guards: [StaffGuard])
class StaffDashboardScreen extends StatelessWidget {
  /// Creates a [StaffDashboardScreen].
  const StaffDashboardScreen({
    this.access = ShoppingAccessLevel.staff,
    this.from,
    this.returnTo,
    this.sections = const <String>[],
    this.orderIds,
    super.key,
  });

  /// Access level selected by a staff dashboard deep link.
  final ShoppingAccessLevel access;

  /// Optional start date selected by a staff dashboard deep link.
  final DateTime? from;

  /// Optional return URI selected by a staff dashboard deep link.
  final Uri? returnTo;

  /// Staff dashboard sections selected by repeated query parameters.
  final List<String> sections;

  /// Order IDs selected by repeated query parameters.
  final List<int>? orderIds;

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('Staff dashboard')),
    );
  }
}
