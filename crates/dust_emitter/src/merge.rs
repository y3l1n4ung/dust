use std::collections::{HashMap, HashSet};

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
        let mut mixin_index_by_class = HashMap::<&str, usize>::new();

        for contribution in contributions {
            for helper in &contribution.shared_helpers {
                if helper_seen.insert(helper.as_str()) {
                    merged.shared_helpers.push(helper.clone());
                }
            }

            for mixin in &contribution.mixin_members {
                merge_mixin_members(&mut merged, &mut mixin_index_by_class, mixin);
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

fn merge_mixin_members<'a>(
    merged: &mut MergedSections,
    index_by_class: &mut HashMap<&'a str, usize>,
    mixin: &'a ClassMixinContribution,
) {
    if let Some(index) = index_by_class.get(mixin.class_name.as_str()).copied() {
        merged.mixin_members[index]
            .members
            .extend(mixin.members.iter().cloned());
        return;
    }

    index_by_class.insert(&mixin.class_name, merged.mixin_members.len());
    merged.mixin_members.push(mixin.clone());
}
