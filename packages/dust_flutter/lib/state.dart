/// Typed Flutter state management annotations and runtime for Dust.
library;

export 'dart:async' show scheduleMicrotask, StreamSubscription;
export 'package:flutter/widgets.dart'
    show BuildContext, InheritedModel, State, StatefulWidget, Widget;

export 'src/state/annotations.dart';
export 'src/state/view_model.dart';
