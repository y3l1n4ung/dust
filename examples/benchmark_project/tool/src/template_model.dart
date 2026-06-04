enum BenchmarkPattern {
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
  return switch (BenchmarkPattern.values[index % BenchmarkPattern.values.length]) {
    BenchmarkPattern.deriveScalar => 'ScalarModel$number',
    BenchmarkPattern.deriveLinked => 'LinkedModel$number',
    BenchmarkPattern.deriveHierarchy => 'EntityView$number',
    BenchmarkPattern.serdeScalar => 'SerdeScalarModel$number',
    BenchmarkPattern.serdeOptions => 'SerdeOptionsModel$number',
    BenchmarkPattern.serdeNested => 'NestedEnvelope$number',
    BenchmarkPattern.serdeCodec => 'CodecEnvelope$number',
    BenchmarkPattern.serdeLinked => 'LinkedSerdeModel$number',
  };
}
