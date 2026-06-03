import 'package:dust_dart/derive.dart';

part 'category.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class Category with _$Category {
  const Category({required this.id, required this.title, required this.labels});

  final String id;
  final String title;
  final Set<String> labels;
}
