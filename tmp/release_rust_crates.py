#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    print("Python 3.11+ is required for tomllib.", file=sys.stderr)
    raise SystemExit(1)


ROOT = Path(__file__).resolve().parents[1]
CRATE_ORDER = [
    "dust_text",
    "dust_diagnostics",
    "dust_ir",
    "dust_parser_dart",
    "dust_workspace",
    "dust_dart_emit",
    "dust_parser_dart_ts",
    "dust_plugin_api",
    "dust_cache",
    "dust_resolver",
    "dust_plugin_derive",
    "dust_plugin_serde",
    "dust_http_client_plugin",
    "dust_emitter",
    "dust_driver",
    "dust_cli",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Temporary Rust release helper for Dust crates."
    )
    parser.add_argument(
        "--publish",
        action="store_true",
        help="Actually run cargo publish in dependency order.",
    )
    parser.add_argument(
        "--from",
        dest="start_from",
        choices=CRATE_ORDER,
        default=CRATE_ORDER[0],
        help="Start from a specific crate in the publish order.",
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Pass --allow-dirty to cargo commands.",
    )
    parser.add_argument(
        "--poll-seconds",
        type=float,
        default=10.0,
        help="Seconds between crates.io visibility checks after publish.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=900.0,
        help="Maximum wait for a crate version to appear on crates.io.",
    )
    return parser.parse_args()


def read_workspace_version() -> str:
    cargo_toml = ROOT / "Cargo.toml"
    with cargo_toml.open("rb") as handle:
        data = tomllib.load(handle)
    return data["workspace"]["package"]["version"]


def selected_crates(start_from: str) -> list[str]:
    start_index = CRATE_ORDER.index(start_from)
    return CRATE_ORDER[start_index:]


def run(cmd: list[str], *, cwd: Path = ROOT) -> None:
    print("+", " ".join(cmd), flush=True)
    subprocess.run(cmd, cwd=cwd, check=True)


def git_is_clean() -> bool:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip() == ""


def cargo_flags(args: argparse.Namespace) -> list[str]:
    return ["--allow-dirty"] if args.allow_dirty else []


def crate_version_visible(crate: str, version: str) -> bool:
    url = f"https://crates.io/api/v1/crates/{crate}"
    request = urllib.request.Request(
        url,
        headers={"User-Agent": "dust-release-temp-script/0.1.0"},
    )
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            payload = json.load(response)
    except urllib.error.URLError:
        return False

    for item in payload.get("versions", []):
        if item.get("num") == version:
            return True
    return False


def wait_for_crate(crate: str, version: str, poll_seconds: float, timeout_seconds: float) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if crate_version_visible(crate, version):
            print(f"{crate} {version} is visible on crates.io.", flush=True)
            return
        print(
            f"Waiting for {crate} {version} to appear on crates.io...",
            flush=True,
        )
        time.sleep(poll_seconds)
    raise SystemExit(
        f"Timed out waiting for {crate} {version} to appear on crates.io."
    )


def preflight(crates: list[str], args: argparse.Namespace) -> None:
    flags = cargo_flags(args)
    print("Running package assembly preflight for all selected crates.", flush=True)
    for crate in crates:
        run(["cargo", "package", *flags, "--list", "-p", crate])

    first = crates[0]
    print(
        "\nRunning cargo publish --dry-run for the first crate only.\n"
        "Dependent crates cannot dry-run against crates.io until lower-layer "
        "crates are actually published and indexed.",
        flush=True,
    )
    run(["cargo", "publish", *flags, "--dry-run", "-p", first])


def publish(crates: list[str], version: str, args: argparse.Namespace) -> None:
    if not args.allow_dirty and not git_is_clean():
        raise SystemExit(
            "Refusing to publish from a dirty git state. Commit first or pass --allow-dirty."
        )

    flags = cargo_flags(args)
    for crate in crates:
        run(["cargo", "publish", *flags, "-p", crate])
        wait_for_crate(crate, version, args.poll_seconds, args.timeout_seconds)

    print(
        f"\nRust publishes complete. Push tag v{version} to trigger the GitHub release workflow.",
        flush=True,
    )


def main() -> int:
    args = parse_args()
    version = read_workspace_version()
    crates = selected_crates(args.start_from)

    print(f"Workspace root: {ROOT}")
    print(f"Workspace version: {version}")
    print("Crates:", ", ".join(crates))

    if args.publish:
        publish(crates, version, args)
    else:
        preflight(crates, args)
        print(
            "\nPreflight complete. Re-run with --publish to perform the actual release.",
            flush=True,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
