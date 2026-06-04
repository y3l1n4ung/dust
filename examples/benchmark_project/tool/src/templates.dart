import 'derive_templates.dart';
import 'serde_templates.dart';
import 'template_model.dart';

export 'template_model.dart' show fileNameForIndex;

String renderBenchmarkModelFile(int index) {
  final pattern = BenchmarkPattern.values[index % BenchmarkPattern.values.length];
  return switch (pattern) {
    BenchmarkPattern.deriveScalar => renderDeriveScalar(index),
    BenchmarkPattern.deriveLinked => renderDeriveLinked(index),
    BenchmarkPattern.deriveHierarchy => renderDeriveHierarchy(index),
    BenchmarkPattern.serdeScalar => renderSerdeScalar(index),
    BenchmarkPattern.serdeOptions => renderSerdeOptions(index),
    BenchmarkPattern.serdeNested => renderSerdeNested(index),
    BenchmarkPattern.serdeCodec => renderSerdeCodec(index),
    BenchmarkPattern.serdeLinked => renderSerdeLinked(index),
  };
}
