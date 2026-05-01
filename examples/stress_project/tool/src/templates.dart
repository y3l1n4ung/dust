import 'derive_templates.dart';
import 'serde_templates.dart';
import 'template_model.dart';

export 'template_model.dart' show fileNameForIndex;

String renderStressModelFile(int index) {
  final pattern = StressPattern.values[index % StressPattern.values.length];
  return switch (pattern) {
    StressPattern.deriveScalar => renderDeriveScalar(index),
    StressPattern.deriveLinked => renderDeriveLinked(index),
    StressPattern.deriveHierarchy => renderDeriveHierarchy(index),
    StressPattern.serdeScalar => renderSerdeScalar(index),
    StressPattern.serdeOptions => renderSerdeOptions(index),
    StressPattern.serdeNested => renderSerdeNested(index),
    StressPattern.serdeCodec => renderSerdeCodec(index),
    StressPattern.serdeLinked => renderSerdeLinked(index),
  };
}
