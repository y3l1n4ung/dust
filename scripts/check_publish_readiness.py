#!/usr/bin/env python3
"""Check every package can actually be published before any of them is.

Automated publishing from GitHub Actions is configured per package on pub.dev,
on a page that only exists once the package does. A package that has never been
published therefore cannot be set up in advance: its first version has to go out
by hand, and only then can a tag publish it.

The release job publishes packages in order and stops at the first refusal, so
discovering this in the middle leaves earlier packages published and later ones
not — a partial release that cannot be rolled back. This runs before any of
them, so the answer arrives while nothing has happened yet.

Network-dependent, so it is a release-job check rather than part of the lint
gate. `--offline` skips the pub.dev lookups and checks only what is local.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import unittest
import urllib.error
import urllib.request
from pathlib import Path

PUB_API = "https://pub.dev/api/packages/{package}"


def publishable_packages(root: Path) -> list[Path]:
    """Returns the packages the release workflow publishes."""
    found: list[str] = []
    for toolchain in ("dart", "flutter"):
        result = subprocess.run(
            [str(root / "scripts/workspace_packages.sh"), toolchain],
            capture_output=True, text=True, check=True,
        )
        found += result.stdout.split()
    return [root / entry for entry in sorted(set(found))]


def is_published(package: str, timeout: float = 20.0) -> bool:
    """Returns whether pub.dev already hosts this package."""
    try:
        with urllib.request.urlopen(PUB_API.format(package=package), timeout=timeout):
            return True
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise
    return False


def check(root: Path, offline: bool = False) -> list[str]:
    """Returns one message per package that a tag could not publish."""
    problems = []
    for package in publishable_packages(root):
        name = package.name
        if not (package / "CHANGELOG.md").exists():
            problems.append(f"{name} has no CHANGELOG.md, which pub.dev expects")
        if offline:
            continue
        if not is_published(name):
            problems.append(
                f"{name} has never been published, so pub.dev has no page on which "
                "automated publishing could have been configured. Publish its first "
                "version by hand, then a tag can publish the rest"
            )
    return problems


class CheckPublishReadinessTests(unittest.TestCase):
    """Self-tests, which never reach the network."""

    def test_a_package_without_a_changelog_is_reported(self) -> None:
        import tempfile

        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "scripts").mkdir()
            script = root / "scripts/workspace_packages.sh"
            script.write_text(
                '#!/usr/bin/env bash\n[ "$1" = dart ] && echo packages/demo\nexit 0\n'
            )
            script.chmod(0o700)
            (root / "packages/demo").mkdir(parents=True)
            problems = check(root, offline=True)
            self.assertEqual(len(problems), 1)
            self.assertIn("CHANGELOG", problems[0])

    def test_a_complete_package_passes_offline(self) -> None:
        import tempfile

        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "scripts").mkdir()
            script = root / "scripts/workspace_packages.sh"
            script.write_text(
                '#!/usr/bin/env bash\n[ "$1" = dart ] && echo packages/demo\nexit 0\n'
            )
            script.chmod(0o700)
            (root / "packages/demo").mkdir(parents=True)
            (root / "packages/demo/CHANGELOG.md").write_text("# Changelog\n")
            self.assertEqual(check(root, offline=True), [])


def main() -> int:
    """Runs the check, or the self-tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()

    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(CheckPublishReadinessTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1

    problems = check(args.root, offline=args.offline)
    if problems:
        print("error: a package in this release cannot be published", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
