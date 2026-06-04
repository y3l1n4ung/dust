# Contributing to Dust

Thank you for your interest in contributing to Dust! This project aims to provide the fastest Dart code generation engine, and we welcome contributions that help us maintain high performance and code quality.

---

## 🚀 Getting Started

To contribute to Dust, you'll need both the Rust and Dart toolchains installed on your machine.

### Prerequisites
- **Rust:** Latest stable version (via [rustup](https://rustup.rs/))
- **Dart:** SDK 3.0 or later (via [dart.dev](https://dart.dev/get-dart))
- **Flutter:** (Optional) For running mobile examples

### Initial Setup
1. Fork and clone the repository.
2. Initialize the project:
   ```bash
   # Install Rust dependencies and run tests
   cargo test --workspace --quiet
   
   # Fetch Dart dependencies
   flutter pub get
   ```

---

## 📂 Repository Map

| Directory | Purpose |
| :--- | :--- |
| `crates/` | Rust crates for parsing, IR, plugins, emitter, and the CLI. |
| `packages/` | Publishable Dart annotation and runtime packages. |
| `examples/` | Real-world example projects and scale/perf fixtures. |
| `docs/` | Comprehensive usage guides and internal architecture docs. |

---

## 🛠️ Development Workflow

### Local Build Commands
Generate the **Product Showcase**:
```bash
cargo run -p dust_cli -- build --root examples/product_showcase
```

Generate the **Benchmark Project** (Scale Testing):
```bash
cd examples/benchmark_project
./generate.sh --count 5000
cd ../..
cargo run -p dust_cli -- build --root examples/benchmark_project
```

### Verification & Testing
Before submitting a PR, ensure all checks pass:

```bash
# Rust checks
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Dart checks (example)
cd packages/derive_annotation && flutter analyze && flutter test
```

---

## ⚖️ Engineering Rules

To keep Dust fast and maintainable, please follow these core principles:

- **Single Pipeline:** Use the shared 4-pass build pipeline. Do not add plugin-specific hacks to the driver.
- **Shared Analysis:** Use Pass 2 (`collect_workspace_analysis`) for cross-file state.
- **No Panics:** Avoid `.expect()` or `.unwrap()` in plugin code. Use `Diagnostic::error` instead.
- **Deterministic:** Generated output must be deterministic and byte-for-byte identical on every run.
- **Small & Clean:** Keep generated Dart code small, readable, and analyzer-clean.

---

## 📬 Pull Requests

1.  **Conventional Commits:** Use prefixes like `feat:`, `fix:`, `perf:`, or `docs:`.
2.  **Small Scopes:** Prefer smaller, focused PRs over giant refactors.
3.  **Include Commands:** In your PR description, list the exact commands you used to verify your changes.
4.  **Performance Impact:** If your change affects the build path, report both **Cold** and **Warm** build times using the benchmark project.

---

## 🐞 Reporting Issues

- **Bugs:** Provide a minimal Dart snippet that reproduces the issue.
- **Features:** Describe the desired Dart API and the expected generated code.
- **Performance:** Include environment details (OS, CPU, RAM) and the scale of the project (file count).

---

## 📜 License

By contributing to Dust, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
