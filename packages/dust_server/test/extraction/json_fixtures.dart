/// A payload whose `fromJson` throws on the wrong shape, so the extractor's
/// 422 path can be exercised.
final class CreateTodo {
  const CreateTodo(this.title);

  factory CreateTodo.fromJson(Map<String, Object?> json) {
    final title = json['title'];
    if (title is! String) {
      throw const FormatException('title must be a string');
    }
    return CreateTodo(title);
  }

  final String title;
}
