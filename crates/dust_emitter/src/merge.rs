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
    pub(crate) fn from_contributions(contributions: Vec<PluginContribution>) -> Self {
        let mut merged = Self::default();

        for contribution in contributions {
            for helper in contribution.shared_helpers {
                if !merged
                    .shared_helpers
                    .iter()
                    .any(|existing| existing == &helper)
                {
                    merged.shared_helpers.push(helper);
                }
            }

            for mixin in contribution.mixin_members {
                merge_mixin_members(&mut merged, mixin);
            }

            merged.support_types.extend(contribution.support_types);
            merged
                .top_level_functions
                .extend(contribution.top_level_functions);
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

fn merge_mixin_members(merged: &mut MergedSections, mixin: ClassMixinContribution) {
    if let Some(existing) = merged
        .mixin_members
        .iter_mut()
        .find(|entry| entry.class_name == mixin.class_name)
    {
        existing.members.extend(mixin.members);
        return;
    }

    merged.mixin_members.push(mixin);
}
