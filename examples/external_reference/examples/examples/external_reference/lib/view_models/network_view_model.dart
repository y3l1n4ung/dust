import 'package:ease_annotation/ease_annotation.dart';
import 'dart:convert';

import 'package:ease_state_helper/ease_state_helper.dart';

import 'package:http/http.dart' as http;

import '../models/api_user.dart';

part 'network_view_model.ease.dart';

/// Network request status
enum RequestStatus { initial, loading, success, error }

/// Network state status
class NetworkStatus {
  final List<ApiUser> users;
  final RequestStatus status;
  final String? error;
  final DateTime? lastFetched;
  final int? selectedUserId;

  const NetworkStatus({
    this.users = const [],
    this.status = RequestStatus.initial,
    this.error,
    this.lastFetched,
    this.selectedUserId,
  });

  bool get isLoading => status == RequestStatus.loading;
  bool get hasError => status == RequestStatus.error;
  bool get hasData => users.isNotEmpty;

  ApiUser? get selectedUser {
    if (selectedUserId == null) return null;
    return users.firstWhere(
      (u) => u.id == selectedUserId,
      orElse: () => users.first,
    );
  }

  NetworkStatus copyWith({
    List<ApiUser>? users,
    RequestStatus? status,
    String? error,
    DateTime? lastFetched,
    int? selectedUserId,
    bool clearSelectedUser = false,
  }) {
    return NetworkStatus(
      users: users ?? this.users,
      status: status ?? this.status,
      error: error,
      lastFetched: lastFetched ?? this.lastFetched,
      selectedUserId:
          clearSelectedUser ? null : (selectedUserId ?? this.selectedUserId),
    );
  }
}

/// Network ViewModel - demonstrates real API calls with caching
@ease
class NetworkViewModel extends StateNotifier<NetworkStatus> {
  NetworkViewModel() : super(const NetworkStatus());

  static const _baseUrl = 'https://jsonplaceholder.typicode.com';
  static const _cacheTimeout = Duration(minutes: 5);

  /// Fetch users from API
  Future<void> fetchUsers({bool forceRefresh = false}) async {
    // Check cache
    if (!forceRefresh && _isCacheValid()) {
      return;
    }

    state = state.copyWith(status: RequestStatus.loading, error: null);

    try {
      final response = await http.get(Uri.parse('$_baseUrl/users'));

      // Check if disposed before updating state
      if (!hasListeners) return;

      if (response.statusCode == 200) {
        final List<dynamic> jsonList = json.decode(response.body);
        final users = jsonList.map((json) => ApiUser.fromJson(json)).toList();

        state = state.copyWith(
          users: users,
          status: RequestStatus.success,
          lastFetched: DateTime.now(),
        );
      } else {
        throw Exception('Failed to load users: ${response.statusCode}');
      }
    } catch (e) {
      // Check if disposed before updating state
      if (!hasListeners) return;

      state = state.copyWith(
        status: RequestStatus.error,
        error: e.toString(),
      );
    }
  }

  /// Check if cache is still valid
  bool _isCacheValid() {
    if (state.lastFetched == null || state.users.isEmpty) {
      return false;
    }
    return DateTime.now().difference(state.lastFetched!) < _cacheTimeout;
  }

  /// Select a user
  void selectUser(int userId) {
    state = state.copyWith(selectedUserId: userId);
  }

  /// Clear selection
  void clearSelection() {
    state = state.copyWith(clearSelectedUser: true);
  }

  /// Refresh data (ignore cache)
  Future<void> refresh() => fetchUsers(forceRefresh: true);
}
