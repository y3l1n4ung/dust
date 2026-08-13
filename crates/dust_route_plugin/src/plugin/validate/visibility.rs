use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, ImportIr, ParamKind};

use crate::plugin::model::RouteAnnotation;

use super::route_constructor;

/// Validates shell and guard type names are visible to generated code.
pub(super) fn validate_visible_route_types(
    library: &DartFileIr,
    class: &ClassIr,
    route: &RouteAnnotation,
    local_classes: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(shell) = route.shell.as_deref()
        && !is_visible_type(library, local_classes, shell)
    {
        diagnostics.push(Diagnostic::error(format!(
            "route shell `{shell}` on `{}` must be declared in the same library or imported",
            class.name
        )));
    } else if let Some(shell) = route.shell.as_deref()
        && let Some(shell_class) = local_class(library, shell)
    {
        validate_local_shell_constructor(shell_class, class, diagnostics);
    }
    for guard in &route.guards {
        if !is_visible_type(library, local_classes, guard) {
            diagnostics.push(Diagnostic::error(format!(
                "route guard `{guard}` on `{}` must be declared in the same library or imported",
                class.name
            )));
        }
    }
}

/// Returns true when a type name is local or imported by the route library.
pub(super) fn is_visible_type(
    library: &DartFileIr,
    local_classes: &HashSet<&str>,
    name: &str,
) -> bool {
    local_classes.contains(name)
        || library
            .import_directives
            .iter()
            .any(|import| import_exposes_type(import, name))
        || library.import_directives.is_empty()
            && library.imports.iter().any(|uri| is_user_type_import(uri))
}

/// Returns a local class by exact name.
fn local_class<'a>(library: &'a DartFileIr, name: &str) -> Option<&'a ClassIr> {
    library.classes.iter().find(|class| class.name == name)
}

/// Validates local shell constructors match the generated `Shell(child: page)` call.
fn validate_local_shell_constructor(
    shell: &ClassIr,
    route: &ClassIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(constructor) = route_constructor(shell) else {
        diagnostics.push(shell_constructor_diagnostic(shell, route));
        return;
    };
    let Some(child) = constructor
        .params
        .iter()
        .find(|param| param.name == "child" && param.kind == ParamKind::Named)
    else {
        diagnostics.push(shell_constructor_diagnostic(shell, route));
        return;
    };
    if !child.ty.is_named("Widget") || child.ty.is_nullable() || child.has_default {
        diagnostics.push(shell_constructor_diagnostic(shell, route));
    }
}

/// Builds the local shell constructor diagnostic with a concrete fix.
fn shell_constructor_diagnostic(shell: &ClassIr, route: &ClassIr) -> Diagnostic {
    Diagnostic::error(format!(
        "route shell `{}` on `{}` needs an unnamed generative constructor with a required named `Widget child` parameter, for example `const {}({{required Widget child, super.key}})`",
        shell.name, route.name, shell.name
    ))
}

/// Returns true when an import directive exposes the requested type.
fn import_exposes_type(import: &ImportIr, name: &str) -> bool {
    is_user_type_import(&import.uri)
        && !import.hide.iter().any(|hidden| hidden == name)
        && (import.show.is_empty() || import.show.iter().any(|shown| shown == name))
}

/// Returns true for imports that may contain app-defined route types.
fn is_user_type_import(uri: &str) -> bool {
    !uri.starts_with("dart:")
        && !uri.starts_with("package:flutter/")
        && !uri.starts_with("package:dust_flutter/")
        && !uri.starts_with("package:dust_dart/")
}
