# Dust Stress Project

Large local Dart fixture project for Dust build/watch scale testing.

## What this project is for

- generate a large number of annotated Dart source files
- exercise different model shapes Dust already supports
- benchmark `dust build` and `dust watch` on a large input set

## Generate 5000 source files

```bash
cd dust/examples/stress_project
dart pub get
./generate.sh
```

This writes the generated sources into `lib/generated_models/`.

## Run Dust

```bash
cd /Users/yelinaung/Projects/Coursera/RustProjects/dart_codegeneration_engine/dust
cargo run -p dust_cli -- build --root /Users/yelinaung/Projects/Coursera/RustProjects/dart_codegeneration_engine/dust/examples/stress_project
```

## Notes

- `lib/generated_models/` is ignored by Git through the local `.gitignore`
- the same folder is excluded from Dart analyzer through `analysis_options.yaml`
- the generator emits multiple model shapes, not only one repeated class form

