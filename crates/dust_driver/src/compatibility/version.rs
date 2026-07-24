use std::cmp::Ordering;

use super::contract::CompatibilityActions;

/// Parsed semantic package version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Version {
    /// Major version.
    major: u64,
    /// Minor version.
    minor: u64,
    /// Patch version.
    patch: u64,
    /// Optional prerelease identifier.
    prerelease: Option<String>,
}

impl Version {
    /// Parses the semantic version subset used by Dart package versions.
    pub(super) fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let version = value.split_once('+').map_or(value, |(version, _)| version);
        let (core, prerelease) = version
            .split_once('-')
            .map_or((version, None), |(core, prerelease)| {
                (core, Some(prerelease.to_owned()))
            });
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }

        Some(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

impl Ord for Version {
    /// Compares semantic versions with stable releases after prereleases.
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(left), Some(right)) => left.cmp(right),
            })
    }
}

impl PartialOrd for Version {
    /// Partially compares semantic versions.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Parsed package version constraint.
pub(super) struct VersionConstraint {
    /// All requirements that must be satisfied.
    requirements: Vec<VersionRequirement>,
}

impl VersionConstraint {
    /// Parses whitespace-separated version requirements.
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        let requirements = value
            .split_whitespace()
            .map(VersionRequirement::parse)
            .collect::<Result<Vec<_>, _>>()?;
        if requirements.is_empty() {
            return Err("constraint is empty".to_owned());
        }
        Ok(Self { requirements })
    }

    /// Returns whether a version satisfies every requirement.
    pub(super) fn is_satisfied_by(&self, version: &Version) -> bool {
        self.requirements
            .iter()
            .all(|requirement| requirement.is_satisfied_by(version))
    }

    /// Selects the best action message for the failed requirement.
    pub(super) fn mismatch_action<'a>(
        &self,
        version: &Version,
        actions: &'a CompatibilityActions,
    ) -> &'a str {
        for requirement in &self.requirements {
            if requirement.is_satisfied_by(version) {
                continue;
            }
            return match requirement.operator {
                VersionOperator::GreaterOrEqual | VersionOperator::GreaterThan => {
                    &actions.package_too_old
                }
                VersionOperator::LessOrEqual | VersionOperator::LessThan => {
                    &actions.package_too_new
                }
                VersionOperator::Equal => {
                    if version < &requirement.version {
                        &actions.package_too_old
                    } else {
                        &actions.package_too_new
                    }
                }
            };
        }

        &actions.package_too_new
    }
}

/// One atomic version requirement.
struct VersionRequirement {
    /// Comparison operator.
    operator: VersionOperator,
    /// Version used by the comparison.
    version: Version,
}

impl VersionRequirement {
    /// Parses one requirement such as `>=0.1.3`.
    fn parse(value: &str) -> Result<Self, String> {
        let (operator, version) = VersionOperator::split(value)?;
        let version = Version::parse(version)
            .ok_or_else(|| format!("invalid version `{version}` in `{value}`"))?;
        Ok(Self { operator, version })
    }

    /// Returns whether a version satisfies this requirement.
    fn is_satisfied_by(&self, version: &Version) -> bool {
        match self.operator {
            VersionOperator::GreaterOrEqual => version >= &self.version,
            VersionOperator::GreaterThan => version > &self.version,
            VersionOperator::LessOrEqual => version <= &self.version,
            VersionOperator::LessThan => version < &self.version,
            VersionOperator::Equal => version == &self.version,
        }
    }
}

/// Supported comparison operators in the compatibility contract.
#[derive(Clone, Copy)]
enum VersionOperator {
    /// `>=`
    GreaterOrEqual,
    /// `>`
    GreaterThan,
    /// `<=`
    LessOrEqual,
    /// `<`
    LessThan,
    /// `=` or `==`
    Equal,
}

impl VersionOperator {
    /// Splits an operator prefix from a version requirement string.
    fn split(value: &str) -> Result<(Self, &str), String> {
        for (prefix, operator) in [
            (">=", Self::GreaterOrEqual),
            ("<=", Self::LessOrEqual),
            ("==", Self::Equal),
            (">", Self::GreaterThan),
            ("<", Self::LessThan),
            ("=", Self::Equal),
        ] {
            if let Some(version) = value.strip_prefix(prefix) {
                return Ok((operator, version));
            }
        }

        Err(format!("unsupported requirement `{value}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Version, VersionConstraint};

    #[test]
    fn version_constraint_accepts_current_minor_range() {
        let constraint = VersionConstraint::parse(">=0.1.3 <0.2.0").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("0.1.3").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("0.1.9+build.1").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("0.1.2").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("0.2.0").unwrap()));
    }

    #[test]
    fn prerelease_is_before_stable_release() {
        assert!(Version::parse("0.1.3-dev.1").unwrap() < Version::parse("0.1.3").unwrap());
    }
}
