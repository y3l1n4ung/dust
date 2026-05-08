import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../middleware/time_machine_middleware.dart';
import '../view_models/drawing_view_model.dart';

/// Drawing View - demonstrates session recording & replay
class DrawingView extends StatefulWidget {
  const DrawingView({super.key});

  @override
  State<DrawingView> createState() => _DrawingViewState();
}

class _DrawingViewState extends State<DrawingView> {
  TimeMachineViewModel<DrawingState>? _timeMachine;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _timeMachine ??= TimeMachineMiddleware.of(context.readDrawingViewModel());
    _timeMachine?.addListener(_onTimeMachineChanged);
  }

  void _onTimeMachineChanged() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    _timeMachine?.removeListener(_onTimeMachineChanged);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final drawing = context.drawingViewModel;
    final tmState = _timeMachine?.state;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Drawing'),
        actions: [
          // Undo
          IconButton(
            icon: const Icon(Icons.undo),
            onPressed: tmState?.canUndo == true ? _timeMachine!.undo : null,
            tooltip: 'Undo',
          ),
          // Redo
          IconButton(
            icon: const Icon(Icons.redo),
            onPressed: tmState?.canRedo == true ? _timeMachine!.redo : null,
            tooltip: 'Redo',
          ),
          // More options
          PopupMenuButton<String>(
            icon: const Icon(Icons.more_vert),
            onSelected: (value) => _handleMenuAction(value, context),
            itemBuilder: (context) => [
              const PopupMenuItem(
                value: 'clear',
                child: ListTile(
                  leading: Icon(Icons.delete_outline),
                  title: Text('Clear Canvas'),
                  contentPadding: EdgeInsets.zero,
                ),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(
                value: 'sessions',
                child: ListTile(
                  leading: Icon(Icons.video_library),
                  title: Text('Saved Sessions'),
                  contentPadding: EdgeInsets.zero,
                ),
              ),
            ],
          ),
        ],
      ),
      body: Column(
        children: [
          // Toolbar
          _Toolbar(
            selectedType: drawing.state.selectedType,
            selectedColor: drawing.state.selectedColor,
            onTypeSelected: drawing.selectType,
            onColorSelected: drawing.selectColor,
          ),
          // Session controls
          _SessionControls(),
          // Canvas
          Expanded(
            child: _Canvas(
              shapes: drawing.state.shapes,
              onAddShape: drawing.addShape,
              isPlaying: TimeMachineMiddleware.isSessionPlaying,
            ),
          ),
        ],
      ),
    );
  }

  void _handleMenuAction(String action, BuildContext context) {
    switch (action) {
      case 'clear':
        context.readDrawingViewModel().clearCanvas();
        break;
      case 'sessions':
        _showSessionsDialog(context);
        break;
    }
  }

  void _showSessionsDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => _SessionsDialog(),
    );
  }
}

