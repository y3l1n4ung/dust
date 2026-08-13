use std::collections::BTreeSet;
use std::path::Path;

use dust_ir::{DartFileIr, TypeIr};

use crate::plugin::model::{RouteImport, RouteSpec, RouterSpec};

use super::formatting::package_import_uri;

/// Selects which generated route file dependencies should be imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RouteImportKind {
    /// Imports used by route metadata.
    Metadata,
    /// Imports used by page-building runtime code.
    Runtime,
    /// Imports used by typed route data and navigation helper signatures.
    RouteTypes,
}

/// Renders imports required by one generated route file family.
pub(super) fn render_route_imports(
    library: &DartFileIr,
    spec: &RouterSpec,
    kind: RouteImportKind,
) -> String {
    let current_import = package_import_uri(library);
    let mut imports = BTreeSet::new();
    if matches!(kind, RouteImportKind::Metadata | RouteImportKind::Runtime) {
        imports.insert(format!(
            "import '{}';\n",
            source_import_from_generated_dir(library)
        ));
    }
    for route in &spec.routes {
        if matches!(kind, RouteImportKind::Metadata | RouteImportKind::Runtime)
            && let Some(import) = &route.import_uri
            && Some(import.as_str()) != current_import.as_deref()
            && !is_internal_route_import(import)
        {
            imports.insert(format!("import '{import}';\n"));
        }
        let references = route_import_references(route, kind);
        if matches!(kind, RouteImportKind::RouteTypes)
            && route_type_needs_source_import(current_import.as_deref(), route, &references)
        {
            imports.insert(format!(
                "import '{}';\n",
                source_import_from_generated_dir(library)
            ));
        }
        for import in &route.imports {
            if Some(import.uri.as_str()) == current_import.as_deref()
                || is_internal_route_import(&import.uri)
                || !route_import_is_referenced(import, &references)
            {
                continue;
            }
            imports.insert(render_import(import));
        }
    }
    let imports = imports.into_iter().collect::<String>();
    if imports.is_empty() {
        imports
    } else {
        format!("{imports}\n")
    }
}

/// Symbols from route annotations that may require replaying page-library imports.
#[derive(Debug, Default)]
struct RouteImportReferences {
    /// Import prefixes referenced as `prefix.Symbol`.
    prefixes: BTreeSet<String>,
    /// Unprefixed type or constructor identifiers referenced by generated code.
    names: BTreeSet<String>,
}

impl RouteImportReferences {
    /// Returns true when no annotation symbol needs an import from the page library.
    fn is_empty(&self) -> bool {
        self.prefixes.is_empty() && self.names.is_empty()
    }
}

/// Collects route annotation references for one generated file family.
fn route_import_references(route: &RouteSpec, kind: RouteImportKind) -> RouteImportReferences {
    let mut references = RouteImportReferences::default();
    match kind {
        RouteImportKind::Metadata => {
            if let Some(shell) = &route.annotation.shell {
                collect_expression_references(shell, &mut references);
            }
            for guard in &route.annotation.guards {
                collect_expression_references(guard, &mut references);
            }
            if let Some(transition) = &route.annotation.transition {
                collect_expression_references(transition, &mut references);
            }
        }
        RouteImportKind::Runtime => {
            if let Some(shell) = &route.annotation.shell {
                collect_expression_references(shell, &mut references);
            }
            if let Some(transition) = &route.annotation.transition {
                collect_expression_references(transition, &mut references);
            }
        }
        RouteImportKind::RouteTypes => {
            collect_type_references(&route.result_type, &mut references);
            for param in &route.params {
                collect_type_ir_references(&param.ty, &mut references);
            }
        }
    }
    references
}

/// Returns true when generated route types need the user route entrypoint.
fn route_type_needs_source_import(
    current_import: Option<&str>,
    route: &RouteSpec,
    references: &RouteImportReferences,
) -> bool {
    if references.is_empty() {
        return false;
    }
    route.import_uri.is_none()
        || route.imports.iter().any(|import| {
            Some(import.uri.as_str()) == current_import
                && route_current_import_is_referenced(import, references)
        })
}

/// Returns true when a route page import of the user entrypoint exposes references.
fn route_current_import_is_referenced(
    import: &RouteImport,
    references: &RouteImportReferences,
) -> bool {
    if let Some(prefix) = &import.prefix {
        return references.prefixes.contains(prefix);
    }
    if !import.show.is_empty() {
        return import
            .show
            .iter()
            .any(|name| references.names.contains(name) && !import.hide.contains(name));
    }
    references
        .names
        .iter()
        .any(|name| !import.hide.contains(name))
}

