/// Generated members that belong inside one emitted mixin block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassMixinContribution {
    /// The source class this mixin targets.
    pub class_name: String,
    /// The generated members to place inside the mixin.
    pub members: Vec<String>,
}

/// Generated code fragments returned by one plugin for one library.
///
/// The emitter will later merge these sections in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginContribution {
    /// Helper declarations shared across generated output.
    pub shared_helpers: Vec<String>,
    /// Class-scoped generated members grouped by target class.
    pub mixin_members: Vec<ClassMixinContribution>,
    /// Support types needed by generated output.
    pub support_types: Vec<String>,
    /// Top-level generated functions.
    pub top_level_functions: Vec<String>,
}

impl PluginContribution {
    /// Appends one generated member to the mixin block for the given class.
    pub fn push_mixin_member(&mut self, class_name: impl Into<String>, member: impl Into<String>) {
        let class_name = class_name.into();
        let member = member.into();

        if let Some(existing) = self
            .mixin_members
            .iter_mut()
            .find(|entry| entry.class_name == class_name)
        {
            existing.members.push(member);
        } else {
            self.mixin_members.push(ClassMixinContribution {
                class_name,
                members: vec![member],
            });
        }
    }

    /// Returns `true` if the contribution contains no generated fragments.
    pub fn is_empty(&self) -> bool {
        self.shared_helpers.is_empty()
            && self.mixin_members.is_empty()
            && self.support_types.is_empty()
            && self.top_level_functions.is_empty()
    }
}
