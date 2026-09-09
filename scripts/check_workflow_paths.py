#!/usr/bin/env python3
"""Check that files named in CI workflows still exist.

A workflow that runs `dart test test/example/examples_test.dart` keeps working
until someone splits that file, and then fails on CI rather than locally. That
happened: the dust_server example suite became eleven suites, and the only job
that serves PostgreSQL over HTTP spent a full CI round-trip reporting
`Does not exist`.

Paths are read relative to the repository root and to each step's
`working-directory`, since workflow steps commonly set one.
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
import unittest
from pathlib import Path

# Repo-relative looking paths with a source-file extension. Anything without an
# extension is likely a directory or a shell word, and is left alone.
PATH_PATTERN = re.compile(
    r"(?<![\w./-])((?:crates|packages|examples|fixtures|scripts|docs|test|lib)"
    r"/[\w./-]+\.(?:dart|rs|py|sh|toml|yaml|yml|json|jinja))"
)
WORKING_DIRECTORY = re.compile(r"working-directory:\s*(\S+)")


def workflow_files(root: Path) -> list[Path]:
    """Returns the workflow files to scan."""
    return sorted((root / ".github/workflows").glob("*.yml"))


def check(root: Path) -> list[str]:
    """Returns one message per workflow path that does not resolve."""
    problems = []
    for workflow in workflow_files(root):
        working_directory = ""
        for number, line in enumerate(workflow.read_text(encoding="utf-8").splitlines(), 1):
            found = WORKING_DIRECTORY.search(line)
            if found:
                working_directory = found.group(1)
            for candidate in PATH_PATTERN.findall(line):
                if (root / candidate).exists():
                    continue
                if working_directory and (root / working_directory / candidate).exists():
                    continue
                problems.append(
                    f"{workflow.relative_to(root)}:{number} names `{candidate}`, "
                    "which does not exist"
                )
    return problems


class CheckWorkflowPathsTests(unittest.TestCase):
    """Self-tests, so the check is exercised without the real workflows."""

    @staticmethod
    def _workflow(root: Path, body: str) -> None:
        directory = root / ".github/workflows"
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "ci.yml").write_text(body, encoding="utf-8")

    def test_a_missing_path_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workflow(root, "        run: dart test test/example/gone_test.dart\n")
            problems = check(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("gone_test.dart", problems[0])

    def test_an_existing_path_passes(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "test/example").mkdir(parents=True)
            (root / "test/example/here_test.dart").write_text("", encoding="utf-8")
            self._workflow(root, "        run: dart test test/example/here_test.dart\n")
            self.assertEqual(check(root), [])

    def test_a_path_relative_to_working_directory_passes(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "packages/demo/test").mkdir(parents=True)
            (root / "packages/demo/test/here_test.dart").write_text("", encoding="utf-8")
            self._workflow(
                root,
                "        working-directory: packages/demo\n"
                "        run: dart test test/here_test.dart\n",
            )
            self.assertEqual(check(root), [])

    def test_a_directory_without_an_extension_is_left_alone(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workflow(root, "        run: dart test test/example\n")
            self.assertEqual(check(root), [])


def main() -> int:
    """Runs the check, or the self-tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()

    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(CheckWorkflowPathsTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1

    problems = check(args.root)
    if problems:
        print("error: a workflow names a file that does not exist", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
