import 'dart:async';
import 'package:state_management_prototype/features/session/models/session_state.dart';
import 'package:dust_state/dust_state.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_post.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

part 'session_view_model.g.dart';

final class SessionViewModelArgs extends ViewModelArgs {
  const SessionViewModelArgs({required this.repository, super.observer});

  final PrototypeRepository repository;
}

@ViewModel(state: SessionState, args: SessionViewModelArgs)
class SessionViewModel extends $SessionViewModel {
  SessionViewModel(super.args);

  @override
  Future<void> onInit() async {
    await refresh(showLoading: true);
    emit(state.copyWith(isInitialized: true));
  }

  Future<void> refresh({bool showLoading = false}) async {
    emit(
      state.copyWith(
        isLoading: showLoading,
        isRefreshing: !showLoading,
        isPostsLoading: true,
        errorMessage: null,
      ),
    );

    try {
      final owner = await repository.fetchOwner(userId: 1);
      final posts = await repository.fetchPosts(userId: 1);
      emit(
        state.copyWith(
          owner: owner,
          posts: posts.take(3).toList(),
          isLoading: false,
          isRefreshing: false,
          isPostsLoading: false,
          errorMessage: null,
        ),
      );
    } catch (_) {
      emit(
        state.copyWith(
          isLoading: false,
          isRefreshing: false,
          isPostsLoading: false,
          errorMessage: 'Unable to load details right now.',
        ),
      );
    }
  }

  void updateName(String name) {
    if (state.owner == null) return;
    emit(state.copyWith(owner: state.owner!.copyWith(name: name)));
  }
}
