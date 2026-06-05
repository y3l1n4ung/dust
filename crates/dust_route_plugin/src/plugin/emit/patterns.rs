use std::collections::BTreeSet;

use crate::plugin::model::RouteSpec;

pub(super) fn route_switch_pattern(
    route: &RouteSpec,
    bound_params: Option<&BTreeSet<String>>,
) -> String {
    if route.params.is_empty() {
        format!("{}()", route.route_class)
    } else {
        let params = route
            .params
            .iter()
            .map(|param| {
                let should_bind = match bound_params {
                    Some(names) => names.contains(&param.name),
                    None => true,
                };
                if should_bind {
                    format!("{}: final {}", param.name, param.name)
                } else {
                    format!("{}: _", param.name)
                }
            })
            .collect::<Vec<_>>();
        let inline = format!("{}({})", route.route_class, params.join(", "));
        if inline.len() <= 76 {
            return inline;
        }
        format!(
            "{}(\n      {},\n    )",
            route.route_class,
            params.join(",\n      ")
        )
    }
}
