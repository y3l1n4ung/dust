import 'dart:async';
import 'dart:convert';

import 'package:ease_state_helper/ease_state_helper.dart';

/// A recorded state change event.
class StateEvent {
  final String viewModelType;
  final int timestamp;
  final dynamic stateSnapshot;
  final String? action;

  StateEvent({
    required this.viewModelType,
    required this.timestamp,
    required this.stateSnapshot,
    this.action,
  });

  Map<String, dynamic> toJson() => {
        'viewModelType': viewModelType,
        'timestamp': timestamp,
        'state': _serializeState(stateSnapshot),
        'action': action,
      };

  static dynamic _serializeState(dynamic state) {
    if (state == null) return null;
    // Handle primitive types
    if (state is num || state is String || state is bool) return state;
    if (state is List) return state.map(_serializeState).toList();
    if (state is Map) {
      return state.map((k, v) => MapEntry(k.toString(), _serializeState(v)));
    }
    try {
      // Try to call toJson if available
      return (state as dynamic).toJson();
    } catch (_) {
      // Return type name as fallback (non-serializable)
      return {'_type': state.runtimeType.toString(), '_nonSerializable': true};
    }
  }

  factory StateEvent.fromJson(Map<String, dynamic> json) => StateEvent(
        viewModelType: json['viewModelType'],
        timestamp: json['timestamp'],
        stateSnapshot: json['state'],
        action: json['action'],
      );
}

/// A recorded session containing multiple state events.
class RecordedSession {
  final String name;
  final DateTime recordedAt;
  final List<StateEvent> events;
  final Map<String, dynamic> initialStates;

  RecordedSession({
    required this.name,
    required this.recordedAt,
    required this.events,
    required this.initialStates,
  });

  Duration get duration {
    if (events.isEmpty) return Duration.zero;
    return Duration(
      milliseconds: events.last.timestamp - events.first.timestamp,
    );
  }

  String toJson() => jsonEncode({
        'name': name,
        'recordedAt': recordedAt.toIso8601String(),
        'events': events.map((e) => e.toJson()).toList(),
        'initialStates': initialStates.map(
          (k, v) => MapEntry(k, StateEvent._serializeState(v)),
        ),
      });

  factory RecordedSession.fromJson(String jsonStr) {
    final json = jsonDecode(jsonStr) as Map<String, dynamic>;
    return RecordedSession(
      name: json['name'],
      recordedAt: DateTime.parse(json['recordedAt']),
      events:
          (json['events'] as List).map((e) => StateEvent.fromJson(e)).toList(),
      initialStates: json['initialStates'] ?? {},
    );
  }
}

/// State for the time machine.
class TimeMachineState {
  final bool canUndo;
  final bool canRedo;
  final int undoCount;
  final int redoCount;
  final bool isRecording;
  final bool isPlaying;
  final int recordingCount;

  const TimeMachineState({
    this.canUndo = false,
    this.canRedo = false,
    this.undoCount = 0,
    this.redoCount = 0,
    this.isRecording = false,
    this.isPlaying = false,
    this.recordingCount = 0,
  });
}

/// Callback for state serialization.
typedef StateSerializer<T> = Map<String, dynamic> Function(T state);
typedef StateDeserializer<T> = T Function(Map<String, dynamic> json);

/// ViewModel that provides undo/redo functionality.
class TimeMachineViewModel<T> extends StateNotifier<TimeMachineState> {
  final StateNotifier<T> _notifier;
  final int _maxHistory;

  final List<T> _undoStack = [];
  final List<T> _redoStack = [];
  final List<T> _recording = [];

  bool _isUndoRedoing = false;
  bool _isRecording = false;
  bool _isPlaying = false;
  Timer? _playbackTimer;

  TimeMachineViewModel(this._notifier, {int maxHistory = 50})
      : _maxHistory = maxHistory,
        super(const TimeMachineState());

  bool get canUndo => _undoStack.isNotEmpty;
  bool get canRedo => _redoStack.isNotEmpty;
  bool get isRecording => _isRecording;
  bool get isPlaying => _isPlaying;

  void recordChange(T oldState, T newState) {
    if (_isUndoRedoing || _isPlaying) return;

    _undoStack.add(oldState);
    _redoStack.clear();

    if (_isRecording) {
      _recording.add(newState);
    }

    while (_undoStack.length > _maxHistory) {
      _undoStack.removeAt(0);
    }

    _updateState();
  }

