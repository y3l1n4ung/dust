use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::model::RouterSpec;

use super::{patterns::route_switch_pattern, shell::effective_shell};

#[derive(Serialize)]
struct ShellConsistencyContext {
    entries: String,
}

#[derive(Serialize)]
struct PageBuilderContext {
    cases: String,
}

#[derive(Serialize)]
struct PageBuilderCaseContext<'a> {
    pattern: String,
    name: &'a str,
    transition_arg: String,
    fullscreen_dialog: bool,
    maintain_state: bool,
    child: String,
}

pub(super) fn render_shell_consistency_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "shell_consistency",
        include_str!("templates/shell_consistency.jinja"),
        ShellConsistencyContext {
            entries: spec
                .routes
                .iter()
                .map(|route| {
                    format!(
                        "  {}: {},\n",
                        route.page_class,
                        effective_shell(route, &spec.routes).unwrap_or("null")
                    )
                })
                .collect(),
        },
    ));
    out.push_str("\n\n");
}

pub(super) fn render_page_builder(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "page_builder",
        include_str!("templates/page_builder.jinja"),
        PageBuilderContext {
            cases: spec
                .routes
                .iter()
                .map(|route| render_page_builder_case(spec, route))
                .collect(),
        },
    ));
    out.push('\n');
}

fn render_page_builder_case(spec: &RouterSpec, route: &crate::plugin::model::RouteSpec) -> String {
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
    render_template(
        "page_builder_case",
        include_str!("templates/page_builder_case.jinja"),
        PageBuilderCaseContext {
            pattern: route_switch_pattern(route, None),
            name: &route.name,
            transition_arg: route
                .annotation
                .transition
                .as_ref()
                .map(|transition| format!("      transition: {transition},\n"))
                .unwrap_or_default(),
            fullscreen_dialog: route.annotation.fullscreen_dialog,
            maintain_state: route.annotation.maintain_state,
            child,
        },
    ) + "\n"
}
