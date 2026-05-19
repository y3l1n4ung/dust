import 'package:flutter/material.dart';

import 'app_state.dart';
import 'route.dart';

void main() {
  runApp(const RoutingPrototypeApp());
}

class RoutingPrototypeApp extends StatelessWidget {
  const RoutingPrototypeApp({super.key});

  @override
  Widget build(BuildContext context) {
    const seed = Color(0xFF0F766E);
    return MaterialApp.router(
      title: 'Dust Routing Prototype',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: seed,
          brightness: Brightness.light,
        ),
        scaffoldBackgroundColor: const Color(0xFFF5F1E8),
        useMaterial3: true,
        textTheme: Typography.material2021().black.apply(
          fontFamily: 'Georgia',
          displayColor: const Color(0xFF1B2523),
          bodyColor: const Color(0xFF263431),
        ),
        cardTheme: CardThemeData(
          color: Colors.white,
          elevation: 0,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(28),
            side: const BorderSide(color: Color(0xFFE1D8C7)),
          ),
        ),
        filledButtonTheme: FilledButtonThemeData(
          style: FilledButton.styleFrom(
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(999),
            ),
          ),
        ),
        outlinedButtonTheme: OutlinedButtonThemeData(
          style: OutlinedButton.styleFrom(
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(999),
            ),
          ),
        ),
      ),
      routerConfig: AppRouter(session: appSession).config,
    );
  }
}
