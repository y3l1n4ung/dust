import 'package:flutter/material.dart' hide Route;

import '../route.dart';

@Route('/404/:path', name: 'notFound', guards: [])
class BenchmarkNotFoundPage extends StatelessWidget {
  const BenchmarkNotFoundPage({required this.path, super.key});

  final String path;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(child: Text('No benchmark route for $path')),
    );
  }
}
