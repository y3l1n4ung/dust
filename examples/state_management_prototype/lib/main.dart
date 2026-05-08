import 'package:dust_http_client_annotation/dust_http_client_annotation.dart';
import 'package:flutter/widgets.dart';
import 'package:state_management_prototype/app/prototype_app.dart';
import 'package:state_management_prototype/features/session/view_models/session_view_model.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/view_models/task_board_view_model.dart';
import 'package:state_management_prototype/features/theme/view_models/theme_view_model.dart';
import 'package:state_management_prototype/shared/api/prototype_api.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  final repository = LivePrototypeRepository(PrototypeApi(Dio()));

  runApp(
    ThemeViewModelScope(
      create: (_) => ThemeViewModel(),
      child: ShellViewModelScope(
        create: (_) => ShellViewModel(),
        child: SessionViewModelScope(
          create: (_) => SessionViewModel(repository)..initialize(),
          child: TaskBoardViewModelScope(
            create: (_) => TaskBoardViewModel(repository)..initialize(),
            child: const PrototypeApp(),
          ),
        ),
      ),
    ),
  );
}
