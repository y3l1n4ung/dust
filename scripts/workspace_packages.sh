#!/usr/bin/env bash
# Lists publishable packages from the root pubspec workspace.
#
# The lint, format, test and coverage scripts each used to carry their own copy
# of this list, so adding a package meant remembering all of them. A package
# missing from one of those lists is not analysed, not tested, and nothing says
# so. The workspace already names every package; this reads it.
#
# Usage: workspace_packages.sh dart|flutter
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

TOOLCHAIN="${1:-}"
case "$TOOLCHAIN" in
  dart | flutter) ;;
  *)
    echo "Usage: $0 dart|flutter" >&2
    exit 2
    ;;
esac

# A package that depends on Flutter has to be driven by the Flutter toolchain;
# everything else under packages/ takes the Dart one.
python3 - "$ROOT_DIR" "$TOOLCHAIN" <<'PY'
import re
import sys
from pathlib import Path

root, toolchain = Path(sys.argv[1]), sys.argv[2]
text = (root / "pubspec.yaml").read_text(encoding="utf-8")
block = re.search(r"(?ms)^workspace:\n((?:\s*-\s*\S+\n)+)", text)
if not block:
    raise SystemExit("no workspace list in pubspec.yaml")

for entry in re.findall(r"-\s*(\S+)", block.group(1)):
    if not entry.startswith("packages/"):
        continue
    pubspec = (root / entry / "pubspec.yaml").read_text(encoding="utf-8")
    needs_flutter = re.search(r"(?m)^\s+flutter:\s*$", pubspec) is not None
    if needs_flutter == (toolchain == "flutter"):
        print(entry)
PY
