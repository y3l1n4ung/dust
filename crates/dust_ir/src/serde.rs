use crate::{AnnotationValueIr, ConstructorParamIr};

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
    /// Internal or adjacent tag field for sealed class variants.
    pub tag: Option<String>,
    /// Adjacent content field for sealed class variants.
    pub content: Option<String>,
    /// Whether sealed class decoding should try variants without a tag.
    pub untagged: bool,
    /// Whether unknown keys should be rejected when deserializing.
    pub disallow_unrecognized_keys: bool,
    /// Sealed class variant metadata, in factory declaration order.
    pub variants: Vec<SerdeVariantConfigIr>,
}

impl SerdeClassConfigIr {
    /// Returns `true` when the config carries no effective settings.
    pub fn is_empty(&self) -> bool {
        self.rename.is_none()
            && self.rename_all.is_none()
            && self.tag.is_none()
            && self.content.is_none()
            && !self.untagged
            && !self.disallow_unrecognized_keys
            && self.variants.is_empty()
    }

    /// Returns whether the class config requests sealed class serde metadata.
    pub fn uses_sealed_representation(&self) -> bool {
        self.tag.is_some() || self.content.is_some() || self.untagged
    }
}

/// Normalized metadata for one sealed class SerDe variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerdeVariantConfigIr {
    /// Source factory constructor name, for example `paid`.
    pub constructor_name: String,
    /// Redirected target class name, for example `PaymentPaid`.
    pub target_class_name: String,
    /// Resolved tag value after variant rename and class `renameAll`.
    pub tag: String,
    /// Source factory constructor parameters used to synthesize variant classes.
    pub params: Vec<ConstructorParamIr>,
}

/// Normalized serde-related configuration attached to one field.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerdeFieldConfigIr {
    /// Optional explicit JSON key rename.
    pub rename: Option<String>,
    /// Alternative accepted JSON keys during deserialization.
    pub aliases: Vec<String>,
    /// Raw source expression for one custom serde codec, if one was provided.
    pub codec_source: Option<String>,
    /// Raw source expression for a default value, if one was provided.
    pub default_value_source: Option<String>,
    /// Parser-owned typed default value, when available from annotation IR.
    pub default_value: Option<AnnotationValueIr>,
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
            && self.codec_source.is_none()
            && self.default_value_source.is_none()
            && self.default_value.is_none()
            && !self.skip_serializing
            && !self.skip_deserializing
    }
}
