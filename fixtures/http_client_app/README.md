# HTTP Client Analyzer Fixture

Small fixture that proves generated HTTP clients and generated request tests analyze in a real Dart package.

```sh
(cd fixtures/http_client_app && dart pub get)
cargo run --quiet -p dust_cli -- build --clean --root fixtures/http_client_app
(cd fixtures/http_client_app && dart analyze --fatal-infos)
(cd fixtures/http_client_app && dart test)
```
