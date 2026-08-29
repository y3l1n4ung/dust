#!/usr/bin/env python3
"""Print LCOV line coverage and fail when it falls below a floor."""

from __future__ import annotations

import argparse
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    parser.add_argument("floor", type=float)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    total = 0
    covered = 0
    uncovered: dict[str, list[str]] = {}
    current: str | None = None
    missing: list[str] = []

    for line in args.report.read_text().splitlines():
        if line.startswith("SF:"):
            current = line[3:]
            missing = []
        elif line.startswith("DA:"):
            number, count, *_ = line[3:].split(",")
            total += 1
            if int(count) > 0:
                covered += 1
            else:
                missing.append(number)
        elif line == "end_of_record" and current and missing:
            uncovered[current] = missing

    if total == 0:
        raise SystemExit("no coverage data; did the suite run?")

    percent = covered * 100 / total
    print(f"{covered}/{total} lines = {percent:.2f}%")

    for path, lines in uncovered.items():
        print(f"  uncovered {path}: {', '.join(lines)}")

    if percent + 1e-9 < args.floor:
        raise SystemExit(
            f"coverage {percent:.2f}% is below the {args.floor:.2f}% floor"
        )


if __name__ == "__main__":
    main()
