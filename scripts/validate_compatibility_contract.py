#!/usr/bin/env python3
"""Validate Dust CLI and package compatibility metadata."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from pathlib import Path

from compatibility_version import Version

REQUIRED_PACKAGES = (
    "dust_dart",
    "dust_flutter",
    "dust_db_sqlite3",
    "dust_db_postgres",
    "dust_server",
)


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
        require_changelog_section(
            root / "packages" / package / "CHANGELOG.md", package_version, package
        )

    if normalized_tag:
        require_changelog_section(root / "CHANGELOG.md", release_tag, "Dust")


def require_changelog_section(path: Path, version: str, label: str) -> None:
    """Fail unless a changelog carries a dated section for one version.

    The release workflow publishes whatever version a pubspec names, and skips
    a version pub.dev already has. A package bumped without a changelog entry,
    or left unbumped while its source moved, therefore publishes nothing and
    says nothing. Tying the two together is the cheapest place to notice.
    """

    if not path.exists():
        raise ValidationError(f"{path} is missing")

    heading = re.compile(
        rf"^## \[v?{re.escape(version.removeprefix('v'))}\] - \d{{4}}-\d{{2}}-\d{{2}}$",
        re.MULTILINE,
    )
    if not heading.search(path.read_text(encoding="utf-8")):
        raise ValidationError(
            f"{path} has no dated section for {label} {version}"
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


def run_self_tests() -> int:
    """Run this script's own tests, which live in `compatibility_tests.py`."""

    import unittest

    from compatibility_tests import CompatibilityScriptTests

    suite = unittest.defaultTestLoader.loadTestsFromTestCase(CompatibilityScriptTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
