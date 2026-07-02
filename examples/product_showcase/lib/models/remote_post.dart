import 'package:dust_dart/serde.dart';

part 'remote_post.g.dart';

/// Remote post model for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemotePost with _$RemotePost {
  /// User ID.
  final int userId;

  /// Unique identifier.
  final int id;

  /// Title.
  final String title;

  /// Body.
  final String body;

  /// Creates a [RemotePost].
  RemotePost({
    required this.userId,
    required this.id,
    required this.title,
    required this.body,
  });

  /// Creates a [RemotePost] from JSON.
  factory RemotePost.fromJson(Map<String, Object?> json) =>
      _$RemotePostFromJson(json);
}

/// Remote post draft model for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class RemotePostDraft with _$RemotePostDraft {
  /// User ID.
  final int userId;

  /// Title.
  final String title;

  /// Body.
  final String body;

  /// Creates a [RemotePostDraft].
  RemotePostDraft({
    required this.userId,
    required this.title,
    required this.body,
  });

  /// Creates a [RemotePostDraft] from JSON.
  factory RemotePostDraft.fromJson(Map<String, Object?> json) =>
      _$RemotePostDraftFromJson(json);
}
