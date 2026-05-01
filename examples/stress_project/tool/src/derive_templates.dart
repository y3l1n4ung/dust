import 'template_model.dart';
import 'template_support.dart';

String renderDeriveScalar(int index) {
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: ["import 'package:derive_annotation/derive_annotation.dart';"],
    declarations: [
      '''
@Derive([ToString(), Eq(), CopyWith()])
class $className with ${mixinName(className)} {
  const $className({
    required this.id,
    required this.version,
    required this.score,
    this.active = true,
  });

  final String id;
  final int version;
  final double score;
  final bool active;
}''',
    ],
  );
}

String renderDeriveLinked(int index) {
  final className = primaryClassNameForIndex(index);
  final previousClass = primaryClassNameForIndex(index - 1);
  final previousFile = fileNameForIndex(index - 1);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_annotation/derive_annotation.dart';",
      "import '$previousFile.dart';",
    ],
    declarations: [
      '''
@Derive([ToString(), Eq(), CopyWith()])
class $className with ${mixinName(className)} {
  const $className({
    required this.previous,
    required this.history,
    required this.byId,
  });

  final $previousClass previous;
  final List<$previousClass> history;
  final Map<String, $previousClass> byId;
}''',
    ],
  );
}

String renderDeriveHierarchy(int index) {
  final number = index + 1;
  final baseName = 'EntityBase$number';
  final className = primaryClassNameForIndex(index);
  return renderFile(
    fileName: fileNameForIndex(index),
    imports: [
      "import 'package:derive_annotation/derive_annotation.dart';",
      "import '../support/common.dart';",
    ],
    declarations: [
      '''
@Derive([ToString(), Eq()])
abstract class $baseName extends GeneratedNode with AuditStamp, ${mixinName(baseName)} {
  const $baseName(this.id, this.code);

  final String id;
  final String code;
}''',
      '''
@Derive([ToString(), Eq(), CopyWith()])
class $className extends $baseName with ${mixinName(className)} {
  const $className(
    super.id,
    super.code, {
    required this.label,
    required this.tags,
  });

  final String label;
  final List<String> tags;
}''',
    ],
  );
}
