# State Management Specification (Native Flutter MVVM + Dust Optimization)

## Implementation Status

- Generated scopes use `InheritedModel<Object>` with typed aspect enums.
- `watchXxxViewModel()` returns a smart proxy that registers dependencies when state properties are read.
- Imported unannotated state files are inspected for `final` state fields, so normal app state classes get granular aspects without extra annotations.
- `updateShouldNotifyDependent` now emits a switch-based dispatcher instead of a linear field chain per dependency.
- `readXxxViewModel()` remains non-subscribing for event handlers and mutations.

## 1. Core Philosophy: "Transparent Performance"
The generated state management system uses a "Hidden Base Class" pattern to provide standard Flutter semantics (`watch`, `read`, `listen`) while delivering optimized O(1) rebuild performance through automatic aspect registration.

- **Hidden Base Class**: Your ViewModels extend a generated `$_XxxViewModel` class. This class manages the state (`ValueNotifier`), repository injection, and effect dispatching.
- **Smart Proxy**: When you `watch` a ViewModel, you receive a generated Proxy. This proxy intercepts property access to automatically register dependencies with an `InheritedModel`.
- **Zero-Boilerplate Business Logic**: Your ViewModel code focuses purely on logic, with helpers like `emit()` and `state` instead of `.value`.

---

## 2. End-to-End Example

### A. The ViewModel (`user_view_model.dart`)
```dart
@ViewModel()
class UserViewModel extends $_UserViewModel {
  UserViewModel(super.repository) : super(const UserState());

  Future<void> refresh() async {
    emit(state.copyWith(isLoading: true)); // Provided by base class
    final user = await repository.fetchUser(id: 1);
    emit(state.copyWith(user: user, isLoading: false));
  }
}
```

### B. UI Usage (`profile_page.dart`)
```dart
class ProfilePage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    // Returns a Smart Proxy _$UserViewModelProxy
    final vm = context.watchUserViewModel(); 

    return Column(
      children: [
        // Automatically registers for the 'name' aspect only!
        Text(vm.name), 
        
        // Accessing 'isLoading' registers for that aspect too.
        if (vm.isLoading) const CircularProgressIndicator(),
        
        ElevatedButton(
          onPressed: () => context.readUserViewModel().refresh(),
          child: const Text('Refresh'),
        ),
      ],
    );
  }
}
```

---

## 3. The Generated "Hidden" Layer (`user_view_model.g.dart`)

### A. The Base Class (`$_UserViewModel`)
```dart
abstract class $_UserViewModel extends ValueNotifier<UserState> {
  $_UserViewModel(this.repository, super.initialValue);
  
  final UserRepository repository;
  
  UserState get state => value;
  void emit(UserState next) => value = next;
  
  // Internal effect management...
}
```

### B. The Smart Proxy (`_$UserViewModelProxy`)
```dart
class _$UserViewModelProxy {
  final BuildContext _context;
  final UserViewModel _vm;

  _$UserViewModelProxy(this._context, this._vm);

  String get name {
    UserViewModelScope.of(_context, aspect: 'name');
    return _vm.state.name;
  }

  bool get isLoading {
    UserViewModelScope.of(_context, aspect: 'isLoading');
    return _vm.state.isLoading;
  }
}
```

### C. The Scope Widget (`UserViewModelScope`)
Uses `InheritedModel` to handle the granular aspects registered by the Proxy.

```dart
class UserViewModelScope extends StatefulWidget {
  // ... boilerplate ...
  
  static UserViewModel _getRaw(BuildContext context) {
    return InheritedModel.inheritFrom<_UserViewModelInherited>(context)!.viewModel;
  }

  static UserViewModel of(BuildContext context, {Object? aspect}) {
    return InheritedModel.inheritFrom<_UserViewModelInherited>(context, aspect: aspect)!.viewModel;
  }
}

class _UserViewModelInherited extends InheritedModel<Object> {
  final UserViewModel viewModel;
  final UserState _state;

  _UserViewModelInherited({required this.viewModel, required super.child}) 
    : _state = viewModel.value;

  @override
  bool updateShouldNotify(_UserViewModelInherited oldWidget) => true;

  @override
  bool updateShouldNotifyDependent(_UserViewModelInherited oldWidget, Set<Object> dependencies) {
    // Granular O(1) check: Did any of the properties accessed by the widget actually change?
    for (final aspect in dependencies) {
      if (aspect == 'name' && _state.name != oldWidget._state.name) return true;
      if (aspect == 'isLoading' && _state.isLoading != oldWidget._state.isLoading) return true;
    }
    return false;
  }
}
```
