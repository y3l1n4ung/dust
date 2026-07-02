import 'package:dust_dart/derive.dart';

part 'external_integration_state.g.dart';

/// External integration load status values for the shopping app example.
enum ExternalIntegrationStatus {
  /// No external integration work has started.
  initial,

  /// External integration work is in progress.
  loading,

  /// External integration work completed.
  success,

  /// External integration work failed.
  error,
}

/// State used by the external integration ViewModel fixture.
@Derive([ToString(), CopyWith(), Eq()])
class ExternalIntegrationState with _$ExternalIntegrationState {
  /// Creates an [ExternalIntegrationState].
  const ExternalIntegrationState({
    this.status = ExternalIntegrationStatus.initial,
    this.productTitles = const [],
    this.platformMessage,
    this.socketReply,
    this.errorMessage,
  });

  /// Current load status.
  final ExternalIntegrationStatus status;

  /// Product titles loaded through an injected HTTP client.
  final List<String> productTitles;

  /// Message loaded through an injected platform channel.
  final String? platformMessage;

  /// Latest reply loaded through an injected WebSocket.
  final String? socketReply;

  /// Last external integration error.
  final String? errorMessage;
}