  void _updateState() {
    state = TimeMachineState(
      canUndo: _undoStack.isNotEmpty,
      canRedo: _redoStack.isNotEmpty,
      undoCount: _undoStack.length,
      redoCount: _redoStack.length,
      isRecording: _isRecording,
      isPlaying: _isPlaying,
      recordingCount: _recording.length,
    );
  }

  bool undo() {
    if (!canUndo || _isPlaying) return false;

    _isUndoRedoing = true;
    try {
      final currentState = _notifier.state;
      final previousState = _undoStack.removeLast();
      _redoStack.add(currentState);

      _notifier.setState(previousState, action: 'undo');
      _updateState();
      return true;
    } finally {
      _isUndoRedoing = false;
    }
  }

  bool redo() {
    if (!canRedo || _isPlaying) return false;

    _isUndoRedoing = true;
    try {
      final currentState = _notifier.state;
      final nextState = _redoStack.removeLast();
      _undoStack.add(currentState);

      _notifier.setState(nextState, action: 'redo');
      _updateState();
      return true;
    } finally {
      _isUndoRedoing = false;
    }
  }

  void clearHistory() {
    _undoStack.clear();
    _redoStack.clear();
    _updateState();
  }

  void startRecording() {
    if (_isPlaying) return;
    _recording.clear();
    _recording.add(_notifier.state);
    _isRecording = true;
    _updateState();
  }

  void stopRecording() {
    _isRecording = false;
    _updateState();
  }

  void playRecording({Duration interval = const Duration(milliseconds: 300)}) {
    if (_recording.isEmpty || _isPlaying) return;

    _isPlaying = true;
    _updateState();

    int index = 0;
    _playbackTimer = Timer.periodic(interval, (timer) {
      if (index >= _recording.length) {
        stopPlayback();
        return;
      }

      _isUndoRedoing = true;
      _notifier.setState(_recording[index], action: 'replay');
      _isUndoRedoing = false;
      index++;
    });
  }

  void stopPlayback() {
    _playbackTimer?.cancel();
    _playbackTimer = null;
    _isPlaying = false;
    _updateState();
  }

  void clearRecording() {
    _recording.clear();
    _updateState();
  }

  @override
  void dispose() {
    _playbackTimer?.cancel();
    super.dispose();
  }
}

/// Global session state.
class SessionState {
  final bool isRecording;
  final bool isPlaying;
  final int eventCount;
  final Duration? duration;
  final String? sessionName;

  const SessionState({
    this.isRecording = false,
    this.isPlaying = false,
    this.eventCount = 0,
    this.duration,
    this.sessionName,
  });
}

/// Middleware that provides time-travel debugging with session recording.
class TimeMachineMiddleware extends EaseMiddleware {
  final int maxHistory;
  final Set<Type>? includedStates;
  final Set<Type>? excludedStates;

  // Per-ViewModel controllers
  static final Map<StateNotifier<dynamic>, TimeMachineViewModel<dynamic>>
      _controllers = {};

  // Global session recording
  static bool _isSessionRecording = false;
  static bool _isSessionPlaying = false;
  static int _sessionStartTime = 0;
  static final List<StateEvent> _sessionEvents = [];
  static final Map<String, dynamic> _initialStates = {};
  static final List<RecordedSession> _savedSessions = [];
  static Timer? _sessionPlaybackTimer;

  // Session state notifier for UI updates
  static final _sessionStateNotifier = ValueNotifier<SessionState>(
    const SessionState(),
  );

  /// Listen to session state changes.
  static ValueNotifier<SessionState> get sessionState => _sessionStateNotifier;

  /// All saved sessions.
  static List<RecordedSession> get savedSessions =>
      List.unmodifiable(_savedSessions);

  TimeMachineMiddleware({
    this.maxHistory = 50,
    this.includedStates,
    this.excludedStates,
  });

  bool _shouldTrack(Type stateType) {
    if (stateType.toString().startsWith('TimeMachineViewModel')) return false;
    if (excludedStates?.contains(stateType) == true) return false;
    if (includedStates != null && !includedStates!.contains(stateType)) {
      return false;
    }
    return true;
  }

  static void _updateSessionState() {
    _sessionStateNotifier.value = SessionState(
      isRecording: _isSessionRecording,
      isPlaying: _isSessionPlaying,
      eventCount: _sessionEvents.length,
      duration: _sessionEvents.isEmpty
          ? null
          : Duration(
              milliseconds: _sessionEvents.last.timestamp -
                  (_sessionEvents.first.timestamp)),
    );
  }

  // ============================================
  // Per-ViewModel API
  // ============================================

  static TimeMachineViewModel<T>? of<T>(StateNotifier<T> notifier) {
    return _controllers[notifier] as TimeMachineViewModel<T>?;
  }

