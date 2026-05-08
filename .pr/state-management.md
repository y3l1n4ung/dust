# State Management Specification (Native Flutter MVVM)

## 1. Core Philosophy: "Native Flutter, Rust CLI Powered"
The generated state management system uses standard Flutter patterns to feel like a first-party part of the framework. The Rust CLI handles the code generation for precise rebuilds (O(1) performance), while you manage your architecture using standard Flutter composition.

- **Zero-Config Smart Access**: `context.userViewModel` is the single entry point. It automatically differentiates between a **watch** (in `build`) and a **read** (in callbacks).
- **Explicit Scoping**: Use generated `UserViewModelScope` widgets to provide ViewModels to any sub-tree. This gives you full control over lifecycle and DI.
- **Lifecycle Managed**: `UserViewModelScope` is a `StatefulWidget` under the hood; it creates your ViewModel and ensures it is `dispose()`'d exactly when the scope is unmounted.

---

## 2. End-to-End Example: Production Quality

### A. The ViewModel & State (Standard ValueNotifier)
```dart
import 'package:flutter/foundation.dart';
import 'package:meta/meta.dart';

part 'user_view_model.g.dart';

@immutable
class UserState {
  final String name;
  final bool isLoading;

  const UserState({required this.name, this.isLoading = false});

  UserState copyWith({String? name, bool? isLoading}) => 
    UserState(name: name ?? this.name, isLoading: isLoading ?? this.isLoading);
}

@ViewModel()
class UserViewModel extends ValueNotifier<UserState> {
  final UserRepository repository;

  UserViewModel(this.repository) : super(const UserState(name: 'Guest'));

  Future<void> refresh() async {
    value = value.copyWith(isLoading: true);
    final newName = await repository.fetchName();
    value = value.copyWith(name: newName, isLoading: false);
    notifyEffect(const ProfileUpdatedEffect());
  }
}
```

### B. UI Usage (Manual Scoping & Smart Access)
Wrap your screen or component in the generated `UserViewModelScope`.

```dart
class ProfileScreen extends StatelessWidget {
  const ProfileScreen({super.key});

  @override
  Widget build(BuildContext context) {
    // 1. Provide the ViewModel and its dependencies
    return UserViewModelScope(
      create: (context) => UserViewModel(UserRepositoryImpl()),
      child: const _ProfileBody(),
    );
  }
}

class _ProfileBody extends StatelessWidget {
  const _ProfileBody();

  @override
  Widget build(BuildContext context) {
    // 2. Listen for effects
    context.userViewModel.onEffect((e) => showSnackbar(context, 'Updated!'));

    return Scaffold(
      body: Center(
        // 3. SMART ACCESS: Automatically registers O(1) aspect rebuild for 'name'
        child: Text('Hello, ${context.userViewModel.name}'),
      ),
    );
  }
}
```

---

## 3. Observability: Telemetry and Logging
Provide an optional `StateObserver` to any Scope to monitor its transitions.

```dart
UserViewModelScope(
  create: (context) => UserViewModel(repo),
  observer: AppLogger(), // Optional observer for this specific scope
  child: ...,
)
```

---

## 4. The Generated Logic (Internal Flutter Implementation)

This is what the CLI generates to make the system feel native and performant.

### A. The Public Scope Widget (`UserViewModelScope`)
A standard `StatefulWidget` that manages the ViewModel's lifecycle.

```dart
class UserViewModelScope extends StatefulWidget {
  final UserViewModel Function(BuildContext context) create;
  final StateObserver? observer;
  final Widget child;

  const UserViewModelScope({
    super.key, 
    required this.create, 
    this.observer,
    required this.child,
  });

  @override
  State<UserViewModelScope> createState() => _UserViewModelScopeState();
  
  /// Internal 'of' pattern used by the Proxy
  static UserViewModel of(BuildContext context, {String? aspect}) {
    return InheritedModel.inheritFrom<_UserViewModelInherited>(context, aspect: aspect)!.vm;
  }
}

class _UserViewModelScopeState extends State<UserViewModelScope> {
  late final UserViewModel vm = widget.create(context);

  @override
  void initState() {
    super.initState();
    if (widget.observer != null) vm.attachObserver(widget.observer!);
  }

  @override
  void dispose() {
    vm.dispose(); // Native Flutter lifecycle management
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _UserViewModelInherited(vm: vm, child: widget.child);
  }
}
```

### B. The Smart Proxy (`_$UserViewModelProxy`)
Intercepts property access for precision rebuilds.

```dart
class _$UserViewModelProxy {
  final BuildContext _context;
  final UserViewModel _vm;

  const _$UserViewModelProxy(this._context, this._vm);

  String get name {
    if (_isBuilding(_context)) {
      // Registers O(1) rebuild dependency for ONLY the 'name' property
      UserViewModelScope.of(_context, aspect: 'name');
    }
    return _vm.value.name;
  }
  
  void refresh() => _vm.refresh();
  
  void onEffect(void Function(Object) callback) => _vm.registerEffectListener(_context, callback);
}
```

### C. The Context Extension
```dart
extension UserViewModelX on BuildContext {
  /// Resolves the nearest [UserViewModel] in the tree and returns a Smart Proxy.
  _$UserViewModelProxy get userViewModel {
    final vm = UserViewModelScope.of(this); 
    return _$UserViewModelProxy(this, vm);
  }
}
```
