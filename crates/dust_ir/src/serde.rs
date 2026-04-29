/// Normalized rename strategies derived from `SerDeRename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerdeRenameRuleIr {
    /// Lowercase words without separators.
    LowerCase,
    /// Uppercase words without separators.
    UpperCase,
    /// `PascalCase`.
    PascalCase,
    /// `camelCase`.
    CamelCase,
    /// `snake_case`.
    SnakeCase,
    /// `SCREAMING_SNAKE_CASE`.
    ScreamingSnakeCase,
    /// `kebab-case`.
    KebabCase,
    /// `SCREAMING-KEBAB-CASE`.
    ScreamingKebabCase,
}

/// Normalized serde-related configuration attached to a class.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerdeClassConfigIr {
    /// Optional explicit rename for the target.
    pub rename: Option<String>,
    /// Optional global field renaming rule.
    pub rename_all: Option<SerdeRenameRuleIr>,
    /// Whether unknown keys should be rejected when deserializing.
    pub disallow_unrecognized_keys: bool,
}

impl SerdeClassConfigIr {
    /// Returns `true` when the config carries no effective settings.
    pub fn is_empty(&self) -> bool {
        self.rename.is_none() && self.rename_all.is_none() && !self.disallow_unrecognized_keys
    }
}

/// Normalized serde-related configuration attached to one field.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerdeFieldConfigIr {
    /// Optional explicit JSON key rename.
    pub rename: Option<String>,
    /// Alternative accepted JSON keys during deserialization.
    pub aliases: Vec<String>,
    /// Raw source expression for a default value, if one was provided.
    pub default_value_source: Option<String>,
    /// Whether serialization should skip this field.
    pub skip_serializing: bool,
    /// Whether deserialization should skip this field.
    pub skip_deserializing: bool,
}

impl SerdeFieldConfigIr {
    /// Returns `true` when the config carries no effective settings.
    pub fn is_empty(&self) -> bool {
        self.rename.is_none()
            && self.aliases.is_empty()
            && self.default_value_source.is_none()
            && !self.skip_serializing
            && !self.skip_deserializing
    }
}
