import 'package:dust_flutter/state.dart';
import 'package:dust_dart/serde.dart';

import '../data/shopping_repository.dart';
import '../services/storage_service.dart';

part 'app_view_model.g.dart';

enum AppBackendMode { liveFakeStore }

@Derive([ToString(), Eq(), CopyWith()])
class AppState with _$AppState {
  const AppState({this.backendMode = AppBackendMode.liveFakeStore});

  final AppBackendMode backendMode;
}

final class AppViewModelArgs extends ViewModelArgs {
  const AppViewModelArgs({
    required this.repository,
    required this.storage,
    super.observer,
  });

  final ShoppingRepository repository;
  final StorageService storage;
}

@ViewModel(state: AppState, args: AppViewModelArgs)
class AppViewModel extends $AppViewModel {
  AppViewModel(super.args);
}
