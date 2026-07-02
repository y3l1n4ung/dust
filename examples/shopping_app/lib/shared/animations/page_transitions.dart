import 'package:flutter/material.dart';

/// Kept for the original shopping app animation examples.
/// Dust routing uses Flutter PageTransitionsBuilder in route annotations.
enum SlideDirection {
  /// Right slide direction.
  right,

  /// Left slide direction.
  left,

  /// Up slide direction.
  up,

  /// Down slide direction.
  down,
}

/// Slide page route model for the shopping app example.
class SlidePageRoute<T> extends MaterialPageRoute<T> {
  /// Creates a [SlidePageRoute].
  SlidePageRoute({required super.builder, super.settings});
}
