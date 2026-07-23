import 'dart:io';

import 'package:test/test.dart';

void main() {
  test('typed codec helpers reject wrong Dart field codecs', () async {
    final repoRoot = _findRepoRoot(Directory.current);
    final packageConfig = File(
      _join(repoRoot.path, '.dart_tool', 'package_config.json'),
    );
    expect(
      packageConfig.existsSync(),
      isTrue,
      reason: 'Run dart pub get at the Dust workspace root before this test.',
    );

    final temp = await Directory.systemTemp.createTemp('dust_bad_codec_');
    addTearDown(() async {
      if (temp.existsSync()) {
        await temp.delete(recursive: true);
      }
    });

    final fixture = File(_join(temp.path, 'bad_codec.dart'));
    await fixture.writeAsString(r'''
import 'package:dust_dart/serde.dart';

final class Token {
  const Token(this.value);
  final String value;
}

final class WrongCodec implements SerDeCodec<String, Object?> {
  const WrongCodec();

  @override
  Object? serialize(String value) => value;

  @override
  String deserialize(Object? json) => json as String;
}

const wrongCodec = WrongCodec();

Object? encode(Token token) =>
    JsonHelper.encodeWithCodec<Token, Object?>(wrongCodec, token);

Token decode(Object? json) =>
    JsonHelper.decodeWithCodec<Token, Object?>(wrongCodec, json, 'token');
''');

    final result = await Process.run(
      Platform.resolvedExecutable,
      [
        '--packages=${packageConfig.path}',
        'analyze',
        fixture.path,
      ],
    );
    final output = '${result.stdout}\n${result.stderr}';

    expect(result.exitCode, isNot(0), reason: output);
    expect(output, contains('Serializer<Token, Object?>'));
    expect(output, contains('Deserializer<Token, Object?>'));
    expect(output, contains('argument_type_not_assignable'));
  });
}

Directory _findRepoRoot(Directory start) {
  var current = start;
  while (true) {
    final pubspec = File(_join(current.path, 'pubspec.yaml'));
    if (pubspec.existsSync() &&
        pubspec.readAsStringSync().contains('\nworkspace:')) {
      return current;
    }
    final parent = current.parent;
    if (parent.path == current.path) {
      throw StateError('Could not find Dust workspace root from ${start.path}');
    }
    current = parent;
  }
}

String _join(String first, String second, [String? third]) {
  final separator = Platform.pathSeparator;
  final joined =
      first.endsWith(separator) ? '$first$second' : '$first$separator$second';
  return third == null ? joined : '$joined$separator$third';
}
