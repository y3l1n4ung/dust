import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

void main() {
  test('decodeJsonObject accepts only JSON objects', () {
    expect(decodeJsonObject('{"name":"Ada"}'), {'name': 'Ada'});
    expect(
      () => decodeJsonObject('[1, 2, 3]'),
      throwsA(isA<FormatException>()),
    );
  });
}
