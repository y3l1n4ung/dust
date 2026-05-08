# State Management Prototype

Flutter prototype showing how to build a small app around Dust-generated
models and HTTP clients without an additional state-management package.

This example uses:

- `derive_annotation` for immutable `copyWith`, equality, and debug output
- `derive_serde_annotation` for JSON models
- `dust_http_client_annotation` for the generated API client
- Flutter built-ins for state wiring: `ValueNotifier`,
  `InheritedNotifier`, and `ValueListenableBuilder`

The structure stays MVVM-focused:

- models: immutable Dust-generated state and transport types
- view_models: async orchestration and user actions
- views: Flutter widgets that bind to one view model
- data: repository + generated API access

## Run

```bash
cd examples/state_management_prototype
flutter pub get
cargo run -p dust_cli -- build --root .
flutter run
```

## Structure

- `lib/models` contains Dust-generated app and transport models
- `lib/api` contains the generated JSONPlaceholder client
- `lib/view_models` contains the manual MVVM view model
- `lib/views` contains the prototype UI inspired by `external_reference`

## Notes

- The app uses JSONPlaceholder for live data.
- State updates stay local and immutable by copying Dust-generated models.
- The example is intentionally small enough to read end-to-end.
