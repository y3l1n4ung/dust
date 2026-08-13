use std::collections::BTreeSet;

use dust_ir::TypeIr;

use crate::plugin::model::RouteSpec;

use super::imports::RouteImportKind;

/// Symbols from route annotations that may require replaying page-library imports.
#[derive(Debug, Default)]
pub(super) struct RouteImportReferences {
    /// Import prefixes referenced as `prefix.Symbol`.
    pub(super) prefixes: BTreeSet<String>,
    /// Unprefixed type or constructor identifiers referenced by generated code.
    pub(super) names: BTreeSet<String>,
}

impl RouteImportReferences {
    /// Returns true when no annotation symbol needs an import from the page library.
    pub(super) fn is_empty(&self) -> bool {
        self.prefixes.is_empty() && self.names.is_empty()
    }
}

/// Collects route annotation references for one generated file family.
pub(super) fn route_import_references(
    route: &RouteSpec,
    kind: RouteImportKind,
) -> RouteImportReferences {
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
