enum StressPattern {
  deriveScalar,
  deriveLinked,
  deriveHierarchy,
  serdeScalar,
  serdeOptions,
  serdeNested,
  serdeCodec,
  serdeLinked,
}

String fileNameForIndex(int index) =>
    'model_${(index + 1).toString().padLeft(5, '0')}';

String primaryClassNameForIndex(int index) {
  final number = index + 1;
  return switch (StressPattern.values[index % StressPattern.values.length]) {
    StressPattern.deriveScalar => 'ScalarModel$number',
    StressPattern.deriveLinked => 'LinkedModel$number',
    StressPattern.deriveHierarchy => 'EntityView$number',
    StressPattern.serdeScalar => 'SerdeScalarModel$number',
    StressPattern.serdeOptions => 'SerdeOptionsModel$number',
    StressPattern.serdeNested => 'NestedEnvelope$number',
    StressPattern.serdeCodec => 'CodecEnvelope$number',
    StressPattern.serdeLinked => 'LinkedSerdeModel$number',
  };
}
