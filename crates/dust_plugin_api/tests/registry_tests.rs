use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ClassKindIr, LibraryIr, SpanIr, SymbolId};
use dust_plugin_api::{
    ClassMixinContribution, DustPlugin, PluginContribution, PluginRegistry, SymbolPlan,
};
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

fn sample_library() -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 80),
            fields: Vec::new(),
            constructors: Vec::new(),
            traits: Vec::new(),
            serde: None,
        }],
    }
}

struct FakePlugin {
    name: &'static str,
    traits: Vec<SymbolId>,
    configs: Vec<SymbolId>,
    requested: Vec<&'static str>,
}

impl DustPlugin for FakePlugin {
    fn plugin_name(&self) -> &'static str {
        self.name
    }

    fn claimed_traits(&self) -> Vec<SymbolId> {
        self.traits.clone()
    }

    fn claimed_configs(&self) -> Vec<SymbolId> {
        self.configs.clone()
    }

    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        self.requested
            .iter()
            .map(|name| (*name).to_owned())
            .collect()
    }

    fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        PluginContribution::default()
    }
}

#[test]
fn registry_rejects_duplicate_trait_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: vec![SymbolId::new("derive_annotation::Debug")],
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: vec![SymbolId::new("derive_annotation::Debug")],
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("trait symbol `derive_annotation::Debug` is already owned")
    );
}

#[test]
fn registry_rejects_duplicate_config_ownership() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: Vec::new(),
            configs: vec![SymbolId::new("derive_serde_annotation::SerDe")],
            requested: Vec::new(),
        }))
        .unwrap();

    let error = registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: Vec::new(),
            configs: vec![SymbolId::new("derive_serde_annotation::SerDe")],
            requested: Vec::new(),
        }))
        .unwrap_err();

    assert!(
        error
            .message
            .contains("config symbol `derive_serde_annotation::SerDe` is already owned")
    );
}

#[test]
fn symbol_plan_preserves_first_seen_order_and_dedupes() {
    let library = sample_library();
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_a",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: vec!["_$UserToJson", "_undefined"],
        }))
        .unwrap();
    registry
        .register(Box::new(FakePlugin {
            name: "plugin_b",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: vec!["_undefined", "_$UserFromJson"],
        }))
        .unwrap();

    let plan = registry.build_symbol_plan(&library);

    let names = plan
        .reserved()
        .iter()
        .map(|symbol| symbol.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["_$UserToJson", "_undefined", "_$UserFromJson"]);
    assert!(plan.contains("_undefined"));
}

#[test]
fn plugin_contribution_empty_check_matches_sections() {
    let mut contribution = PluginContribution::default();
    assert!(contribution.is_empty());

    contribution.push_mixin_member("User", "String toString() => 'User()';");
    assert!(!contribution.is_empty());
}

#[test]
fn plugin_names_follow_registration_order() {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "derive",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();
    registry
        .register(Box::new(FakePlugin {
            name: "serde",
            traits: Vec::new(),
            configs: Vec::new(),
            requested: Vec::new(),
        }))
        .unwrap();

    assert_eq!(registry.plugin_names(), vec!["derive", "serde"]);
}

#[test]
fn push_mixin_member_groups_members_by_class() {
    let mut contribution = PluginContribution::default();
    contribution.push_mixin_member("User", "String toString() => 'User()';");
    contribution.push_mixin_member("User", "User copyWith({String? id}) => User();");
    contribution.push_mixin_member("Team", "Team copyWith({String? name}) => Team();");

    assert_eq!(
        contribution.mixin_members,
        vec![
            ClassMixinContribution {
                class_name: "User".to_owned(),
                members: vec![
                    "String toString() => 'User()';".to_owned(),
                    "User copyWith({String? id}) => User();".to_owned(),
                ],
            },
            ClassMixinContribution {
                class_name: "Team".to_owned(),
                members: vec!["Team copyWith({String? name}) => Team();".to_owned()],
            },
        ]
    );
}

#[test]
fn registry_runs_validation_and_emission_in_registration_order() {
    struct OrderPlugin {
        name: &'static str,
    }

    impl DustPlugin for OrderPlugin {
        fn plugin_name(&self) -> &'static str {
            self.name
        }

        fn validate(&self, _library: &LibraryIr) -> Vec<Diagnostic> {
            vec![Diagnostic::note(format!("validated by {}", self.name))]
        }

        fn emit(&self, _library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
            let mut contribution = PluginContribution::default();
            contribution.push_mixin_member("User", format!("// {}", self.name));
            contribution
        }
    }

    let library = sample_library();
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(OrderPlugin { name: "a" }))
        .unwrap();
    registry
        .register(Box::new(OrderPlugin { name: "b" }))
        .unwrap();

    let diagnostics = registry.validate_library(&library);
    let plan = registry.build_symbol_plan(&library);
    let contributions = registry.emit_contributions(&library, &plan);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec!["validated by a", "validated by b"]
    );
    assert_eq!(contributions.len(), 2);
    assert_eq!(contributions[0].mixin_members[0].members[0], "// a");
    assert_eq!(contributions[1].mixin_members[0].members[0], "// b");
}
