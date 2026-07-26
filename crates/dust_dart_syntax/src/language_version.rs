use std::fmt;

/// A Dart language version represented as `major.minor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DartLanguageVersion {
    /// Major Dart language version.
    pub major: u16,
    /// Minor Dart language version.
    pub minor: u16,
}

impl DartLanguageVersion {
    /// Dart 2.12, the null-safety baseline.
    pub const DART_2_12: Self = Self::new(2, 12);
    /// Dart 2.17, the super-parameter baseline.
    pub const DART_2_17: Self = Self::new(2, 17);
    /// Dart 3.0, the records, patterns, and class-modifier baseline.
    pub const DART_3_0: Self = Self::new(3, 0);
    /// Dart 3.3, the extension-type baseline.
    pub const DART_3_3: Self = Self::new(3, 3);
    /// Dart 3.8, the null-aware collection element baseline.
    pub const DART_3_8: Self = Self::new(3, 8);
    /// Dart 3.10, the dot-shorthand baseline.
    pub const DART_3_10: Self = Self::new(3, 10);
    /// Dart 3.12, the private named-parameter baseline.
    pub const DART_3_12: Self = Self::new(3, 12);
    /// Dart 3.13, the primary-constructor baseline tracked by Dust fixtures.
    pub const DART_3_13: Self = Self::new(3, 13);

    /// Creates a Dart language version.
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the latest language version known to this Dust build.
    pub const fn latest_known() -> Self {
        Self::DART_3_13
    }

    /// Parses `major.minor` or `major.minor.patch` text.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim().trim_matches(['"', '\'']);
        let mut parts = input.split('.');
        let major = parse_u16_prefix(parts.next()?)?;
        let minor = parse_u16_prefix(parts.next()?)?;
        Some(Self::new(major, minor))
    }

    /// Extracts a Dart file-level language version comment such as
    /// `// @dart=3.0` or `// @dart = 3.0`.
    pub fn from_source_language_comment(source: &str) -> Option<Self> {
        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                continue;
            }
            let Some(comment) = trimmed.strip_prefix("//") else {
                break;
            };
            let comment = comment.trim_start();
            let Some(rest) = comment.strip_prefix("@dart") else {
                continue;
            };
            let rest = rest.trim_start();
            let version = rest.strip_prefix('=')?.trim_start();
            return Self::parse(version);
        }
        None
    }

    /// Extracts the lowest Dart SDK language version from a pubspec SDK
    /// constraint such as `>=3.0.0 <4.0.0` or `^3.3.0`.
    pub fn from_sdk_constraint_lower_bound(constraint: &str) -> Option<Self> {
        let constraint = constraint.trim().trim_matches(['"', '\'']);
        for marker in [">=", "^"] {
            if let Some(index) = constraint.find(marker) {
                let version = &constraint[index + marker.len()..];
                return first_version_in(version);
            }
        }
        first_version_in(constraint)
    }

    /// Returns whether this version supports the given Dart language feature.
    pub fn supports(self, feature: DartLanguageFeature) -> bool {
        self >= feature.required_version()
    }

    /// Returns whether this version supports records.
    pub fn supports_records(self) -> bool {
        self.supports(DartLanguageFeature::Records)
    }

    /// Returns whether this version supports patterns.
    pub fn supports_patterns(self) -> bool {
        self.supports(DartLanguageFeature::Patterns)
    }

    /// Returns whether this version supports class modifiers.
    pub fn supports_class_modifiers(self) -> bool {
        self.supports(DartLanguageFeature::ClassModifiers)
    }

    /// Returns whether this version supports extension types.
    pub fn supports_extension_types(self) -> bool {
        self.supports(DartLanguageFeature::ExtensionTypes)
    }

    /// Returns whether this version supports null-aware collection elements.
    pub fn supports_null_aware_collection_elements(self) -> bool {
        self.supports(DartLanguageFeature::NullAwareCollectionElements)
    }

    /// Returns whether this version supports dot shorthands.
    pub fn supports_dot_shorthands(self) -> bool {
        self.supports(DartLanguageFeature::DotShorthands)
    }

    /// Returns whether this version supports private named parameters.
    pub fn supports_private_named_parameters(self) -> bool {
        self.supports(DartLanguageFeature::PrivateNamedParameters)
    }

    /// Returns whether this version supports primary constructors.
    pub fn supports_primary_constructors(self) -> bool {
        self.supports(DartLanguageFeature::PrimaryConstructors)
    }
}

