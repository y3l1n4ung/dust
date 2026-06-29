import 'package:flutter/material.dart';

import '../route.dart';

@AppRoute('/404', name: 'notFound', guards: [])
class BenchmarkNotFoundPage extends StatelessWidget {
  const BenchmarkNotFoundPage({this.path = '', super.key});

  final String path;

  @override
  Widget build(BuildContext context) {
    return Scaffold(body: Center(child: Text('No benchmark route for $path')));
  }
}
