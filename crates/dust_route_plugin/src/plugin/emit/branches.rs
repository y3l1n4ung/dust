use std::collections::{BTreeMap, BTreeSet};

use dust_dart_emit::dart_string_literal;

use crate::plugin::model::RouterSpec;

use super::shell::effective_branch;

/// One generated branch constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BranchConstant {
    /// Dart identifier for the generated constant.
    pub(super) name: String,
    /// Source branch value from route annotations.
    pub(super) value: String,
}

/// Returns the generated branch constant expression for a branch value.
pub(super) fn branch_constant_expr(spec: &RouterSpec, branch: &str) -> String {
    branch_constants(spec)
        .into_iter()
        .find(|constant| constant.value == branch)
        .map(|constant| constant.name)
        .unwrap_or_else(|| dart_string_literal(branch))
}

/// Returns stable generated constants for every effective route branch.
pub(super) fn branch_constants(spec: &RouterSpec) -> Vec<BranchConstant> {
    let branches = spec
        .routes
        .iter()
        .filter_map(|route| effective_branch(route, &spec.routes).map(str::to_owned))
        .collect::<BTreeSet<_>>();
    let mut bases = BTreeMap::<String, Vec<String>>::new();
    for branch in branches {
        bases
            .entry(branch_constant_base(spec, &branch))
            .or_default()
            .push(branch);
    }

    bases
        .into_iter()
        .flat_map(|(base, branches)| {
            let needs_hash = branches.len() > 1;
            branches.into_iter().map(move |branch| BranchConstant {
                name: if needs_hash {
                    format!("{base}H{:08x}", stable_hash(&branch))
                } else {
                    base.clone()
                },
                value: branch,
            })
        })
        .collect()
}

/// Builds a readable branch constant identifier before collision disambiguation.
fn branch_constant_base(spec: &RouterSpec, branch: &str) -> String {
    let prefix = spec
        .route_branch_function
        .strip_suffix("RouteBranch")
        .unwrap_or("app");
    format!("{prefix}Branch{}", branch_identifier_suffix(branch))
}

/// Converts arbitrary branch text into an identifier suffix.
fn branch_identifier_suffix(branch: &str) -> String {
    let suffix = branch
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(upper_camel_part)
        .collect::<String>();
    if suffix.is_empty() {
        "Value".to_owned()
    } else {
        suffix
    }
}

/// Converts one branch-name fragment to UpperCamelCase.
fn upper_camel_part(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Returns a small deterministic hash for branch constant collisions.
fn stable_hash(value: &str) -> u32 {
    const FNV_OFFSET: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;
    value.bytes().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
    })
}
