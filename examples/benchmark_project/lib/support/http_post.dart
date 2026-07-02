import 'package:dust_dart/serde.dart';

part 'http_post.g.dart';

/// HTTP post model for the benchmark example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class HttpPost with _$HttpPost {
  /// User ID.
  final int userId;

  /// Unique identifier.
  final int id;

  /// Title.
  final String title;

  /// Body.
  final String body;

  /// Creates a [HttpPost].
  HttpPost({
    required this.userId,
    required this.id,
    required this.title,
    required this.body,
  });

  /// Creates a [HttpPost] from JSON.
  factory HttpPost.fromJson(Map<String, Object?> json) =>
      _$HttpPostFromJson(json);
}