/// Returns true when a page-library import exposes a generated-code reference.
fn route_import_is_referenced(import: &RouteImport, references: &RouteImportReferences) -> bool {
    if let Some(prefix) = &import.prefix {
        return references.prefixes.contains(prefix);
    }
    if references.names.is_empty() {
        return false;
    }
    if !import.show.is_empty() {
        return import
            .show
            .iter()
            .any(|name| references.names.contains(name) && !import.hide.contains(name));
    }
    import_uri_type_name(&import.uri)
        .is_some_and(|name| references.names.contains(&name) && !import.hide.contains(&name))
}

/// Returns whether an import is generated by the route plugin itself.
fn is_internal_route_import(import: &str) -> bool {
    matches!(
        import,
        "route.g.dart"
            | "routing_core.dart"
            | "route/routes.g.dart"
            | "route/runtime.g.dart"
            | "route/paths.g.dart"
            | "route/metadata.g.dart"
            | "route/navigation.g.dart"
    )
}

/// Returns the router source import URI from generated files under `route/`.
fn source_import_from_generated_dir(library: &DartFileIr) -> String {
    let source_path = Path::new(&library.source_path);
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("route.dart");
    format!("../{source_name}")
}

/// Records type and constructor names that can appear in route annotations.
fn collect_expression_references(source: &str, references: &mut RouteImportReferences) {
    let tokens = dart_identifier_tokens(source);
    for (index, token) in tokens.iter().enumerate() {
        if tokens.get(index + 1).is_some_and(|next| next == ".") {
            references.prefixes.insert(token.clone());
            continue;
        }
        if index > 0
            && tokens
                .get(index - 1)
                .is_some_and(|previous| previous == ".")
        {
            continue;
        }
        if is_public_identifier(token) {
            references.names.insert(token.clone());
        }
    }
}

/// Records result-type symbols used by generated path and navigator signatures.
fn collect_type_references(source: &str, references: &mut RouteImportReferences) {
    let source = source.trim().trim_end_matches('?').trim();
    if matches!(
        source,
        "void" | "bool" | "int" | "double" | "String" | "Object" | "dynamic"
    ) {
        return;
    }
    collect_expression_references(source, references);
}

/// Records non-core route parameter type symbols used by generated route data.
fn collect_type_ir_references(ty: &TypeIr, references: &mut RouteImportReferences) {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => {}
        TypeIr::Named { name, args, .. }
            if name.as_ref() == "DateTime" || name.as_ref() == "Uri" =>
        {
            for arg in args {
                collect_type_ir_references(arg, references);
            }
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => {
            for arg in args {
                collect_type_ir_references(arg, references);
            }
        }
        TypeIr::Named { name, args, .. } => {
            collect_expression_references(name, references);
            for arg in args {
                collect_type_ir_references(arg, references);
            }
        }
        TypeIr::Function { signature, .. }
        | TypeIr::Record {
            shape: signature, ..
        } => {
            collect_type_references(signature, references);
        }
    }
}

/// Tokenizes enough Dart expression syntax to detect prefixed route references.
fn dart_identifier_tokens(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in source.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if ch == '.' {
                tokens.push(".".to_owned());
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Keeps constructors and type names while ignoring lower-case values and args.
fn is_public_identifier(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_uppercase())
}

/// Infers a conventional public type name from a Dart import file name.
fn import_uri_type_name(uri: &str) -> Option<String> {
    let stem = uri.rsplit('/').next()?.strip_suffix(".dart")?;
    Some(upper_camel(stem))
}

/// Converts snake, kebab, and spaced file stems to UpperCamelCase.
fn upper_camel(value: &str) -> String {
    value
        .split(|ch: char| ch == '_' || ch == '-' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Renders a Dart import while preserving prefix, deferred, show, and hide clauses.
fn render_import(import: &RouteImport) -> String {
    let mut rendered = format!("import '{}'", import.uri);
    if let Some(prefix) = &import.prefix {
        if import.is_deferred {
            rendered.push_str(" deferred");
        }
        rendered.push_str(&format!(" as {prefix}"));
    }
    if !import.show.is_empty() {
        rendered.push_str(&format!(" show {}", import.show.join(", ")));
    }
    if !import.hide.is_empty() {
        rendered.push_str(&format!(" hide {}", import.hide.join(", ")));
    }
    rendered.push_str(";\n");
    rendered
}
