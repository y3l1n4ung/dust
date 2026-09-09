"""Self-tests for the compatibility contract validator."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from compatibility_version import Version
from validate_compatibility_contract import (
    REQUIRED_PACKAGES,
    ValidationError,
    find_cli_entry,
    read_pubspec_version,
    read_workspace_version,
    require_changelog_section,
    satisfies_constraint,
    validate_repository,
)

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

    def test_repository_validation_catches_a_bump_with_no_changelog(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            write_fixture_repo(root)
            (root / "packages/dust_server/pubspec.yaml").write_text(
                "name: dust_server\nversion: 0.1.0-beta.3\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                ValidationError, "no dated section for dust_server 0.1.0-beta.3"
            ):
                validate_repository(root)

    def test_repository_validation_catches_a_tag_with_no_changelog(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            write_fixture_repo(root)
            (root / "CHANGELOG.md").write_text(
                "# Changelog\n\n## [Unreleased]\n\n## [v0.1.2] - 2026-07-10\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                ValidationError, "no dated section for Dust v0.1.3"
            ):
                validate_repository(root, "v0.1.3")

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
                            "dust_db_postgres": ">=0.1.3 <0.2.0",
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
        "dust_db_postgres": "0.1.3",
        "dust_server": "0.1.0-beta.2",
    }.items():
        (root / "packages" / package / "pubspec.yaml").write_text(
            f"name: {package}\nversion: {version}\n",
            encoding="utf-8",
        )
        (root / "packages" / package / "CHANGELOG.md").write_text(
            f"# Changelog\n\n## [Unreleased]\n\n## [{version}] - 2026-07-28\n",
            encoding="utf-8",
        )
    (root / "CHANGELOG.md").write_text(
        "# Changelog\n\n## [Unreleased]\n\n## [v0.1.3] - 2026-07-28\n",
        encoding="utf-8",
    )
