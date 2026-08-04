/// Selects whether a validating plugin may use its live backing service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationAccess {
    /// Validate against the live backing service.
    Online,
    /// Validate only from previously written metadata.
    Offline,
}

/// Selects whether validation may update plugin-owned metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataOutput {
    /// Validation may write or refresh metadata.
    Write,
    /// Validation must not write metadata.
    ReadOnly,
}

/// Shared execution policy for plugins that validate against external state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginExecutionMode {
    /// Whether validation may use the live backing service.
    pub validation: ValidationAccess,
    /// Whether validation may update plugin-owned metadata.
    pub metadata: MetadataOutput,
}

impl PluginExecutionMode {
    /// Creates an online validation mode with the requested metadata policy.
    pub const fn online(metadata: MetadataOutput) -> Self {
        Self {
            validation: ValidationAccess::Online,
            metadata,
        }
    }

    /// Creates an offline validation mode with the requested metadata policy.
    pub const fn offline(metadata: MetadataOutput) -> Self {
        Self {
            validation: ValidationAccess::Offline,
            metadata,
        }
    }

    /// Returns the stable cache-key segment for this mode.
    pub const fn cache_key(self) -> &'static str {
        match (self.validation, self.metadata) {
            (ValidationAccess::Online, MetadataOutput::Write) => "online:write-metadata",
            (ValidationAccess::Online, MetadataOutput::ReadOnly) => "online:no-metadata",
            (ValidationAccess::Offline, MetadataOutput::Write) => "offline:write-metadata",
            (ValidationAccess::Offline, MetadataOutput::ReadOnly) => "offline:no-metadata",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataOutput, PluginExecutionMode};

    #[test]
    fn cache_keys_distinguish_validation_and_metadata_policies() {
        assert_eq!(
            PluginExecutionMode::online(MetadataOutput::Write).cache_key(),
            "online:write-metadata"
        );
        assert_eq!(
            PluginExecutionMode::online(MetadataOutput::ReadOnly).cache_key(),
            "online:no-metadata"
        );
        assert_eq!(
            PluginExecutionMode::offline(MetadataOutput::Write).cache_key(),
            "offline:write-metadata"
        );
        assert_eq!(
            PluginExecutionMode::offline(MetadataOutput::ReadOnly).cache_key(),
            "offline:no-metadata"
        );
    }
}
