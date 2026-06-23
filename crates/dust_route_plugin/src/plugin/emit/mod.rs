use std::collections::BTreeSet;

use dust_dart_emit::render_template;
use dust_ir::DartFileIr;
use serde::Serialize;

use super::model::RouterSpec;

/// Shared formatting helpers for generated Dart code.
mod formatting;
/// Renders the generated route metadata tree.
mod metadata;
/// Renders navigation helpers and guard lookup code.
mod navigation;
/// Renders page builder and shell consistency helpers.
mod page_builder;
/// Renders URI-to-route parser code.
mod parser;
/// Renders route parameter encoder and decoder expressions.
mod parser_decode;
/// Compares and splits route path segments.
mod path;
/// Renders Dart pattern matching fragments for route classes.
mod patterns;
/// Renders stack restoration helpers.
mod restore;
/// Renders generated route data classes.
mod route_classes;
/// Resolves effective shell widgets for nested routes.
mod shell;

use formatting::package_import_uri;
use navigation::render_helpers;
use page_builder::{render_page_builder, render_shell_consistency_helpers};
use parser::render_parser;
use restore::render_restore_stack;
use route_classes::render_route_classes;

/// Template context for the top-level generated route file.
#[derive(Serialize)]
struct RouteFileContext<'a> {
    /// Rendered imports required by workspace routes.
    imports: String,
    /// Optional no-transition builder helper source.
    no_transition_builder: String,
    /// Generated router base class name.
    generated_base_class: &'a str,
    /// Initial generated route class.
    initial_route_class: &'a str,
    /// Optional refresh listenable override.
    refresh_getter: String,
    /// Rendered generated body sections.
    body: String,
}

/// Renders the complete generated route file for a router spec.
pub(crate) fn render_route_generated(library: &DartFileIr, spec: &RouterSpec) -> String {
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
                refresh_getter: render_refresh_getter(spec),
                body,
            },
        )
    )
}

/// Renders the helper transition builder used by no-transition routes.
fn render_no_transition_builder() -> String {
    render_template(
        "no_transition_builder",
        include_str!("templates/no_transition_builder.jinja"),
        (),
    )
}

/// Returns true when any route references the no-transition helper.
fn uses_no_transition_builder(spec: &RouterSpec) -> bool {
    spec.routes.iter().any(|route| {
        route
            .annotation
            .transition
            .as_deref()
            .is_some_and(|transition| transition.contains("_NoTransitionBuilder"))
    })
}

/// Renders the router refresh-listenable override when available.
fn render_refresh_getter(spec: &RouterSpec) -> String {
    spec.refresh_listenable
        .as_ref()
        .map(|field| {
            format!(
                "  @override\n  Listenable? get refreshListenable => (this as {}).{};",
                spec.router_class, field
            )
        })
        .unwrap_or_default()
}
