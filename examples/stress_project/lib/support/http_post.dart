import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'http_post.g.dart';

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class HttpPost with _$HttpPost {
  final int userId;
  final int id;
  final String title;
  final String body;

  HttpPost({
    required this.userId,
    required this.id,
    required this.title,
    required this.body,
  });

  factory HttpPost.fromJson(Map<String, Object?> json) =>
      _$HttpPostFromJson(json);
}
