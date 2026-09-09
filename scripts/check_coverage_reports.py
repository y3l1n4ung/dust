#!/usr/bin/env python3
"""Check that Sonar is given a coverage report for every package.

A package the gate tests but Sonar never receives a report for counts as
entirely uncovered. That is not hypothetical: `dust_db_postgres` was measured
at 100% by the PostgreSQL job, uploaded nowhere, and dragged the Quality Gate
on new code to 75.6% until the report path was added.

Nothing failed at the time. The package list and the report list simply
disagreed, and only Sonar noticed, hours later and in a comment. This makes the
two lists agree, or fails.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

# `<toolchain>-<package>` is the artifact naming the CI jobs already use.
REPORT_PATH = ".sonar/coverage/coverage-{toolchain}-{package}/lcov.info"


def workspace_packages(root: Path, toolchain: str) -> list[str]:
    """Returns the packages `scripts/workspace_packages.sh` reports."""
    result = subprocess.run(
        [str(root / "scripts/workspace_packages.sh"), toolchain],
        capture_output=True,
        text=True,
        check=True,
    )
    return [line.split("/")[-1] for line in result.stdout.split()]


def configured_reports(properties: str) -> set[str]:
    """Returns the Dart lcov report paths Sonar is configured with."""
    match = re.search(r"(?m)^sonar\.dart\.lcov\.reportPaths=(.+)$", properties)
    if not match:
        return set()
    return {path.strip() for path in match.group(1).split(",") if path.strip()}


def check(root: Path) -> list[str]:
    """Returns one message per package Sonar would see as uncovered."""
    properties = (root / "sonar-project.properties").read_text(encoding="utf-8")
    configured = configured_reports(properties)

    missing = []
    for toolchain in ("dart", "flutter"):
        for package in workspace_packages(root, toolchain):
            expected = REPORT_PATH.format(toolchain=toolchain, package=package)
            if expected not in configured:
                missing.append(
                    f"`{package}` is tested by the {toolchain} gate but "
                    f"sonar-project.properties lists no `{expected}`, so every "
                    "line in it counts as uncovered"
                )
    return missing


class CheckCoverageReportsTests(unittest.TestCase):
    """Self-tests, so the check is exercised without the real repository."""

    def test_reads_the_configured_report_paths(self) -> None:
        properties = "sonar.dart.lcov.reportPaths=a/lcov.info, b/lcov.info\n"
        self.assertEqual(configured_reports(properties), {"a/lcov.info", "b/lcov.info"})

    def test_no_report_paths_at_all_is_empty(self) -> None:
        self.assertEqual(configured_reports("sonar.projectKey=x\n"), set())

    def test_a_package_with_no_report_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "scripts").mkdir()
            script = root / "scripts/workspace_packages.sh"
            script.write_text(
                '#!/usr/bin/env bash\n[ "$1" = dart ] && echo packages/demo\nexit 0\n'
            )
            script.chmod(0o755)
            (root / "sonar-project.properties").write_text(
                "sonar.dart.lcov.reportPaths=.sonar/coverage/coverage-dart-other/lcov.info\n"
            )
            problems = check(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("demo", problems[0])


def main() -> int:
    """Runs the check, or the self-tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()

    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(CheckCoverageReportsTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1

    missing = check(args.root)
    if missing:
        print("error: Sonar is missing coverage for packages the gate tests", file=sys.stderr)
        for problem in missing:
            print(f"  {problem}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
