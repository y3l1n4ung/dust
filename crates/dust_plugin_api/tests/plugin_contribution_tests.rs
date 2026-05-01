use dust_plugin_api::{ClassMixinContribution, PluginContribution};

#[test]
fn plugin_contribution_empty_check_matches_sections() {
    let mut contribution = PluginContribution::default();
    assert!(contribution.is_empty());

    contribution.push_mixin_member("User", "String toString() => 'User()';");
    assert!(!contribution.is_empty());
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
