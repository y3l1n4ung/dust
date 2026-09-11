#!/usr/bin/env python3
"""Check every publishable package carries the CLI's version.

The release workflow publishes all five Dart packages from one `v*` tag, and
pub.dev automated publishing is configured per package with a tag pattern of
`v{version}`. A package whose version differs from the tag is refused:

    publishing is configured to only be allowed from actions with specific ref
    pattern, this token has 'refs/tags/v0.1.4' ref for which publishing is not
    allowed. Expected tag 'v0.1.0-beta.3'.

So the versions are lockstep by construction, and reasoning about one package's
semver in isolation gives the wrong answer. `dust_server` sat on its own
prerelease track and the `v0.1.4` tag failed on it, after publishing the three
packages ahead of it. `dust_flutter` was then bumped to 0.1.5 as an honest
patch, which would have done the same thing to `v0.2.0`.

Packages publish in order and the job stops at the first failure, so getting
this wrong leaves a partial release that cannot be rolled back.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


def cli_version(root: Path) -> str:
    """Returns the workspace version every package has to match."""
    text = (root / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', text)
    if not match:
        raise SystemExit("no [workspace.package] version in Cargo.toml")
    return match.group(1)


def package_version(pubspec: Path) -> str | None:
    """Returns the version a pubspec declares."""
    match = re.search(r"(?m)^version:\s*(\S+)", pubspec.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def publishable_packages(root: Path) -> list[Path]:
    """Returns the packages the release workflow publishes."""
    result = subprocess.run(
        [str(root / "scripts/workspace_packages.sh"), "dart"],
        capture_output=True, text=True, check=True,
    )
    flutter = subprocess.run(
        [str(root / "scripts/workspace_packages.sh"), "flutter"],
        capture_output=True, text=True, check=True,
    )
    return [root / entry for entry in sorted(result.stdout.split() + flutter.stdout.split())]


def check(root: Path) -> list[str]:
    """Returns one message per package that could not publish from the tag."""
    expected = cli_version(root)
    problems = []
    for package in publishable_packages(root):
        pubspec = package / "pubspec.yaml"
        if not pubspec.exists():
            continue
        found = package_version(pubspec)
        if found != expected:
            problems.append(
                f"{package.name} is {found}, but the release tag will be "
                f"v{expected}; pub.dev refuses a package whose version does not "
                f"match the tag, so this would expect tag v{found} and fail"
            )
    return problems


class CheckReleaseVersionsTests(unittest.TestCase):
    """Self-tests, so the check is exercised without the real workspace."""

    @staticmethod
    def _workspace(root: Path, versions: dict[str, str]) -> None:
        (root / "Cargo.toml").write_text(
            '[workspace.package]\nversion = "0.2.0"\n', encoding="utf-8"
        )
        (root / "scripts").mkdir(exist_ok=True)
        names = " ".join(f"packages/{name}" for name in versions)
        script = root / "scripts/workspace_packages.sh"
        script.write_text(
            f'#!/usr/bin/env bash\n[ "$1" = dart ] && for p in {names}; do echo "$p"; done\nexit 0\n'
        )
        script.chmod(0o755)
        for name, version in versions.items():
            package = root / "packages" / name
            package.mkdir(parents=True, exist_ok=True)
            (package / "pubspec.yaml").write_text(f"name: {name}\nversion: {version}\n")

    def test_matching_versions_pass(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, {"a": "0.2.0", "b": "0.2.0"})
            self.assertEqual(check(root), [])

    def test_a_lagging_package_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, {"a": "0.2.0", "b": "0.1.5"})
            problems = check(root)
            self.assertEqual(len(problems), 1)
            self.assertIn("b is 0.1.5", problems[0])
            self.assertIn("v0.1.5", problems[0])

    def test_a_prerelease_track_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self._workspace(root, {"a": "0.1.0-beta.3"})
            self.assertEqual(len(check(root)), 1)


def main() -> int:
    """Runs the check, or the self-tests."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()

    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(CheckReleaseVersionsTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1

    problems = check(args.root)
    if problems:
        print("error: a package cannot publish from the release tag", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
