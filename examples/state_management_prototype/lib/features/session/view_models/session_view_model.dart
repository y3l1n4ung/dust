import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/session/models/session_state.dart';
import 'package:state_management_prototype/shared/annotations.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';

part 'session_view_model.g.dart';

@ViewModel()
class SessionViewModel extends ValueNotifier<SessionState> {
  SessionViewModel(this._repository) : super(const SessionState());

  final PrototypeRepository _repository;
  bool _initializing = false;

  SessionState get state => value;
  set state(SessionState nextState) => value = nextState;

  Future<void> initialize() async {
    if (_initializing || state.hasData) {
      return;
    }
    _initializing = true;
    try {
      await refresh(showLoading: true);
    } finally {
      _initializing = false;
    }
  }

  Future<void> refresh({bool showLoading = false}) async {
    state = state.copyWith(
      isLoading: showLoading,
      isRefreshing: !showLoading,
      errorMessage: null,
    );

    try {
      final owner = await _repository.fetchOwner(userId: 1);
      state = state.copyWith(
        owner: owner,
        isLoading: false,
        isRefreshing: false,
        errorMessage: null,
      );
    } catch (_) {
      state = state.copyWith(
        isLoading: false,
        isRefreshing: false,
        errorMessage: 'Unable to load owner details right now.',
      );
    }
  }
}