impl Default for DartLanguageVersion {
    fn default() -> Self {
        Self::latest_known()
    }
}

impl fmt::Display for DartLanguageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// A Dart language feature Dust may need to parse, format, diagnose, or emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DartLanguageFeature {
    /// Dart records.
    Records,
    /// Dart patterns.
    Patterns,
    /// Class modifiers such as `sealed`, `base`, `interface`, and `final`.
    ClassModifiers,
    /// Extension types.
    ExtensionTypes,
    /// Null-aware elements in collection literals.
    NullAwareCollectionElements,
    /// Dot shorthands such as `.ready`.
    DotShorthands,
    /// Private named parameters.
    PrivateNamedParameters,
    /// Primary constructors.
    PrimaryConstructors,
}

impl DartLanguageFeature {
    /// Returns the first Dart language version that supports this feature.
    pub const fn required_version(self) -> DartLanguageVersion {
        match self {
            Self::Records | Self::Patterns | Self::ClassModifiers => DartLanguageVersion::DART_3_0,
            Self::ExtensionTypes => DartLanguageVersion::DART_3_3,
            Self::NullAwareCollectionElements => DartLanguageVersion::DART_3_8,
            Self::DotShorthands => DartLanguageVersion::DART_3_10,
            Self::PrivateNamedParameters => DartLanguageVersion::DART_3_12,
            Self::PrimaryConstructors => DartLanguageVersion::DART_3_13,
        }
    }

    /// Returns a user-facing feature name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Records => "records",
            Self::Patterns => "patterns",
            Self::ClassModifiers => "class modifiers",
            Self::ExtensionTypes => "extension types",
            Self::NullAwareCollectionElements => "null-aware collection elements",
            Self::DotShorthands => "dot shorthands",
            Self::PrivateNamedParameters => "private named parameters",
            Self::PrimaryConstructors => "primary constructors",
        }
    }
}

/// Returns whether a Dart version supports primary constructors.
pub fn supports_primary_constructors(version: DartLanguageVersion) -> bool {
    version.supports_primary_constructors()
}

/// Returns whether a Dart version supports dot shorthands.
pub fn supports_dot_shorthands(version: DartLanguageVersion) -> bool {
    version.supports_dot_shorthands()
}

/// Parses the leading ASCII digit run from one version component.
fn parse_u16_prefix(input: &str) -> Option<u16> {
    let digits = input
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty())
        .then_some(digits)
        .and_then(|digits| digits.parse().ok())
}

/// Returns the first `major.minor` version found in free-form text.
fn first_version_in(input: &str) -> Option<DartLanguageVersion> {
    let input = input.trim_start();
    for (index, ch) in input.char_indices() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if let Some(version) = DartLanguageVersion::parse(&input[index..]) {
            return Some(version);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{DartLanguageFeature, DartLanguageVersion};

    #[test]
    fn parses_language_version_text_and_comments() {
        assert_eq!(
            DartLanguageVersion::parse("3.10.0"),
            Some(DartLanguageVersion::new(3, 10))
        );
        assert_eq!(
            DartLanguageVersion::from_source_language_comment("\n// @dart = 2.17\nclass User {}\n"),
            Some(DartLanguageVersion::new(2, 17))
        );
        assert_eq!(
            DartLanguageVersion::from_source_language_comment("class User {}\n// @dart=2.12\n"),
            None
        );
    }

    #[test]
    fn parses_pubspec_sdk_constraint_lower_bound() {
        assert_eq!(
            DartLanguageVersion::from_sdk_constraint_lower_bound(">=3.0.0 <4.0.0"),
            Some(DartLanguageVersion::new(3, 0))
        );
        assert_eq!(
            DartLanguageVersion::from_sdk_constraint_lower_bound("^3.8.0"),
            Some(DartLanguageVersion::new(3, 8))
        );
    }

    #[test]
    fn gates_known_dart_language_features() {
        let dart_3_0 = DartLanguageVersion::DART_3_0;
        let dart_3_10 = DartLanguageVersion::DART_3_10;

        assert!(dart_3_0.supports(DartLanguageFeature::Records));
        assert!(dart_3_0.supports(DartLanguageFeature::Patterns));
        assert!(!dart_3_0.supports_dot_shorthands());
        assert!(dart_3_10.supports_dot_shorthands());
    }
}
