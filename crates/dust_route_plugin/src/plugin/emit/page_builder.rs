use crate::plugin::model::RouterSpec;

use super::{patterns::route_switch_pattern, shell::effective_shell};

pub(super) fn render_shell_consistency_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str("const Map<Type, Type?> _kAppliedShellsByPage = {\n");
    for route in &spec.routes {
        out.push_str(&format!(
            "  {}: {},\n",
            route.page_class,
            effective_shell(route, &spec.routes).unwrap_or("null")
        ));
    }
    out.push_str("};\n\n");
    out.push_str("bool _shellConsistencyCheck() {\n");
    out.push_str("  bool visit(GeneratedRoute route) {\n");
    out.push_str("    final page = route.page;\n");
    out.push_str("    if (page != null && _kAppliedShellsByPage[page] != route.shell) {\n");
    out.push_str("      return false;\n");
    out.push_str("    }\n");
    out.push_str("    return route.routes.every(visit);\n");
    out.push_str("  }\n");
    out.push_str("  return $appRoutes.every(visit);\n");
    out.push_str("}\n\n");
}

pub(super) fn render_page_builder(out: &mut String, spec: &RouterSpec) {
    out.push_str("Page<void> buildAppRoutePage(AppRoutePath route) {\n");
    out.push_str("  assert(\n");
    out.push_str("    _shellConsistencyCheck(),\n");
    out.push_str("    'Shell mismatch between \\$appRoutes and buildAppRoutePage',\n");
    out.push_str("  );\n");
    out.push_str("  return switch (route) {\n");
    for route in &spec.routes {
        let pattern = route_switch_pattern(route, None);
        let page_args = route
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, param.name))
            .collect::<Vec<_>>()
            .join(", ");
        let child = if page_args.is_empty() {
            format!("const {}()", route.page_class)
        } else {
            format!("{}({page_args})", route.page_class)
        };
        let child = if let Some(shell) = effective_shell(route, &spec.routes) {
            format!("{shell}(child: {child})")
        } else {
            child
        };
        let transition_arg = route
            .annotation
            .transition
            .as_ref()
            .map(|transition| format!("      transition: {transition},\n"))
            .unwrap_or_default();
        out.push_str(&format!(
            "    {pattern} => generatedPage(\n      location: route.location,\n      name: '{}',\n{transition_arg}      fullscreenDialog: {},\n      maintainState: {},\n      child: {child},\n    ),\n",
            route.name, route.annotation.fullscreen_dialog, route.annotation.maintain_state,
        ));
    }
    out.push_str("  };\n}\n");
}
