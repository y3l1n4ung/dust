/// Typed Flutter state management annotations and runtime for Dust.
library;

export 'dart:async' show Future, StreamSubscription, scheduleMicrotask;
export 'package:flutter/widgets.dart'
    show
        BuildContext,
        ErrorDescription,
        FlutterError,
        FlutterErrorDetails,
        InheritedWidget,
        State,
        StatefulWidget,
        StatelessWidget,
        Widget;

export 'src/state/annotations.dart';
export 'src/state/view_model.dart';