  // ============================================
  // Global Session Recording API
  // ============================================

  /// Start recording a global session.
  static void startSession() {
    if (_isSessionPlaying) return;

    _sessionEvents.clear();
    _initialStates.clear();
    _sessionStartTime = DateTime.now().millisecondsSinceEpoch;
    _isSessionRecording = true;

    // Capture initial state of all tracked ViewModels
    for (final entry in _controllers.entries) {
      final vmType = entry.key.runtimeType.toString();
      _initialStates[vmType] = entry.key.state;
    }

    _updateSessionState();
  }

  /// Stop recording the session.
  static RecordedSession? stopSession({String name = 'Session'}) {
    if (!_isSessionRecording) return null;

    _isSessionRecording = false;
    _updateSessionState();

    if (_sessionEvents.isEmpty) return null;

    final session = RecordedSession(
      name: name,
      recordedAt: DateTime.now(),
      events: List.from(_sessionEvents),
      initialStates: Map.from(_initialStates),
    );

    _savedSessions.add(session);
    return session;
  }

  /// Export current session as JSON string.
  static String? exportSession({String name = 'Exported Session'}) {
    final session = RecordedSession(
      name: name,
      recordedAt: DateTime.now(),
      events: List.from(_sessionEvents),
      initialStates: Map.from(_initialStates),
    );

    if (session.events.isEmpty) return null;
    return session.toJson();
  }

  /// Import a session from JSON string.
  static RecordedSession? importSession(String jsonStr) {
    try {
      final session = RecordedSession.fromJson(jsonStr);
      _savedSessions.add(session);
      return session;
    } catch (e) {
      return null;
    }
  }

  /// Play back a recorded session.
  static void playSession(
    RecordedSession session, {
    Duration interval = const Duration(milliseconds: 100),
    double speed = 1.0,
  }) {
    if (_isSessionPlaying || session.events.isEmpty) return;

    _isSessionPlaying = true;
    _updateSessionState();

    int eventIndex = 0;
    final adjustedInterval = Duration(
      milliseconds: (interval.inMilliseconds / speed).round(),
    );

    _sessionPlaybackTimer = Timer.periodic(adjustedInterval, (timer) {
      if (eventIndex >= session.events.length) {
        stopSessionPlayback();
        return;
      }

      final event = session.events[eventIndex];

      // Find the matching notifier and apply state
      for (final entry in _controllers.entries) {
        final vmType = entry.key.runtimeType.toString();
        if (vmType == event.viewModelType) {
          // Apply the state
          try {
            entry.key.setState(event.stateSnapshot, action: 'session-replay');
          } catch (_) {
            // State type mismatch, skip
          }
          break;
        }
      }

      eventIndex++;
    });
  }

  /// Stop session playback.
  static void stopSessionPlayback() {
    _sessionPlaybackTimer?.cancel();
    _sessionPlaybackTimer = null;
    _isSessionPlaying = false;
    _updateSessionState();
  }

  /// Clear all saved sessions.
  static void clearSessions() {
    _savedSessions.clear();
  }

  /// Get session recording status.
  static bool get isSessionRecording => _isSessionRecording;
  static bool get isSessionPlaying => _isSessionPlaying;
  static int get sessionEventCount => _sessionEvents.length;

  // ============================================
  // Middleware Hooks
  // ============================================

  @override
  void onStateInit<T>(StateInitEvent<T> event) {
    if (!_shouldTrack(event.notifier.runtimeType)) return;

    _controllers[event.notifier] = TimeMachineViewModel<T>(
      event.notifier,
      maxHistory: maxHistory,
    );
  }

  @override
  void onStateChange<T>(StateChangeEvent<T> event) {
    if (!_shouldTrack(event.notifier.runtimeType)) return;

    // Per-ViewModel tracking
    final controller = _controllers[event.notifier] as TimeMachineViewModel<T>?;
    controller?.recordChange(event.oldState, event.newState);

    // Global session recording
    if (_isSessionRecording && !_isSessionPlaying) {
      _sessionEvents.add(StateEvent(
        viewModelType: event.notifier.runtimeType.toString(),
        timestamp: DateTime.now().millisecondsSinceEpoch - _sessionStartTime,
        stateSnapshot: event.newState,
        action: event.action,
      ));
      _updateSessionState();
    }
  }

  @override
  void onStateDispose(StateDisposeEvent event) {
    _controllers.removeWhere((notifier, controller) {
      if (notifier.runtimeType.toString() == event.stateName) {
        controller.dispose();
        return true;
      }
      return false;
    });
  }
}
