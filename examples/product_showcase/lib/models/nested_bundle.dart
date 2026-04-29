import 'package:derive_annotation/derive_annotation.dart';

part 'nested_bundle.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class NestedBundle with _$NestedBundleDust {
  const NestedBundle({
    required this.groups,
    required this.metrics,
  });

  final List<List<String>> groups;
  final Map<String, List<int>> metrics;
}
