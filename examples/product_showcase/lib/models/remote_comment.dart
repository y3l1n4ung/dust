import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'remote_comment.g.dart';

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemoteComment with _$RemoteCommentDust {
  final int postId;
  final int id;
  final String name;
  final String email;
  final String body;

  RemoteComment({
    required this.postId,
    required this.id,
    required this.name,
    required this.email,
    required this.body,
  });

  factory RemoteComment.fromJson(Map<String, Object?> json) =>
      _$RemoteCommentFromJson(json);
}
