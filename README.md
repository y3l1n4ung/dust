# 🌪️ Dust

**Extreme speed Dart code generation.**

[![CI](https://github.com/y3l1n4ung/dust/actions/workflows/ci.yml/badge.svg)](https://github.com/y3l1n4ung/dust/actions)
[![Release](https://img.shields.io/github/v/release/y3l1n4ung/dust?logo=github&color=blue)](https://github.com/y3l1n4ung/dust/releases)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

Dust is a high-performance alternative to `build_runner`. It replaces slow, serial generation with a multi-threaded Rust engine that is **10x–50x faster**.

> [!IMPORTANT]
> **Tired of waiting for your code to generate?** Dust is built to handle the largest Flutter projects with near-instant rebuilds.

---

## ✨ Why Dust?

- 🚀 **Performance:** Written in Rust. Generates thousands of files in seconds. 
- 📦 **Simple Setup:** Single-binary CLI. No complex dependency management in `pubspec.yaml`.
- 🧩 **All-in-One:** Data classes, JSON, HTTP clients, Routing, and i18n in one unified tool.
- 🔄 **Incremental:** Intelligent watch mode only rebuilds the specific files you edited.
- 🛡️ **Type Safe:** Advanced validation catches errors before you even run your app.

---

## 🏗️ Supported Features

| Feature | Description | Documentation |
| :--- | :--- | :--- |
| **Data Classes** | `ToString`, `Eq`, `HashCode`, and `CopyWith` generation. | [Read Guide →](docs/usage/derive.md) |
| **JSON Serialization** | Blazing fast JSON encode/decode with support for renames and custom codecs. | [Read Guide →](docs/usage/serde.md) |
| **HTTP Client** | Type-safe, Dio-backed API client generation from annotations. | [Read Guide →](docs/usage/http.md) |
| **Routing** | Boilerplate-free Navigator 2.0 routing with typed parameters. | [Read Guide →](docs/usage/routing.md) |
| **State Management** | Lightweight, high-performance state containers with action generation. | [Read Guide →](docs/usage/state.md) |
| **i18n** | Extraction-first translation system with chunked generation. | _(Coming Soon)_ |

---

## 🚀 Quick Start

### 1. Install the CLI

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/y3l1n4ung/dust/main/install.ps1 | iex
```

### 2. Add Annotations
```dart
import 'package:derive_annotation/derive_annotation.dart';
import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'user.g.dart';

@Derive([ToString(), CopyWith(), Serialize()])
class User with _$User {
  final String name;
  const User(this.name);
}
```

### 3. Build
```bash
dust build
```

---

## 🛠️ Commands

| Command | Description |
| :--- | :--- |
| `dust build` | Run a full project generation. |
| `dust watch` | High-performance file watcher for instant rebuilds. |
| `dust check` | CI mode: Verifies if generated files are up to date. |
| `dust clean` | Clears all generated files and persistent caches. |

---

## 🤝 Contributing

Dust is open-source and we welcome all contributors!

- **Found a bug?** [Open an issue](https://github.com/y3l1n4ung/dust/issues)
- **Rust/Dart Setup:** [See CONTRIBUTING.md](CONTRIBUTING.md)
- **Architecture:** [Read the Developer Guide](docs/developer.md)

---

## 📜 License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 [Ye Lin Aung](https://github.com/y3l1n4ung).
