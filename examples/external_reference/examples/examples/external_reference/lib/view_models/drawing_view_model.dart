import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';
import 'package:flutter/material.dart';

part 'drawing_view_model.ease.dart';

/// A shape on the canvas.
@immutable
class DrawingShape {
  final String id;
  final ShapeType type;
  final Offset position;
  final Color color;

  const DrawingShape({
    required this.id,
    required this.type,
    required this.position,
    required this.color,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'type': type.index,
        'x': position.dx,
        'y': position.dy,
        'color': color.toARGB32(),
      };

  factory DrawingShape.fromJson(Map<String, dynamic> json) => DrawingShape(
        id: json['id'],
        type: ShapeType.values[json['type']],
        position: Offset(json['x'].toDouble(), json['y'].toDouble()),
        color: Color(json['color']),
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DrawingShape &&
          id == other.id &&
          type == other.type &&
          position == other.position &&
          color == other.color;

  @override
  int get hashCode => Object.hash(id, type, position, color);
}

enum ShapeType { circle, square, triangle }

/// Drawing canvas state.
@immutable
class DrawingState {
  final List<DrawingShape> shapes;
  final ShapeType selectedType;
  final Color selectedColor;

  const DrawingState({
    this.shapes = const [],
    this.selectedType = ShapeType.circle,
    this.selectedColor = Colors.blue,
  });

  Map<String, dynamic> toJson() => {
        'shapes': shapes.map((s) => s.toJson()).toList(),
        'selectedType': selectedType.index,
        'selectedColor': selectedColor.toARGB32(),
      };

  factory DrawingState.fromJson(Map<String, dynamic> json) => DrawingState(
        shapes: (json['shapes'] as List)
            .map((s) => DrawingShape.fromJson(s))
            .toList(),
        selectedType: ShapeType.values[json['selectedType']],
        selectedColor: Color(json['selectedColor']),
      );

  DrawingState copyWith({
    List<DrawingShape>? shapes,
    ShapeType? selectedType,
    Color? selectedColor,
  }) {
    return DrawingState(
      shapes: shapes ?? this.shapes,
      selectedType: selectedType ?? this.selectedType,
      selectedColor: selectedColor ?? this.selectedColor,
    );
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DrawingState &&
          _listEquals(shapes, other.shapes) &&
          selectedType == other.selectedType &&
          selectedColor == other.selectedColor;

  @override
  int get hashCode => Object.hash(
        Object.hashAll(shapes),
        selectedType,
        selectedColor,
      );

  static bool _listEquals<T>(List<T> a, List<T> b) {
    if (identical(a, b)) return true;
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}

/// Drawing ViewModel with undo/redo support.
@ease
class DrawingViewModel extends StateNotifier<DrawingState> {
  DrawingViewModel() : super(const DrawingState());

  int _counter = 0;

  void addShape(Offset position) {
    final shape = DrawingShape(
      id: 'shape_${++_counter}',
      type: state.selectedType,
      position: position,
      color: state.selectedColor,
    );

    setState(
      state.copyWith(shapes: [...state.shapes, shape]),
      action: 'addShape',
    );
  }

  void clearCanvas() {
    if (state.shapes.isEmpty) return;
    setState(state.copyWith(shapes: []), action: 'clearCanvas');
  }

  void selectType(ShapeType type) {
    setState(state.copyWith(selectedType: type), action: 'selectType');
  }

  void selectColor(Color color) {
    setState(state.copyWith(selectedColor: color), action: 'selectColor');
  }
}
