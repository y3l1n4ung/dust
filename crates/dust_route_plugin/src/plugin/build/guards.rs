use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, ParamKind};
use dust_plugin_api::SymbolPlan;

use crate::plugin::{
    constants::GUARDS_ANALYSIS_KEY,
    model::{GuardFact, GuardParamSpec, GuardSpec, RouteSpec, RouterFieldSpec},
};

use super::routes::{parse_route_type_name, route_constructor};

/// Builds guard specs required by the route set and resolves router injections.
pub(super) fn build_guard_specs(
    library: &DartFileIr,
    plan: &SymbolPlan,
    routes: &[RouteSpec],
    router_fields: &[RouterFieldSpec],
) -> Result<Vec<GuardSpec>, Vec<Diagnostic>> {
    let guard_names = guard_classes(routes);
    let local_guards = local_guard_specs(library);
    let workspace_guards = workspace_guard_specs(plan);
    let mut diagnostics = Vec::new();
    let mut specs = Vec::new();

    for guard in guard_names {
        let spec = local_guards
            .get(&guard)
            .or_else(|| workspace_guards.get(&guard))
            .cloned()
            .unwrap_or_else(|| GuardSpec {
                class_name: guard.clone(),
                params: Vec::new(),
            });
        match resolve_guard_injection(spec, router_fields) {
            Ok(spec) => specs.push(spec),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(specs)
    } else {
        Err(diagnostics)
    }
}

/// Resolves guard constructor dependencies against router fields.
fn resolve_guard_injection(
    mut guard: GuardSpec,
    router_fields: &[RouterFieldSpec],
) -> Result<GuardSpec, Diagnostic> {
    let guard_name = guard.class_name.clone();
    for param in &mut guard.params {
        if param.has_default {
            continue;
        }
        let matches = router_fields
            .iter()
            .filter(|field| field.type_name == param.type_name)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [field] => param.inject_field = Some(field.name.clone()),
            [] => {
                return Err(Diagnostic::error(format!(
                    "route guard `{guard_name}` needs `{}` but router has no field of type `{}`",
                    param.name, param.type_name
                )));
            }
            _ => {
                return Err(Diagnostic::error(format!(
                    "route guard `{guard_name}` dependency `{}` has multiple router fields of type `{}`",
                    param.name, param.type_name
                )));
            }
        }
    }
    Ok(guard)
}

/// Returns the unique guard class names referenced by routes.
fn guard_classes(routes: &[RouteSpec]) -> Vec<String> {
    let mut guards = routes
        .iter()
        .flat_map(|route| route.annotation.guards.iter().cloned())
        .collect::<Vec<_>>();
    guards.sort();
    guards.dedup();
    guards
}

/// Builds guard specs for classes declared in the current library.
fn local_guard_specs(library: &DartFileIr) -> HashMap<String, GuardSpec> {
    library
        .classes
        .iter()
        .map(|class| (class.name.clone(), guard_spec_from_class(class)))
        .collect()
}

/// Builds guard specs from workspace analysis facts.
fn workspace_guard_specs(plan: &SymbolPlan) -> HashMap<String, GuardSpec> {
    plan.workspace_string_set(GUARDS_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<GuardFact>(value).ok())
        .filter_map(guard_spec_from_fact)
        .map(|spec| (spec.class_name.clone(), spec))
        .collect()
}

/// Builds a guard spec from a lowered local class.
fn guard_spec_from_class(class: &ClassIr) -> GuardSpec {
    let params = route_constructor(class)
        .map(|constructor| {
            constructor
                .params
                .iter()
                .filter_map(|param| {
                    Some(GuardParamSpec {
                        name: param.name.clone(),
                        type_name: param.ty.name()?.to_owned(),
                        is_named: matches!(param.kind, ParamKind::Named),
                        has_default: param.has_default,
                        inject_field: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    GuardSpec {
        class_name: class.name.clone(),
        params,
    }
}

/// Builds a guard spec from a serialized workspace guard fact.
fn guard_spec_from_fact(fact: GuardFact) -> Option<GuardSpec> {
    let params = fact
        .params
        .iter()
        .map(|param| {
            Some(GuardParamSpec {
                name: param.name.clone(),
                type_name: parse_route_type_name(param.type_source.as_deref())?,
                is_named: param.is_named,
                has_default: param.has_default,
                inject_field: None,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(GuardSpec {
        class_name: fact.class_name,
        params,
    })
}
