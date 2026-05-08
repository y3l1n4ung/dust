import 'package:derive_annotation/derive_annotation.dart';
import 'package:flutter/material.dart';
import 'package:state_management_prototype/shared/models/remote_user.dart';

part 'session_state.g.dart';

@immutable
@Derive([ToString(), Eq(), CopyWith()])
class SessionState with _$SessionStateDust {
  const SessionState({
    this.owner,
    this.isLoading = true,
    this.isRefreshing = false,
    this.errorMessage,
  });

  final RemoteUser? owner;
  final bool isLoading;
  final bool isRefreshing;
  final String? errorMessage;

  bool get hasData => owner != null;
}
