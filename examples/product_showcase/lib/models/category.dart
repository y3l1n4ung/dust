import 'package:dust_dart/derive.dart';

part 'category.g.dart';

/// Category model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Category with _$Category {
  /// Creates a [Category].
  const Category({required this.id, required this.title, required this.labels});

  /// Unique identifier.
  final String id;

  /// Title.
  final String title;

  /// Labels.
  final Set<String> labels;
}
