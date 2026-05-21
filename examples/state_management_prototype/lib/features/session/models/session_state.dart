import 'package:derive_annotation/derive_annotation.dart';
import 'package:flutter/material.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

part 'session_state.g.dart';

@immutable
@Derive([ToString(), Eq(), CopyWith()])
class SessionState with _$SessionStateDust {
  const SessionState({
    this.owner,
    this.posts = const [],
    this.isLoading = true,
    this.isRefreshing = false,
    this.isPostsLoading = false,
    this.isInitialized = false,
    this.errorMessage,
  });

  final RemoteUser? owner;
  final List<RemotePost> posts;
  final bool isLoading;
  final bool isRefreshing;
  final bool isPostsLoading;
  final bool isInitialized;
  final String? errorMessage;

  bool get hasData => owner != null;
}
