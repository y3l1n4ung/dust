use std::collections::BTreeSet;

use dust_dart_emit::render_template;
use dust_ir::LibraryIr;
use serde::Serialize;

use super::model::RouterSpec;

mod formatting;
mod metadata;
mod navigation;
mod page_builder;
mod parser;
mod path;
mod patterns;
mod restore;
mod route_classes;
mod shell;

use formatting::{package_import_uri, upper_camel_identifier};
use navigation::render_helpers;
use page_builder::{render_page_builder, render_shell_consistency_helpers};
use parser::render_parser;
use restore::render_restore_stack;
use route_classes::render_route_classes;

#[derive(Serialize)]
struct RouteFileContext<'a> {
    imports: String,
    no_transition_builder: String,
    generated_base_class: &'a str,
    initial_route_class: &'a str,
    guard_factories: String,
    body: String,
}

pub(crate) fn render_route_generated(library: &LibraryIr, spec: &RouterSpec) -> String {
    let current_import = package_import_uri(library);
    let imports = spec
        .routes
        .iter()
        .flat_map(|route| {
            route
                .import_uri
                .iter()
                .map(String::as_str)
                .chain(route.imports.iter().map(String::as_str))
        })
        .filter(|import| Some(*import) != current_import.as_deref())
        .filter(|import| !matches!(*import, "route.g.dart" | "routing_core.dart"))
        .collect::<BTreeSet<_>>();

    let mut body = String::new();
    metadata::render_route_metadata(&mut body, &spec.routes);
    render_route_classes(&mut body, spec);
    render_helpers(&mut body, spec);
    render_restore_stack(&mut body, spec);
    render_parser(&mut body, spec);
    render_shell_consistency_helpers(&mut body, spec);
    render_page_builder(&mut body, spec);

    format!(
        "{}\n",
        render_template(
            "route_file",
            include_str!("templates/route_file.jinja"),
            RouteFileContext {
                imports: imports
                    .into_iter()
                    .map(|import| format!("import '{import}';\n"))
                    .collect::<String>(),
                no_transition_builder: if uses_no_transition_builder(spec) {
                    render_no_transition_builder()
                } else {
                    String::new()
                },
                generated_base_class: &spec.generated_base_class,
                initial_route_class: &spec.initial_route_class,
                guard_factories: render_guard_factories(spec),
                body,
            },
        )
    )
}

fn render_no_transition_builder() -> String {
    render_template(
        "no_transition_builder",
        include_str!("templates/no_transition_builder.jinja"),
        (),
    )
}

fn uses_no_transition_builder(spec: &RouterSpec) -> bool {
    spec.routes.iter().any(|route| {
        route
            .annotation
            .transition
            .as_deref()
            .is_some_and(|transition| transition.contains("_NoTransitionBuilder"))
    })
}

fn render_guard_factories(spec: &RouterSpec) -> String {
    spec.guard_classes
        .iter()
        .map(|guard| format!("\n  {guard} create{}();", upper_camel_identifier(guard)))
        .collect::<Vec<_>>()
        .join("\n")
}
