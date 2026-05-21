import 'dart:async';
import 'package:flutter/material.dart';
import 'package:dust_state/dust_state.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';

part 'post_detail_view_model.g.dart';

@immutable
class PostDetailState {
  const PostDetailState({this.post, this.isLoading = false, this.errorMessage});

  final RemotePost? post;
  final bool isLoading;
  final String? errorMessage;

  PostDetailState copyWith({
    RemotePost? post,
    bool? isLoading,
    String? errorMessage,
  }) {
    return PostDetailState(
      post: post ?? this.post,
      isLoading: isLoading ?? this.isLoading,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

final class PostDetailViewModelArgs extends ViewModelArgs {
  const PostDetailViewModelArgs({
    required this.repository,
    required this.postId,
    super.observer,
  });

  final PrototypeRepository repository;
  final int postId;
}

@ViewModel(state: PostDetailState, args: PostDetailViewModelArgs)
class PostDetailViewModel extends $PostDetailViewModel {
  PostDetailViewModel(super.args);

  @override
  Future<void> onInit() async {
    await load();
  }

  Future<void> load() async {
    emit(state.copyWith(isLoading: true, errorMessage: null));
    try {
      final post = await repository.fetchPost(postId: postId);
      emit(state.copyWith(post: post, isLoading: false));
    } catch (e) {
      emit(
        state.copyWith(
          isLoading: false,
          errorMessage: 'Failed to load post details.',
        ),
      );
    }
  }
}
