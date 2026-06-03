import 'package:dust_dart/serde.dart';

part 'remote_post.g.dart';

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemotePost with _$RemotePost {
  final int userId;
  final int id;
  final String title;
  final String body;

  RemotePost({
    required this.userId,
    required this.id,
    required this.title,
    required this.body,
  });

  factory RemotePost.fromJson(Map<String, Object?> json) =>
      _$RemotePostFromJson(json);
}

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemotePostDraft with _$RemotePostDraft {
  final int userId;
  final String title;
  final String body;

  RemotePostDraft({
    required this.userId,
    required this.title,
    required this.body,
  });

  factory RemotePostDraft.fromJson(Map<String, Object?> json) =>
      _$RemotePostDraftFromJson(json);
}
