import 'package:derive_annotation/derive_annotation.dart';

part 'category.g.dart';

@Derive([Debug(), Clone(), PartialEq(), Hash(), CopyWith()])
class Category with _$CategoryDust {
  const Category({
    required this.id,
    required this.title,
    required this.labels,
  });

  final String id;
  final String title;
  final Set<String> labels;
}
