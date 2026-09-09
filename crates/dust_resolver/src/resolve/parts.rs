//! Deciding whether a declaration needs a generated part, and checking the part URI.

use super::*;

/// Returns whether a resolved class requires a generated part file.
pub(super) fn class_needs_part(class: &ResolvedClass, partless_config_symbols: &[&str]) -> bool {
    !class.traits.is_empty()
        || class
            .configs
            .iter()
            .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        || class.constructors.iter().any(|constructor| {
            constructor
                .configs
                .iter()
                .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        })
        || class.fields.iter().any(|field| {
            field
                .configs
                .iter()
                .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
        })
        || class.methods.iter().any(|method| {
            !method.traits.is_empty()
                || method
                    .configs
                    .iter()
                    .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
                || method.params.iter().any(|param| {
                    !param.traits.is_empty()
                        || param.configs.iter().any(|config| {
                            !partless_config_symbols.contains(&config.symbol.0.as_str())
                        })
                })
        })
}

/// Returns whether a resolved class contains any Dust-owned symbol.
pub(super) fn class_has_dust_symbol(class: &ResolvedClass) -> bool {
    !class.traits.is_empty()
        || !class.configs.is_empty()
        || class
            .constructors
            .iter()
            .any(|constructor| !constructor.configs.is_empty())
        || class.fields.iter().any(|field| !field.configs.is_empty())
        || class.methods.iter().any(|method| {
            !method.traits.is_empty()
                || !method.configs.is_empty()
                || method
                    .params
                    .iter()
                    .any(|param| !param.traits.is_empty() || !param.configs.is_empty())
        })
}

/// Returns whether a resolved enum requires a generated part file.
pub(super) fn enum_needs_part(enum_ir: &ResolvedEnum, partless_config_symbols: &[&str]) -> bool {
    !enum_ir.traits.is_empty()
        || enum_ir
            .configs
            .iter()
            .any(|config| !partless_config_symbols.contains(&config.symbol.0.as_str()))
}
