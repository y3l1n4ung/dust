import 'package:ease_annotation/ease_annotation.dart';
import 'dart:async';

import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/search_result.dart';

part 'search_view_model.ease.dart';

/// Search state status
class SearchStatus {
  final String query;
  final List<SearchResult> results;
  final bool isLoading;
  final String? error;
  final List<String> recentSearches;

  const SearchStatus({
    this.query = '',
    this.results = const [],
    this.isLoading = false,
    this.error,
    this.recentSearches = const [],
  });

  bool get hasResults => results.isNotEmpty;
  bool get showResults => query.isNotEmpty && !isLoading;

  SearchStatus copyWith({
    String? query,
    List<SearchResult>? results,
    bool? isLoading,
    String? error,
    List<String>? recentSearches,
  }) {
    return SearchStatus(
      query: query ?? this.query,
      results: results ?? this.results,
      isLoading: isLoading ?? this.isLoading,
      error: error,
      recentSearches: recentSearches ?? this.recentSearches,
    );
  }
}

/// Search ViewModel - demonstrates debounced search with async operations
@ease
class SearchViewModel extends StateNotifier<SearchStatus> {
  SearchViewModel() : super(const SearchStatus());

  Timer? _debounceTimer;

  /// Update search query with debounce
  void updateQuery(String query) {
    // Cancel previous timer
    _debounceTimer?.cancel();

    // Update query immediately for UI
    state = state.copyWith(query: query);

    if (query.isEmpty) {
      state = state.copyWith(results: [], isLoading: false);
      return;
    }

    // Start loading indicator
    state = state.copyWith(isLoading: true);

    // Debounce the actual search
    _debounceTimer = Timer(const Duration(milliseconds: 500), () {
      _performSearch(query);
    });
  }

  /// Perform the actual search
  Future<void> _performSearch(String query) async {
    try {
      // Simulate network delay
      await Future.delayed(const Duration(milliseconds: 300));

      // Check if disposed before updating state
      if (!hasListeners) return;

      // Filter mock database
      final results = mockSearchDatabase.where((item) {
        final queryLower = query.toLowerCase();
        return item.title.toLowerCase().contains(queryLower) ||
            item.description.toLowerCase().contains(queryLower) ||
            item.category.toLowerCase().contains(queryLower);
      }).toList();

      state = state.copyWith(
        results: results,
        isLoading: false,
      );
    } catch (e) {
      // Check if disposed before updating state
      if (!hasListeners) return;

      state = state.copyWith(
        error: 'Search failed: $e',
        isLoading: false,
      );
    }
  }

  /// Add to recent searches
  void addToRecentSearches(String query) {
    if (query.isEmpty) return;

    final recent = [...state.recentSearches];
    recent.remove(query); // Remove if exists
    recent.insert(0, query); // Add to front

    // Keep only last 5
    if (recent.length > 5) {
      recent.removeLast();
    }

    state = state.copyWith(recentSearches: recent);
  }

  /// Clear recent searches
  void clearRecentSearches() {
    state = state.copyWith(recentSearches: []);
  }

  /// Clear search
  void clearSearch() {
    _debounceTimer?.cancel();
    state = state.copyWith(query: '', results: [], isLoading: false);
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    super.dispose();
  }
}
