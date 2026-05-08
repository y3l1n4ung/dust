import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/post.dart';

part 'pagination_view_model.ease.dart';

/// Pagination state status
class PaginationStatus {
  final List<Post> posts;
  final int currentPage;
  final int totalPages;
  final bool isLoading;
  final bool isLoadingMore;
  final String? error;
  final bool hasReachedEnd;

  const PaginationStatus({
    this.posts = const [],
    this.currentPage = 0,
    this.totalPages = 10,
    this.isLoading = false,
    this.isLoadingMore = false,
    this.error,
    this.hasReachedEnd = false,
  });

  bool get canLoadMore => !isLoadingMore && !hasReachedEnd && !isLoading;

  PaginationStatus copyWith({
    List<Post>? posts,
    int? currentPage,
    int? totalPages,
    bool? isLoading,
    bool? isLoadingMore,
    String? error,
    bool? hasReachedEnd,
  }) {
    return PaginationStatus(
      posts: posts ?? this.posts,
      currentPage: currentPage ?? this.currentPage,
      totalPages: totalPages ?? this.totalPages,
      isLoading: isLoading ?? this.isLoading,
      isLoadingMore: isLoadingMore ?? this.isLoadingMore,
      error: error,
      hasReachedEnd: hasReachedEnd ?? this.hasReachedEnd,
    );
  }
}

/// Pagination ViewModel - demonstrates infinite scroll with load more
@ease
class PaginationViewModel extends StateNotifier<PaginationStatus> {
  PaginationViewModel() : super(const PaginationStatus());

  static const _pageSize = 10;

  /// Load initial data
  Future<void> loadInitial() async {
    if (state.isLoading) return;

    state = state.copyWith(isLoading: true, error: null);

    try {
      // Simulate network delay
      await Future.delayed(const Duration(seconds: 1));

      final posts = generateMockPosts(0, _pageSize);

      state = state.copyWith(
        posts: posts,
        currentPage: 1,
        isLoading: false,
        hasReachedEnd: false,
      );
    } catch (e) {
      state = state.copyWith(
        error: 'Failed to load posts: $e',
        isLoading: false,
      );
    }
  }

  /// Load more data (pagination)
  Future<void> loadMore() async {
    if (!state.canLoadMore) return;

    state = state.copyWith(isLoadingMore: true, error: null);

    try {
      // Simulate network delay
      await Future.delayed(const Duration(milliseconds: 800));

      final newPosts = generateMockPosts(state.currentPage, _pageSize);
      final nextPage = state.currentPage + 1;
      final hasReachedEnd = nextPage >= state.totalPages;

      state = state.copyWith(
        posts: [...state.posts, ...newPosts],
        currentPage: nextPage,
        isLoadingMore: false,
        hasReachedEnd: hasReachedEnd,
      );
    } catch (e) {
      state = state.copyWith(
        error: 'Failed to load more: $e',
        isLoadingMore: false,
      );
    }
  }

  /// Refresh data (pull to refresh)
  Future<void> refresh() async {
    state = state.copyWith(
      isLoading: true,
      error: null,
      hasReachedEnd: false,
    );

    try {
      // Simulate network delay
      await Future.delayed(const Duration(seconds: 1));

      final posts = generateMockPosts(0, _pageSize);

      state = state.copyWith(
        posts: posts,
        currentPage: 1,
        isLoading: false,
      );
    } catch (e) {
      state = state.copyWith(
        error: 'Failed to refresh: $e',
        isLoading: false,
      );
    }
  }
}
