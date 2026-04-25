use std::collections::HashSet;

use dust_plugin_api::{ClassMixinContribution, PluginContribution};

/// The merged plugin sections for one emitted library.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MergedSections {
    /// Helper declarations shared across generated output.
    pub shared_helpers: Vec<String>,
    /// Per-class mixin members merged across all plugins.
    pub mixin_members: Vec<ClassMixinContribution>,
    /// Support types needed by generated output.
    pub support_types: Vec<String>,
    /// Top-level generated functions.
    pub top_level_functions: Vec<String>,
}

impl MergedSections {
    /// Merges plugin contributions while preserving registration order.
    pub(crate) fn from_contributions(contributions: &[PluginContribution]) -> Self {
        let mut merged = Self::default();
        let mut helper_seen = HashSet::new();

        for contribution in contributions {
            for helper in &contribution.shared_helpers {
                if helper_seen.insert(helper.clone()) {
                    merged.shared_helpers.push(helper.clone());
                }
            }

            for mixin in &contribution.mixin_members {
                if let Some(existing) = merged
                    .mixin_members
                    .iter_mut()
                    .find(|entry| entry.class_name == mixin.class_name)
                {
                    existing.members.extend(mixin.members.iter().cloned());
                } else {
                    merged.mixin_members.push(mixin.clone());
                }
            }

            merged
                .support_types
                .extend(contribution.support_types.iter().cloned());
            merged
                .top_level_functions
                .extend(contribution.top_level_functions.iter().cloned());
        }

        merged
    }

    /// Returns generated mixin members for the named class.
    pub(crate) fn members_for_class(&self, class_name: &str) -> &[String] {
        self.mixin_members
            .iter()
            .find(|entry| entry.class_name == class_name)
            .map(|entry| entry.members.as_slice())
            .unwrap_or(&[])
    }
}
