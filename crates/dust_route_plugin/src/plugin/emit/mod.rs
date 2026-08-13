/// Renders typed navigation action factory helpers.
mod action_helpers;
/// Renders split route output files.
mod files;
/// Shared formatting helpers for generated Dart code.
mod formatting;
/// Renders imports used by split route generated files.
mod imports;
/// Renders the generated route metadata tree.
mod metadata;
/// Template contexts for generated route metadata.
mod metadata_context;
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

pub(crate) use files::render_route_generated_files;

/// Normalizes generated no-transition helpers to the shared Flutter runtime.
pub(super) fn normalize_private_transition_helper(transition: &str) -> String {
    transition
        .replace("_$NoTransitionBuilder", "GeneratedNoTransitionBuilder")
        .replace("_NoTransitionBuilder", "GeneratedNoTransitionBuilder")
}
