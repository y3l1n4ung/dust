import 'package:dust_dart/dust_dart.dart';

Result<int, String> parseCount(String text) {
  final value = int.tryParse(text);
  return value == null ? const Err('invalid count') : Ok(value);
}

void main() {
  const derive = Derive([ToString()]);
  final label = parseCount(
    '42',
  ).match(ok: (value) => 'count=$value', err: (error) => error);

  print('${derive.traits.length} $label');
}
