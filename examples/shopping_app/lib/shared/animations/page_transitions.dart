import 'package:flutter/material.dart';

/// Kept for the original shopping app animation examples.
/// Dust routing uses Flutter PageTransitionsBuilder in route annotations.
enum SlideDirection { right, left, up, down }

class SlidePageRoute<T> extends MaterialPageRoute<T> {
  SlidePageRoute({required super.builder, super.settings});
}
