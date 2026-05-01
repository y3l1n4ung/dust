import 'dart:io';

import 'src/templates.dart';

void main(List<String> args) {
  final count = _parseCount(args) ?? 5000;
  final root = Directory.current;
  final outputDir = Directory('${root.path}/lib/generated_models');

  if (outputDir.existsSync()) {
    outputDir.deleteSync(recursive: true);
  }
  outputDir.createSync(recursive: true);

  for (var index = 0; index < count; index++) {
    final file = File('${outputDir.path}/${fileNameForIndex(index)}.dart');
    file.writeAsStringSync(renderStressModelFile(index));
  }

  stdout.writeln('generated $count source files in ${outputDir.path}');
}

int? _parseCount(List<String> args) {
  for (var index = 0; index < args.length; index++) {
    if (args[index] == '--count' && index + 1 < args.length) {
      return int.tryParse(args[index + 1]);
    }
  }
  return null;
}
