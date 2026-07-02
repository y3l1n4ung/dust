import 'package:flutter/material.dart';

/// Snackbar type values for the shopping app example.
enum SnackbarType {
  /// Success snackbar type.
  success,

  /// Error snackbar type.
  error,

  /// Info snackbar type.
  info,

  /// Warning snackbar type.
  warning,
}

/// App snackbar model for the shopping app example.
class AppSnackbar {
  /// Show snackbar type.
  static void show({
    required BuildContext context,
    required String message,
    SnackbarType type = SnackbarType.info,
    Duration duration = const Duration(seconds: 3),
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    final (icon, backgroundColor, textColor) = switch (type) {
      SnackbarType.success => (Icons.check_circle, Colors.green, Colors.white),
      SnackbarType.error => (Icons.error, Colors.red, Colors.white),
      SnackbarType.warning => (Icons.warning, Colors.orange, Colors.white),
      SnackbarType.info => (Icons.info, Colors.blue, Colors.white),
    };

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(icon, color: textColor, size: 20),
            const SizedBox(width: 12),
            Expanded(
              child: Text(message, style: TextStyle(color: textColor)),
            ),
          ],
        ),
        backgroundColor: backgroundColor,
        duration: duration,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        action: actionLabel != null
            ? SnackBarAction(
                label: actionLabel,
                textColor: textColor,
                onPressed: onAction ?? () {},
              )
            : null,
      ),
    );
  }

  /// Success snackbar type.
  static void success(
    BuildContext context,
    String message, {
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    show(
      context: context,
      message: message,
      type: SnackbarType.success,
      actionLabel: actionLabel,
      onAction: onAction,
    );
  }

  /// Error snackbar type.
  static void error(
    BuildContext context,
    String message, {
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    show(
      context: context,
      message: message,
      type: SnackbarType.error,
      actionLabel: actionLabel,
      onAction: onAction,
    );
  }

  /// Info snackbar type.
  static void info(
    BuildContext context,
    String message, {
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    show(
      context: context,
      message: message,
      type: SnackbarType.info,
      actionLabel: actionLabel,
      onAction: onAction,
    );
  }

  /// Warning snackbar type.
  static void warning(
    BuildContext context,
    String message, {
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    show(
      context: context,
      message: message,
      type: SnackbarType.warning,
      actionLabel: actionLabel,
      onAction: onAction,
    );
  }
}
