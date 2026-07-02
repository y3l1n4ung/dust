import 'package:dust_dart/serde.dart';

part 'remote_comment.g.dart';

/// Remote comment model for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemoteComment with _$RemoteComment {
  /// Post ID.
  final int postId;

  /// Unique identifier.
  final int id;

  /// Name.
  final String name;

  /// Email.
  final String email;

  /// Body.
  final String body;

  /// Creates a [RemoteComment].
  RemoteComment({
    required this.postId,
    required this.id,
    required this.name,
    required this.email,
    required this.body,
  });

  /// Creates a [RemoteComment] from JSON.
  factory RemoteComment.fromJson(Map<String, Object?> json) =>
      _$RemoteCommentFromJson(json);
}