/// Session recording controls
class _SessionControls extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<SessionState>(
      valueListenable: TimeMachineMiddleware.sessionState,
      builder: (context, state, _) {
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          color: state.isRecording
              ? Colors.red.withValues(alpha: 0.1)
              : state.isPlaying
                  ? Colors.green.withValues(alpha: 0.1)
                  : Theme.of(context).colorScheme.surfaceContainerHighest,
          child: Row(
            children: [
              // Status
              if (state.isRecording) ...[
                const Icon(Icons.fiber_manual_record,
                    size: 12, color: Colors.red),
                const SizedBox(width: 8),
                Text(
                  'Recording (${state.eventCount} events)',
                  style: const TextStyle(fontSize: 12, color: Colors.red),
                ),
              ] else if (state.isPlaying) ...[
                const Icon(Icons.play_arrow, size: 16, color: Colors.green),
                const SizedBox(width: 4),
                const Text(
                  'Playing...',
                  style: TextStyle(fontSize: 12, color: Colors.green),
                ),
              ] else ...[
                Icon(
                  Icons.videocam_outlined,
                  size: 16,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 8),
                Text(
                  'Session Recording',
                  style: TextStyle(
                    fontSize: 12,
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
              const Spacer(),
              // Controls
              if (state.isPlaying)
                TextButton.icon(
                  onPressed: TimeMachineMiddleware.stopSessionPlayback,
                  icon: const Icon(Icons.stop, size: 18),
                  label: const Text('Stop'),
                )
              else if (state.isRecording)
                TextButton.icon(
                  onPressed: () => _stopAndSave(context),
                  icon: const Icon(Icons.stop, size: 18, color: Colors.red),
                  label:
                      const Text('Stop', style: TextStyle(color: Colors.red)),
                )
              else ...[
                TextButton.icon(
                  onPressed: TimeMachineMiddleware.startSession,
                  icon: const Icon(Icons.fiber_manual_record,
                      size: 18, color: Colors.red),
                  label: const Text('Record'),
                ),
                if (TimeMachineMiddleware.savedSessions.isNotEmpty)
                  TextButton.icon(
                    onPressed: () => _playLastSession(),
                    icon: const Icon(Icons.play_arrow, size: 18),
                    label: const Text('Replay'),
                  ),
              ],
            ],
          ),
        );
      },
    );
  }

  void _stopAndSave(BuildContext context) {
    final session = TimeMachineMiddleware.stopSession(name: 'Drawing Session');
    if (session != null) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
              'Session saved: ${session.events.length} events (${session.duration.inSeconds}s)'),
          action: SnackBarAction(
            label: 'Export',
            onPressed: () => _exportSession(context, session),
          ),
        ),
      );
    }
  }

  void _exportSession(BuildContext context, RecordedSession session) {
    final json = session.toJson();
    Clipboard.setData(ClipboardData(text: json));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Session JSON copied to clipboard')),
    );
  }

  void _playLastSession() {
    final sessions = TimeMachineMiddleware.savedSessions;
    if (sessions.isNotEmpty) {
      TimeMachineMiddleware.playSession(sessions.last);
    }
  }
}

/// Dialog showing saved sessions
class _SessionsDialog extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final sessions = TimeMachineMiddleware.savedSessions;

    return AlertDialog(
      title: const Text('Saved Sessions'),
      content: SizedBox(
        width: 300,
        child: sessions.isEmpty
            ? const Padding(
                padding: EdgeInsets.all(16),
                child: Text('No sessions recorded yet.\nTap Record to start.'),
              )
            : ListView.builder(
                shrinkWrap: true,
                itemCount: sessions.length,
                itemBuilder: (context, index) {
                  final session = sessions[index];
                  return ListTile(
                    leading: const Icon(Icons.video_library),
                    title: Text(session.name),
                    subtitle: Text(
                      '${session.events.length} events • ${session.duration.inSeconds}s',
                    ),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          icon: const Icon(Icons.play_arrow),
                          onPressed: () {
                            Navigator.pop(context);
                            TimeMachineMiddleware.playSession(session);
                          },
                          tooltip: 'Play',
                        ),
                        IconButton(
                          icon: const Icon(Icons.copy),
                          onPressed: () {
                            Clipboard.setData(
                                ClipboardData(text: session.toJson()));
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                  content: Text('Copied to clipboard')),
                            );
                          },
                          tooltip: 'Export JSON',
                        ),
                      ],
                    ),
                  );
                },
              ),
      ),
      actions: [
        if (sessions.isNotEmpty)
          TextButton(
            onPressed: () {
              TimeMachineMiddleware.clearSessions();
              Navigator.pop(context);
            },
            child: const Text('Clear All'),
          ),
        TextButton(
          onPressed: () => _importSession(context),
          child: const Text('Import'),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }

  void _importSession(BuildContext context) async {
    final scaffoldMessenger = ScaffoldMessenger.of(context);
    final navigator = Navigator.of(context);

    final data = await Clipboard.getData('text/plain');
    if (data?.text != null) {
      final session = TimeMachineMiddleware.importSession(data!.text!);
      if (session != null) {
        scaffoldMessenger.showSnackBar(
          SnackBar(content: Text('Imported: ${session.name}')),
        );
        navigator.pop();
      } else {
        scaffoldMessenger.showSnackBar(
          const SnackBar(content: Text('Invalid session data')),
        );
      }
    }
  }
}

class _Toolbar extends StatelessWidget {
  final ShapeType selectedType;
  final Color selectedColor;
  final ValueChanged<ShapeType> onTypeSelected;
  final ValueChanged<Color> onColorSelected;

