#!/usr/bin/env python3
"""Keep hand-written Dust source files small.

CONTRIBUTING's "Small & Clean" rule has no teeth without a number. This check
supplies one: every hand-written production source file stays under
``LINE_LIMIT`` lines.

Twenty-four files predate the rule. Rather than block the milestone on a
refactor of all of them, each is pinned in ``BASELINE`` at the size it had when
the check landed: it may shrink, never grow. A pin that is no longer needed is
itself an error, so the baseline cannot quietly outlive the file it excuses,
and the list only ever gets shorter.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
import unittest
from pathlib import Path

LINE_LIMIT = 300

# Files that were already over the limit when this check was added, pinned at
# the size they had then. Shrink them; do not add to this list.
BASELINE: dict[str, int] = {
    "crates/dust_driver/src/lower.rs": 781,
    "crates/dust_parser_dart_ts/src/queries.rs": 529,
    "crates/dust_parser_dart/src/surface.rs": 528,
    "crates/dust_http_client_plugin/src/plugin/emit/class.rs": 499,
    "crates/dust_workspace/src/discover.rs": 481,
    "crates/dust_plugin_derive/src/features/validate/emit.rs": 466,
    "crates/dust_parser_dart_ts/src/declarations.rs": 458,
    "crates/dust_http_client_plugin/src/plugin/emit/test_file.rs": 456,
    "crates/dust_resolver/src/resolve.rs": 454,
    "packages/dust_flutter/lib/src/state/view_model.dart": 450,
    "crates/dust_cli/src/args.rs": 442,
    "crates/dust_parser_dart_ts/src/language_gates.rs": 423,
    "crates/dust_state_plugin/src/plugin/emit/render.rs": 423,
    "crates/dust_resolver/src/serde.rs": 377,
    "crates/dust_parser_dart_ts/src/annotations/values.rs": 364,
    "crates/dust_http_client_plugin/src/plugin/parse/http.rs": 342,
    "crates/dust_resolver/src/resolve_support.rs": 331,
    "crates/dust_driver/src/result.rs": 329,
    "crates/dust_plugin_serde/src/validate.rs": 325,
    "crates/dust_plugin_serde/src/emit_class.rs": 317,
    "crates/dust_ir/src/traits.rs": 316,
    "crates/dust_db_plugin/src/plugin/emit/dao.rs": 314,
    "crates/dust_parser_dart_ts/src/i18n/lower.rs": 313,
    "crates/dust_route_plugin/src/plugin/build/mod.rs": 303,
}

# Hand-written production source. Tests and generated output are excluded: test
# files grow with the cases they cover, and generated Dart is the emitter's
# output rather than something a reviewer reads. Under `src`, that means both
# `tests.rs` modules and inline `#[cfg(test)]` blocks.
SOURCE_GLOBS = (
    ("crates", "*/src/**/*.rs"),
    ("packages", "*/lib/**/*.dart"),
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
    """Returns whether `path` is generated output or a test-only module."""
    return path.name.endswith(".g.dart") or path.name == "tests.rs"


def line_count(path: Path) -> int:
    """Returns the reviewable lines in `path`.

    Inline ``#[cfg(test)] mod tests { ... }`` blocks do not count. They are
    tests, and tests grow with the cases they cover; charging them against a
    module's budget would push authors to write fewer of them.
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    if path.suffix != ".rs":
        return len(lines)

    counted = 0
    index = 0
    while index < len(lines):
        if lines[index].strip() == "#[cfg(test)]":
            index = _skip_block(lines, index)
            continue
        counted += 1
        index += 1
    return counted


def _skip_block(lines: list[str], start: int) -> int:
    """Returns the index after the braced item that begins at `start`."""
    depth = 0
    opened = False
    for index in range(start, len(lines)):
        depth += lines[index].count("{") - lines[index].count("}")
        opened = opened or "{" in lines[index]
        if opened and depth <= 0:
            return index + 1
    return len(lines)


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

    def test_tests_are_not_scanned(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, "crates/demo/tests/big.rs", LINE_LIMIT + 100)
            self._workspace(root, "packages/demo/test/big_test.dart", LINE_LIMIT + 100)
            self._workspace(root, "crates/demo/src/feature/tests.rs", LINE_LIMIT + 100)
            self.assertEqual(check(root), [])

    def test_inline_test_module_does_not_count(self) -> None:
        BASELINE.clear()
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            path = root / "crates/demo/src/lib.rs"
            path.parent.mkdir(parents=True, exist_ok=True)
            body = "// line\n" * LINE_LIMIT
            tests = "#[cfg(test)]\nmod tests {\n" + "    // case\n" * 400 + "}\n"
            path.write_text(body + tests, encoding="utf-8")

            self.assertEqual(line_count(path), LINE_LIMIT)
            self.assertEqual(check(root), [])

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
