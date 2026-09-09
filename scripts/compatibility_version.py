"""SemVer-like versions and constraints, as Dust package metadata spells them."""

from __future__ import annotations

from dataclasses import dataclass
from functools import total_ordering

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
