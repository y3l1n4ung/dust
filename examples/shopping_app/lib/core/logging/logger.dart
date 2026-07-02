import 'dart:developer' as developer;

import 'package:flutter/foundation.dart';

/// Log level values for the shopping app example.
enum LogLevel {
  /// Debug log level.
  debug,

  /// Info log level.
  info,

  /// Warning log level.
  warning,

  /// Error log level.
  error,
}

/// App logger model for the shopping app example.
class AppLogger {
  static final AppLogger _instance = AppLogger._internal();

  /// Creates an [AppLogger] client.
  factory AppLogger() => _instance;
  AppLogger._internal();

  bool _enabled = kDebugMode;
  LogLevel _minLevel = LogLevel.debug;

  /// Configure log level.
  void configure({bool? enabled, LogLevel? minLevel}) {
    if (enabled != null) _enabled = enabled;
    if (minLevel != null) _minLevel = minLevel;
  }

  void _log(
    LogLevel level,
    String tag,
    String message, [
    Object? error,
    StackTrace? stackTrace,
  ]) {
    if (!_enabled || level.index < _minLevel.index) return;

    final timestamp = DateTime.now().toIso8601String();
    final levelStr = level.name.toUpperCase().padRight(7);
    final logMessage = '[$timestamp] $levelStr [$tag] $message';

    if (kDebugMode) {
      developer.log(
        logMessage,
        name: tag,
        error: error,
        stackTrace: stackTrace,
        level: _levelToInt(level),
      );
    }

    // Also print to console for visibility
    // ignore: avoid_print
    print(logMessage);
    if (error != null) {
      // ignore: avoid_print
      print('  Error: $error');
    }
  }

  int _levelToInt(LogLevel level) {
    switch (level) {
      case LogLevel.debug:
        return 500;

      case LogLevel.info:
        return 800;
      case LogLevel.warning:
        return 900;
      case LogLevel.error:
        return 1000;
    }
  }

  /// Debug log level.
  void debug(String tag, String message) => _log(LogLevel.debug, tag, message);

  /// Info log level.
  void info(String tag, String message) => _log(LogLevel.info, tag, message);

  /// Warning log level.
  void warning(String tag, String message) =>
      _log(LogLevel.warning, tag, message);

  /// Error log level.
  void error(
    String tag,
    String message, [
    Object? error,
    StackTrace? stackTrace,
  ]) =>
      _log(LogLevel.error, tag, message, error, stackTrace);

  // State change logging
  /// Void log level.
  void stateChange<T>(String viewModel, T oldState, T newState) {
    debug('STATE', '$viewModel: $oldState -> $newState');
  }

  // API logging
  /// API request log level.
  void apiRequest(String method, String url, [Map<String, dynamic>? body]) {
    info('API', '$method $url${body != null ? ' body: $body' : ''}');
  }

  /// API response log level.
  void apiResponse(String method, String url, int statusCode, [dynamic body]) {
    final level = statusCode >= 400 ? LogLevel.error : LogLevel.info;
    _log(level, 'API', '$method $url -> $statusCode');
  }

  /// API error log level.
  void apiError(String method, String url, Object error) {
    this.error('API', '$method $url FAILED', error);
  }

  // Navigation logging
  /// Navigation log level.
  void navigation(String from, String to) {
    debug('NAV', '$from -> $to');
  }

  // User action logging
  /// User action log level.
  void userAction(String action, [Map<String, dynamic>? details]) {
    info('USER', '$action${details != null ? ' $details' : ''}');
  }
}

// Global logger instance
/// Logger.
final logger = AppLogger();
