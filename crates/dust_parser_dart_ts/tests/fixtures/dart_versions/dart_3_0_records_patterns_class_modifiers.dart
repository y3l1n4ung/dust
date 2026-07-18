// @dart=3.0

sealed class Shape {}

final class Circle extends Shape {
  const Circle(this.center);

  final (double x, double y) center;
}

abstract interface class ShapeReader {
  (String, {int count}) read();
}

String describeRecord((String label, int count) value) {
  switch (value) {
    case (final label, final count):
      return '$label:$count';
  }
}
