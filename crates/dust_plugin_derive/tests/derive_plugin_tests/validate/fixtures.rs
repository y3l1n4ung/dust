//! Library fixtures the validation tests build on.

use super::*;

pub(super) fn validation_library() -> DartFileIr {
    let mut class = class("SignupRequest");
    class.fields = vec![
        field("email", TypeIr::string(), vec![validate("(email: true)")]),
        field(
            "age",
            TypeIr::int(),
            vec![validate("(range: Range(min: 18, max: 120))")],
        ),
        field(
            "password",
            TypeIr::string(),
            vec![
                validate("(length: Length(min: 8), message: 'At least 8 characters')"),
                validate("(regex: r'^(?=.*[A-Z]).+$', message: 'Need uppercase')"),
            ],
        ),
        field(
            "confirmPassword",
            TypeIr::string(),
            vec![validate("(mustMatch: 'password')")],
        ),
    ];
    library(vec![class])
}

pub(super) fn nested_library() -> DartFileIr {
    let mut address = class("Address");
    address.fields = vec![field(
        "zip",
        TypeIr::string(),
        vec![validate("(length: Length(exact: 5))")],
    )];

    let mut profile = class("Profile");
    profile.fields = vec![
        field(
            "bio",
            TypeIr::string().nullable(),
            vec![validate("(length: Length(max: 200))")],
        ),
        field(
            "address",
            TypeIr::named("Address"),
            vec![validate("(nested: true)")],
        ),
        field(
            "phone",
            TypeIr::string(),
            vec![validate("(custom: Profile.checkPhone)")],
        ),
    ];
    library(vec![address, profile])
}

pub(super) fn name(source: &str) -> NameIr {
    NameIr {
        source: source.to_owned(),
        short: source.to_owned(),
        prefix: None,
        span: span(0, source.len() as u32),
    }
}

pub(super) fn flutter_symbol_plan() -> SymbolPlan {
    let mut analysis = WorkspaceAnalysisBuilder::default();
    analysis.add_string_set_value(PACKAGE_FEATURES_ANALYSIS_KEY, PACKAGE_FEATURE_FLUTTER);
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(analysis.build()));
    plan
}
