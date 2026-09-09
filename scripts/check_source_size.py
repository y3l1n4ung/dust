#!/usr/bin/env python3
"""Keep hand-written Dust source files small.

CONTRIBUTING's "Small & Clean" rule has no teeth without a number. This check
supplies one: every hand-written production source file stays under
``LINE_LIMIT`` lines.

Every hand-written Rust, Dart, Python and shell file counts, wherever it lives:
crate and package source, their tests, the example and fixture projects, and
the scripts in this directory. Tests are included because a 2,000-line test
file is as hard to read as a 2,000-line module; splitting one by what it covers
costs nothing and makes a failure easier to place.

Generated Dart is the exception, recognised by the `.g.dart` suffix or by the
header the emitter writes. It is the emitter's output rather than something a
person reads or edits, and its size is a fact about the emitter.

``BASELINE`` is empty, and is meant to stay that way. It exists so a file can
be pinned at its current size while it is being split rather than blocking a
branch. A pin that is no longer needed is itself an error, so nothing can be
parked there and forgotten.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
import unittest
from pathlib import Path

LINE_LIMIT = 300

# Temporary exemptions, pinned at the size the file had when it was added here.
# A pinned file may only shrink, and its entry must go once it is under the
# limit. Empty is the intended state; adding to it needs a reason.
BASELINE: dict[str, int] = {}

# Every hand-written source file, wherever it lives. Only generated Dart is
# excluded, by the `.g.dart` suffix.
SOURCE_GLOBS = (
    ("crates", "*/src/**/*.rs"),
    ("crates", "*/tests/**/*.rs"),
    ("crates", "*/benches/**/*.rs"),
    ("packages", "*/lib/**/*.dart"),
    ("packages", "*/test/**/*.dart"),
    ("packages", "*/example/**/*.dart"),
    ("examples", "*/lib/**/*.dart"),
    ("examples", "*/test/**/*.dart"),
    ("fixtures", "*/lib/**/*.dart"),
    ("fixtures", "*/test/**/*.dart"),
    ("scripts", "**/*.py"),
    ("scripts", "**/*.sh"),
)


def source_files(root: Path) -> list[Path]:
    """Returns the hand-written production sources under `root`, sorted."""
    found: set[Path] = set()
    for directory, pattern in SOURCE_GLOBS:
        for path in (root / directory).glob(pattern):
            if path.is_file() and not _is_excluded(path):
                found.add(path)
    return sorted(found)


def _is_excluded(path: Path) -> bool:
    """Returns whether `path` is generated output rather than hand-written.

    The `.g.dart` suffix covers most of it, but the emitter also writes test
    files under `test/generated/`, which carry the same header. Splitting one
    of those achieves nothing: the next `dust build` writes it back.
    """
    if path.name.endswith(".g.dart"):
        return True
    with path.open(encoding="utf-8", errors="replace") as handle:
        return "GENERATED CODE - DO NOT MODIFY BY HAND" in handle.readline()


def line_count(path: Path) -> int:
    """Returns the number of lines in `path`.

    Every line counts, inline ``#[cfg(test)]`` modules included. A file is as
    long as it reads, and an exemption that shrinks the number without
    shrinking the file only moves the problem somewhere the check cannot see.
    """
    return len(path.read_text(encoding="utf-8").splitlines())


def check(root: Path) -> list[str]:
    """Returns one message per source file that breaks the size rule."""
    violations: list[str] = []
    unused_baseline = dict(BASELINE)

    for path in source_files(root):
        relative = path.relative_to(root).as_posix()
        lines = line_count(path)
        pinned = unused_baseline.pop(relative, None)

        if pinned is None:
            if lines > LINE_LIMIT:
                violations.append(
                    f"{relative}: {lines} lines exceeds the {LINE_LIMIT}-line limit; "
                    "split it into focused modules"
                )
            continue

        if lines > pinned:
            violations.append(
                f"{relative}: {lines} lines grew past its pinned baseline of {pinned}; "
                f"this file may only shrink toward the {LINE_LIMIT}-line limit"
            )
        elif lines <= LINE_LIMIT:
            violations.append(
                f"{relative}: {lines} lines is now within the {LINE_LIMIT}-line limit; "
                "remove its entry from BASELINE in scripts/check_source_size.py"
            )

    for relative in unused_baseline:
        violations.append(
            f"{relative}: pinned in BASELINE but is not a scanned source file; "
            "remove its entry from scripts/check_source_size.py"
        )

    return violations


class CheckSourceSizeTests(unittest.TestCase):
    """Self-tests, so the check is exercised without a real workspace."""

    def setUp(self) -> None:
        self._baseline = dict(BASELINE)
        self.addCleanup(self._restore)

    def _restore(self) -> None:
        BASELINE.clear()
        BASELINE.update(self._baseline)

    @staticmethod
    def _workspace(tmp: Path, relative: str, lines: int) -> Path:
        path = tmp / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("// line\n" * lines, encoding="utf-8")
        return path

    def test_small_file_passes(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "crates/demo/src/lib.rs", LINE_LIMIT)
            self.assertEqual(check(root), [])

    def test_oversized_file_is_reported(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "crates/demo/src/lib.rs", LINE_LIMIT + 1)
            violations = check(root)
            self.assertEqual(len(violations), 1)
            self.assertIn("exceeds", violations[0])

    def test_generated_dart_is_skipped(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "packages/demo/lib/big.g.dart", LINE_LIMIT + 100)
            self.assertEqual(check(root), [])

    def test_generated_header_is_skipped(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            path = root / "packages/demo/test/generated/api_test.dart"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(
                "// GENERATED CODE - DO NOT MODIFY BY HAND\n"
                + "// line\n" * (LINE_LIMIT + 100),
                encoding="utf-8",
            )
            self.assertEqual(check(root), [])

    def test_tests_are_scanned_too(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "crates/demo/tests/big.rs", LINE_LIMIT + 1)
            self._workspace(root, "packages/demo/test/big_test.dart", LINE_LIMIT + 1)
            self._workspace(root, "crates/demo/src/feature/tests.rs", LINE_LIMIT + 1)
            self._workspace(root, "examples/demo/lib/big.dart", LINE_LIMIT + 1)
            self.assertEqual(len(check(root)), 4)

    def test_inline_test_module_counts(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            path = root / "crates/demo/src/lib.rs"
            path.parent.mkdir(parents=True, exist_ok=True)
            body = "// line\n" * 100
            tests = "#[cfg(test)]\nmod tests {\n" + "    // case\n" * LINE_LIMIT + "}\n"
            path.write_text(body + tests, encoding="utf-8")

            self.assertEqual(line_count(path), 100 + LINE_LIMIT + 3)
            self.assertEqual(len(check(root)), 1)

    def test_pinned_file_may_shrink_but_not_grow(self) -> None:
        BASELINE.clear()
        BASELINE["crates/demo/src/lib.rs"] = LINE_LIMIT + 100
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            path = self._workspace(root, "crates/demo/src/lib.rs", LINE_LIMIT + 50)
            self.assertEqual(check(root), [])

            path.write_text("// line\n" * (LINE_LIMIT + 101), encoding="utf-8")
            violations = check(root)
            self.assertEqual(len(violations), 1)
            self.assertIn("grew past its pinned baseline", violations[0])

    def test_pin_must_be_removed_once_the_file_is_small(self) -> None:
        BASELINE.clear()
        BASELINE["crates/demo/src/lib.rs"] = LINE_LIMIT + 100
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "crates/demo/src/lib.rs", LINE_LIMIT)
            violations = check(root)
            self.assertEqual(len(violations), 1)
            self.assertIn("remove its entry from BASELINE", violations[0])

    def test_pin_for_a_deleted_file_is_reported(self) -> None:
        BASELINE.clear()
        BASELINE["crates/demo/src/gone.rs"] = LINE_LIMIT + 100
        with tempfile.TemporaryDirectory() as name:
            violations = check(Path(name))
            self.assertEqual(len(violations), 1)
            self.assertIn("is not a scanned source file", violations[0])


def main() -> int:
    """Runs the size check, or the self-tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run this script's own tests instead of checking the workspace",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="repository root to scan",
    )
    args = parser.parse_args()

    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(CheckSourceSizeTests)
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        return 0 if result.wasSuccessful() else 1

    violations = check(args.root)
    if violations:
        print("error: hand-written source files must stay small", file=sys.stderr)
        for violation in violations:
            print(f"  {violation}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
