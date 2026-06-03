import 'package:dust_dart/dust_dart.dart';

void main() {
  const derive = Derive([ToString()]);
  print(derive.traits.length);
}