  const _Toolbar({
    required this.selectedType,
    required this.selectedColor,
    required this.onTypeSelected,
    required this.onColorSelected,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).colorScheme.outlineVariant,
          ),
        ),
      ),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            ...ShapeType.values.map((type) => Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: ChoiceChip(
                    label: Text(_shapeLabel(type)),
                    selected: selectedType == type,
                    onSelected: (_) => onTypeSelected(type),
                  ),
                )),
            const SizedBox(width: 16),
            Container(
              width: 1,
              height: 24,
              color: Theme.of(context).colorScheme.outlineVariant,
            ),
            const SizedBox(width: 16),
            ...[
              Colors.blue,
              Colors.red,
              Colors.green,
              Colors.orange,
              Colors.purple,
              Colors.cyan
            ].map((color) => Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: GestureDetector(
                    onTap: () => onColorSelected(color),
                    child: Container(
                      width: 32,
                      height: 32,
                      decoration: BoxDecoration(
                        color: color,
                        shape: BoxShape.circle,
                        border: Border.all(
                          color: selectedColor == color
                              ? Colors.white
                              : Colors.transparent,
                          width: 3,
                        ),
                        boxShadow: selectedColor == color
                            ? [
                                BoxShadow(
                                  color: color.withValues(alpha: 0.5),
                                  blurRadius: 8,
                                )
                              ]
                            : null,
                      ),
                    ),
                  ),
                )),
          ],
        ),
      ),
    );
  }

  String _shapeLabel(ShapeType type) {
    switch (type) {
      case ShapeType.circle:
        return '● Circle';
      case ShapeType.square:
        return '■ Square';
      case ShapeType.triangle:
        return '▲ Triangle';
    }
  }
}

class _Canvas extends StatelessWidget {
  final List<DrawingShape> shapes;
  final ValueChanged<Offset> onAddShape;
  final bool isPlaying;

  const _Canvas({
    required this.shapes,
    required this.onAddShape,
    required this.isPlaying,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTapDown:
          isPlaying ? null : (details) => onAddShape(details.localPosition),
      onPanUpdate:
          isPlaying ? null : (details) => onAddShape(details.localPosition),
      child: Container(
        color: Theme.of(context).colorScheme.surfaceContainerLowest,
        child: Stack(
          children: [
            CustomPaint(
              size: Size.infinite,
              painter: _GridPainter(
                color: Theme.of(context).colorScheme.outlineVariant,
              ),
            ),
            ...shapes.map((shape) => Positioned(
                  left: shape.position.dx - 25,
                  top: shape.position.dy - 25,
                  child: _ShapeWidget(shape: shape),
                )),
            if (shapes.isEmpty && !isPlaying)
              Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(
                      Icons.gesture,
                      size: 48,
                      color: Theme.of(context).colorScheme.outline,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Tap or drag to draw shapes',
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.outline,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Hit Record to capture a session',
                      style: TextStyle(
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.outline,
                      ),
                    ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _ShapeWidget extends StatelessWidget {
  final DrawingShape shape;

  const _ShapeWidget({required this.shape});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 50,
      height: 50,
      child: CustomPaint(
        painter: _ShapePainter(type: shape.type, color: shape.color),
      ),
    );
  }
}

class _ShapePainter extends CustomPainter {
  final ShapeType type;
  final Color color;

  _ShapePainter({required this.type, required this.color});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.fill;

    switch (type) {
      case ShapeType.circle:
        canvas.drawCircle(
          Offset(size.width / 2, size.height / 2),
          size.width / 2,
          paint,
        );
        break;
      case ShapeType.square:
        canvas.drawRect(
          Rect.fromLTWH(0, 0, size.width, size.height),
          paint,
        );
        break;
      case ShapeType.triangle:
        final path = Path()
          ..moveTo(size.width / 2, 0)
          ..lineTo(size.width, size.height)
          ..lineTo(0, size.height)
          ..close();
        canvas.drawPath(path, paint);
        break;
    }
  }

  @override
  bool shouldRepaint(_ShapePainter oldDelegate) =>
      type != oldDelegate.type || color != oldDelegate.color;
}

class _GridPainter extends CustomPainter {
  final Color color;

  _GridPainter({required this.color});

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color.withValues(alpha: 0.3)
      ..strokeWidth = 1;

    const spacing = 20.0;

    for (double x = 0; x < size.width; x += spacing) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y < size.height; y += spacing) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(_GridPainter oldDelegate) => color != oldDelegate.color;
}
