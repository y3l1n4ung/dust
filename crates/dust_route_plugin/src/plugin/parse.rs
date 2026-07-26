use dust_ir::{ConfigApplicationIr, NormalizedConfigIr, RouteConfigIr, RouterConfigIr};

use super::model::RouteAnnotation;

/// Returns resolver-normalized `AppRoute` configuration.
pub(crate) fn route_config(configs: &[ConfigApplicationIr]) -> Option<&RouteConfigIr> {
    configs.iter().find_map(|config| match &config.normalized {
        Some(NormalizedConfigIr::Route(route)) => Some(route),
        _ => None,
    })
}

/// Copies resolver-normalized route configuration into a plugin route fact.
pub(crate) fn route_annotation(config: &RouteConfigIr) -> RouteAnnotation {
    RouteAnnotation {
        path: config.path.clone(),
        name: config.name.clone(),
        result_type: config.result_type.clone(),
        shell: config.shell.clone(),
        guards: config.guards.clone(),
        guards_configured: config.guards_configured,
        transition: config.transition.clone(),
        fullscreen_dialog: config.fullscreen_dialog,
        maintain_state: config.maintain_state,
    }
}

/// Returns resolver-normalized `AppRouter` configuration.
pub(crate) fn router_config(configs: &[ConfigApplicationIr]) -> Option<&RouterConfigIr> {
    configs.iter().find_map(|config| match &config.normalized {
        Some(NormalizedConfigIr::Router(router)) => Some(router),
        _ => None,
    })
}
