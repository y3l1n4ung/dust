import 'package:flutter/material.dart';

class ErrorDialog {
  static Future<void> show({
    required BuildContext context,
    required String title,
    required String message,
    String buttonText = 'OK',
    VoidCallback? onRetry,
  }) {
    return showDialog(
      context: context,
      builder: (context) => _ErrorDialogContent(
        title: title,
        message: message,
        buttonText: buttonText,
        onRetry: onRetry,
      ),
    );
  }
}

class _ErrorDialogContent extends StatefulWidget {
  final String title;
  final String message;
  final String buttonText;
  final VoidCallback? onRetry;

  const _ErrorDialogContent({
    required this.title,
    required this.message,
    required this.buttonText,
    this.onRetry,
  });

  @override
  State<_ErrorDialogContent> createState() => _ErrorDialogContentState();
}

class _ErrorDialogContentState extends State<_ErrorDialogContent>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _shakeAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 500),
    );

    _shakeAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 0, end: 10), weight: 1),
      TweenSequenceItem(tween: Tween(begin: 10, end: -10), weight: 2),
      TweenSequenceItem(tween: Tween(begin: -10, end: 10), weight: 2),
      TweenSequenceItem(tween: Tween(begin: 10, end: -5), weight: 2),
      TweenSequenceItem(tween: Tween(begin: -5, end: 0), weight: 1),
    ]).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));

    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _shakeAnimation,
      builder: (context, child) {
        return Transform.translate(
          offset: Offset(_shakeAnimation.value, 0),
          child: child,
        );
      },
      child: AlertDialog(
        icon: const Icon(Icons.error_outline, color: Colors.red, size: 48),
        title: Text(widget.title),
        content: Text(widget.message),
        actions: [
          if (widget.onRetry != null)
            TextButton(
              onPressed: () {
                Navigator.of(context).pop();
                widget.onRetry?.call();
              },
              child: const Text('Retry'),
            ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(widget.buttonText),
          ),
        ],
      ),
    );
  }
}
