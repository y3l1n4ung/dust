import 'dart:io';

void main(List<String> args) {
  final count = _parseCount(args) ?? 5000;
  final root = Directory.current;
  final outputDir = Directory('${root.path}/lib/generated_models');

  if (outputDir.existsSync()) {
    outputDir.deleteSync(recursive: true);
  }
  outputDir.createSync(recursive: true);

  for (var index = 0; index < count; index++) {
    final fileName = _fileName(index);
    final file = File('${outputDir.path}/$fileName.dart');
    file.writeAsStringSync(_renderFile(index));
  }

  stdout.writeln(
    'generated $count source files in ${outputDir.path}',
  );
}

int? _parseCount(List<String> args) {
  for (var index = 0; index < args.length; index++) {
    if (args[index] == '--count' && index + 1 < args.length) {
      return int.tryParse(args[index + 1]);
    }
  }
  return null;
}

String _fileName(int index) => 'model_${(index + 1).toString().padLeft(5, '0')}';

String _renderFile(int index) {
  switch (index % 8) {
    case 0:
      return _renderScalarModel(index);
    case 1:
      return _renderNullableModel(index);
    case 2:
      return _renderCollectionModel(index);
    case 3:
      return _renderNamedConstructorModel(index);
    case 4:
      return _renderInheritanceModel(index);
    case 5:
      return _renderNestedCollectionModel(index);
    case 6:
      return _renderMixinChainModel(index);
    case 7:
      return _renderCallableRecordModel(index);
    default:
      throw StateError('unreachable pattern');
  }
}

String _renderScalarModel(int index) {
  final className = 'ScalarModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className({
    required this.id,
    required this.rank,
    this.active = true,
  });

  final String id;
  final int rank;
  final bool active;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderNullableModel(int index) {
  final className = 'NullableModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className({
    required this.id,
    this.note,
    this.aliases,
  });

  final String id;
  final String? note;
  final List<String>? aliases;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderCollectionModel(int index) {
  final className = 'CollectionModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className({
    required this.tags,
    required this.scores,
    required this.flags,
  });

  final List<String> tags;
  final Map<String, int> scores;
  final Set<String> flags;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderNamedConstructorModel(int index) {
  final className = 'RequestModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className.create({
    required this.path,
    required this.headers,
  });

  final String path;
  final Map<String, String> headers;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderInheritanceModel(int index) {
  final abstractName = 'EntityBase${index + 1}';
  final concreteName = 'EntityView${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

import '../support/common.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq()])
abstract class $abstractName extends GeneratedNode with AuditStamp, _\$$abstractName Dust {
  const $abstractName(this.id);

  final String id;
}

@Derive([Debug(), Eq(), CopyWith()])
class $concreteName extends $abstractName with _\$$concreteName Dust {
  const $concreteName(
    super.id, {
    required this.label,
    required this.tags,
  });

  final String label;
  final List<String> tags;
}
'''.replaceFirst('_\$$abstractName Dust', '_\$${abstractName}Dust').replaceFirst('_\$$concreteName Dust', '_\$${concreteName}Dust');
}

String _renderNestedCollectionModel(int index) {
  final className = 'NestedModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className({
    required this.groups,
    required this.metrics,
  });

  final List<List<String>> groups;
  final Map<String, List<int>> metrics;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderMixinChainModel(int index) {
  final className = 'TaggedModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

import '../support/common.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with AuditStamp, _\$$className Dust {
  const $className({
    required this.code,
    required this.aliases,
  });

  final String code;
  final List<String> aliases;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}

String _renderCallableRecordModel(int index) {
  final className = 'CallableRecordModel${index + 1}';
  final fileName = _fileName(index);
  return '''
import 'package:derive_annotation/derive_annotation.dart';

part '$fileName.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class $className with _\$$className Dust {
  const $className({
    required this.transform,
    required this.summary,
  });

  final void Function(String, int) transform;
  final (String, int) summary;
}
'''.replaceFirst('_\$$className Dust', '_\$${className}Dust');
}
