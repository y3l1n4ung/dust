use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{
    BuiltinType, ClassIr, ConstructorIr, ConstructorParamIr, DartFileIr, ImportIr, ParamKind,
    TypeIr,
};

use super::{
    model::RouteAnnotation,
    parse::{route_annotation, route_config},
};

/// Validates all `@AppRoute` pages in a lowered Dart library.
pub(crate) fn validate_library_routes(library: &DartFileIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    let local_classes = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        let has_route = class
            .configs
            .iter()
            .any(|config| config.symbol.0 == "dust_flutter::AppRoute");
        if !has_route {
            continue;
        }
        let Some(config) = route_config(&class.configs) else {
            diagnostics.push(Diagnostic::error(format!(
                "`@AppRoute` on `{}` requires a string path argument",
                class.name
            )));
            continue;
        };
        let route = route_annotation(config);

        if !route.path.starts_with('/') {
            diagnostics.push(Diagnostic::error(format!(
                "route `{}` path `{}` must be absolute",
                class.name, route.path
            )));
        }
        if !paths.insert(route.path.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route path `{}`",
                route.path
            )));
        }
        if let Some(name) = &route.name
            && !names.insert(name.clone())
        {
            diagnostics.push(Diagnostic::error(format!("duplicate route name `{name}`")));
        }

        validate_route_params(
            library,
            class,
            &route.path,
            &local_classes,
            &mut diagnostics,
        );
        validate_visible_route_types(library, class, &route, &local_classes, &mut diagnostics);
    }

    diagnostics
}

/// Validates shell and guard type names are visible to generated code.
fn validate_visible_route_types(
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
fn is_visible_type(library: &DartFileIr, local_classes: &HashSet<&str>, name: &str) -> bool {
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

/// Validates constructor parameters used by a route path and query string.
fn validate_route_params(
    library: &DartFileIr,
    class: &ClassIr,
    path: &str,
    local_classes: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(constructor) = route_constructor(class) else {
        diagnostics.push(Diagnostic::error(format!(
            "route page `{}` needs an unnamed generative constructor",
            class.name
        )));
        return;
    };

    let path_params = path_params(path);
    for path_param in &path_params {
        let Some(param) = constructor
            .params
            .iter()
            .find(|param| &param.name == path_param)
        else {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `:{path_param}` on `{}` has no matching constructor parameter",
                class.name
            )));
            continue;
        };
        if param.ty.is_nullable() {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `{}` on `{}` must be required and non-nullable",
                param.name, class.name
            )));
        }
        if param.has_default {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `{}` on `{}` cannot use a constructor default",
                param.name, class.name
            )));
        }
        if !is_supported_url_primitive(&param.ty) {
            diagnostics.push(unsupported_param_diagnostic(&class.name, param));
        }
    }

    for param in &constructor.params {
        if param.name == "key" || path_params.contains(&param.name) {
            continue;
        }
        if param.has_default && param.default_value_source.is_none() {
            diagnostics.push(Diagnostic::error(format!(
                "route query parameter `{}` on `{}` has a constructor default that Dust could not preserve",
                param.name, class.name
            )));
        }
        if !is_supported_route_query_param(
            library,
            local_classes,
            &param.ty,
            class,
            param,
            diagnostics,
        ) {
            diagnostics.push(unsupported_param_diagnostic(&class.name, param));
        }
    }
}

/// Returns the unnamed generative constructor for a route page.
fn route_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
}

/// Extracts `:param` placeholders from a route path.
fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns true when a type can round-trip through route URLs.
fn is_supported_url_primitive(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Builtin {
            kind: BuiltinType::String | BuiltinType::Int | BuiltinType::Double | BuiltinType::Bool,
            ..
        }
    )
}

/// Returns true when a constructor parameter can round-trip through a query string.
fn is_supported_route_query_param(
    library: &DartFileIr,
    local_classes: &HashSet<&str>,
    ty: &TypeIr,
    class: &ClassIr,
    param: &ConstructorParamIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if is_supported_url_primitive(ty)
        || ty.is_named("DateTime")
        || ty.is_named("Uri")
        || is_supported_repeated_query_param(ty)
    {
        return true;
    }
    if matches!(ty, TypeIr::Named { name, args, .. }
        if args.is_empty() && is_visible_enum_like_type(library, local_classes, name))
    {
        return true;
    }
    if matches!(ty, TypeIr::Named { name, .. } if name.as_ref() == "List") {
        diagnostics.push(Diagnostic::error(format!(
            "route repeated query parameter `{}` on `{}` must be `List<String>` or `List<int>`",
            param.name, class.name
        )));
        return true;
    }
    false
}

/// Returns true when a named query type can be referenced as an enum by generated code.
fn is_visible_enum_like_type(
    library: &DartFileIr,
    local_classes: &HashSet<&str>,
    name: &str,
) -> bool {
    library.enums.iter().any(|enum_| enum_.name == name)
        || is_visible_type(library, local_classes, name)
}

/// Returns true for the repeated query shapes supported by generated routing.
fn is_supported_repeated_query_param(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Named { name, args, .. }
            if name.as_ref() == "List"
                && args.len() == 1
                && (args[0].is_builtin(BuiltinType::String)
                    || args[0].is_builtin(BuiltinType::Int))
    )
}

/// Builds a diagnostic for unsupported route constructor parameter types.
fn unsupported_param_diagnostic(class_name: &str, param: &ConstructorParamIr) -> Diagnostic {
    Diagnostic::error(format!(
        "route parameter `{}` on `{class_name}` must be a URL type (`String`, `int`, `double`, `bool`, `DateTime`, `Uri`, enum, `List<String>`, or `List<int>`)",
        param.name
    ))
}
