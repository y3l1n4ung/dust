#!/usr/bin/env python3
"""Validate Dust CLI and package compatibility metadata."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
import unittest
from dataclasses import dataclass
from functools import total_ordering
from pathlib import Path

REQUIRED_PACKAGES = (
    "dust_dart",
    "dust_flutter",
    "dust_db_sqlite3",
    "dust_server",
)


@total_ordering
@dataclass(frozen=True, order=False)
class Version:
    """Comparable SemVer-like version used by Dust package metadata."""

    major: int
    minor: int
    patch: int
    prerelease: tuple[str, ...] = ()

    @classmethod
    def parse(cls, source: str) -> "Version":
        value = source.strip()
        value = value.split("+", 1)[0]
        core, _, prerelease = value.partition("-")
        parts = core.split(".")
        if len(parts) != 3 or not all(part.isdigit() for part in parts):
            raise ValueError(f"invalid version {source!r}; expected MAJOR.MINOR.PATCH")
        return cls(
            int(parts[0]),
            int(parts[1]),
            int(parts[2]),
            tuple(prerelease.split(".")) if prerelease else (),
        )

    def __lt__(self, other: "Version") -> bool:
        core = (self.major, self.minor, self.patch)
        other_core = (other.major, other.minor, other.patch)
        if core != other_core:
            return core < other_core
        return prerelease_less(self.prerelease, other.prerelease)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Version):
            return NotImplemented
        return (
            self.major,
            self.minor,
            self.patch,
            self.prerelease,
        ) == (
            other.major,
            other.minor,
            other.patch,
            other.prerelease,
        )


def prerelease_less(left: tuple[str, ...], right: tuple[str, ...]) -> bool:
    """Return SemVer prerelease ordering for two prerelease tuples."""

    if not left and not right:
        return False
    if not left:
        return False
    if not right:
        return True

    for left_part, right_part in zip(left, right):
        if left_part == right_part:
            continue
        left_numeric = left_part.isdigit()
        right_numeric = right_part.isdigit()
        if left_numeric and right_numeric:
            return int(left_part) < int(right_part)
        if left_numeric != right_numeric:
            return left_numeric
        return left_part < right_part

    return len(left) < len(right)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate Dust CLI/package compatibility metadata."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Dust repository root. Defaults to the parent of scripts/.",
    )
    parser.add_argument(
        "--release-tag",
        default="",
        help="Optional release tag, for example v0.1.3. Must match Cargo version.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run script unit tests instead of validating a repository.",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return run_self_tests()

    try:
        validate_repository(args.root.resolve(), args.release_tag)
    except ValidationError as error:
        print(f"compatibility check failed: {error}", file=sys.stderr)
        return 1

    print("compatibility check passed")
    return 0


class ValidationError(RuntimeError):
    """Compatibility metadata validation failure."""


def validate_repository(root: Path, release_tag: str = "") -> None:
    """Validate the repository compatibility contract against local versions."""

    cli_version = read_workspace_version(root / "Cargo.toml")
    normalized_tag = release_tag.removeprefix("v")
    if normalized_tag and normalized_tag != cli_version:
        raise ValidationError(
            f"release tag {release_tag!r} does not match Rust CLI version {cli_version}"
        )

    contract_path = root / "compatibility/dust-cli-packages.json"
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    entry = find_cli_entry(contract, cli_version, contract_path)
    constraints = entry.get("packageConstraints", {})

    for package in REQUIRED_PACKAGES:
        constraint = constraints.get(package)
        if not isinstance(constraint, str):
            raise ValidationError(
                f"{contract_path} CLI {cli_version} is missing {package}"
            )
        package_version = read_pubspec_version(root / "packages" / package / "pubspec.yaml")
        if not satisfies_constraint(package_version, constraint):
            raise ValidationError(
                f"{package} {package_version} does not satisfy CLI {cli_version} "
                f"range {constraint}"
            )


def find_cli_entry(contract: dict[str, object], cli_version: str, path: Path) -> dict[str, object]:
    """Return the contract row for one CLI version."""

    entries = contract.get("entries", [])
    if not isinstance(entries, list):
        raise ValidationError(f"{path} entries must be a list")

    for entry in entries:
        if isinstance(entry, dict) and entry.get("cliVersion") == cli_version:
            return entry

    raise ValidationError(f"{path} has no entry for CLI {cli_version}")


def read_workspace_version(path: Path) -> str:
    """Read [workspace.package] version from Cargo.toml."""

    in_workspace_package = False
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("[") and line.endswith("]"):
            in_workspace_package = line == "[workspace.package]"
            continue
        if in_workspace_package:
            match = re.fullmatch(r'version\s*=\s*"([^"]+)"', line)
            if match:
                return match.group(1)

    raise ValidationError(f"{path} is missing [workspace.package] version")


def read_pubspec_version(path: Path) -> str:
    """Read package version from pubspec.yaml."""

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("version:"):
            return line.split(":", 1)[1].strip().strip("\"'")

    raise ValidationError(f"{path} is missing version")


def satisfies_constraint(version_source: str, constraint: str) -> bool:
    """Return whether a version satisfies a simple pub-style range."""

    version = Version.parse(version_source)
    for token in constraint.split():
        operator, boundary_source = parse_constraint_token(token)
        boundary = Version.parse(boundary_source)
        if operator == ">=" and not (version >= boundary):
            return False
        if operator == ">" and not (version > boundary):
            return False
        if operator == "<=" and not (version <= boundary):
            return False
        if operator == "<" and not (version < boundary):
            return False
        if operator in ("=", "==") and version != boundary:
            return False
    return True


def parse_constraint_token(token: str) -> tuple[str, str]:
    """Parse one constraint token such as >=0.1.3."""

    for operator in (">=", "<=", "==", ">", "<", "="):
        if token.startswith(operator):
            return operator, token[len(operator) :]
    raise ValidationError(f"unsupported version constraint token {token!r}")


class CompatibilityScriptTests(unittest.TestCase):
    """Self-tests for compatibility validation."""

    def test_constraint_accepts_current_range(self) -> None:
        self.assertTrue(satisfies_constraint("0.1.3", ">=0.1.3 <0.2.0"))
        self.assertTrue(satisfies_constraint("0.1.9", ">=0.1.3 <0.2.0"))

    def test_constraint_orders_a_prerelease_lower_bound(self) -> None:
        self.assertTrue(
            satisfies_constraint("0.1.0-beta.3", ">=0.1.0-beta.3 <0.2.0")
        )
        self.assertFalse(
            satisfies_constraint("0.1.0-beta.2", ">=0.1.0-beta.3 <0.2.0")
        )

    def test_constraint_rejects_too_old_and_too_new(self) -> None:
        self.assertFalse(satisfies_constraint("0.1.2", ">=0.1.3 <0.2.0"))
        self.assertFalse(satisfies_constraint("0.2.0", ">=0.1.3 <0.2.0"))

    def test_repository_validation_catches_mismatched_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            write_fixture_repo(root, dust_dart_version="0.1.2")

            with self.assertRaisesRegex(ValidationError, "dust_dart 0.1.2"):
                validate_repository(root)

    def test_repository_validation_catches_release_tag_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            write_fixture_repo(root)

            with self.assertRaisesRegex(ValidationError, "release tag"):
                validate_repository(root, "v0.1.4")


def write_fixture_repo(root: Path, dust_dart_version: str = "0.1.3") -> None:
    """Write a minimal fake repo for script self-tests."""

    (root / "compatibility").mkdir(parents=True)
    for package in REQUIRED_PACKAGES:
        (root / "packages" / package).mkdir(parents=True)
    (root / "Cargo.toml").write_text(
        '[workspace.package]\nversion = "0.1.3"\n',
        encoding="utf-8",
    )
    (root / "compatibility/dust-cli-packages.json").write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "entries": [
                    {
                        "cliVersion": "0.1.3",
                        "packageConstraints": {
                            "dust_dart": ">=0.1.3 <0.2.0",
                            "dust_flutter": ">=0.1.3 <0.2.0",
                            "dust_db_sqlite3": ">=0.1.3 <0.2.0",
                            "dust_server": ">=0.1.0-beta.1 <0.2.0",
                        },
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    for package, version in {
        "dust_dart": dust_dart_version,
        "dust_flutter": "0.1.3",
        "dust_db_sqlite3": "0.1.3",
        "dust_server": "0.1.0-beta.2",
    }.items():
        (root / "packages" / package / "pubspec.yaml").write_text(
            f"name: {package}\nversion: {version}\n",
            encoding="utf-8",
        )


def run_self_tests() -> int:
    """Run unit tests for this script."""

    suite = unittest.defaultTestLoader.loadTestsFromTestCase(CompatibilityScriptTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
