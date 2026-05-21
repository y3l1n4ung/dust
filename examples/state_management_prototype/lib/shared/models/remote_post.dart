import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'remote_post.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RemotePost with _$RemotePostDust {
  const RemotePost({
    required this.userId,
    required this.id,
    required this.title,
    required this.body,
  });

  final int userId;
  final int id;
  final String title;
  final String body;

  factory RemotePost.fromJson(Map<String, Object?> json) =>
      _$RemotePostFromJson(json);
}
