use std::fmt::Write;

use dust_dart_emit::{DART_ITERABLE, DART_LIST, DART_MAP, DART_SET};
use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, TypeIr};

use crate::features::EQ_SYMBOL;

pub(crate) fn has_trait(class: &ClassIr, symbol: &str) -> bool {
    class
        .traits
        .iter()
        .any(|trait_app| trait_app.symbol.0 == symbol)
}

pub(crate) fn validate_eq_hash(class: &ClassIr) -> Vec<Diagnostic> {
    let _ = class;
    Vec::new()
}

pub(crate) fn emit_shared_helpers(library: &DartFileIr) -> Vec<String> {
    let needs_ordered = library
        .classes
        .iter()
        .flat_map(|class| &class.fields)
        .any(|field| needs_ordered_deep_helper(&field.ty));
    let needs_unordered = library
        .classes
        .iter()
        .flat_map(|class| &class.fields)
        .any(|field| needs_unordered_deep_helper(&field.ty));

    let mut helpers = Vec::new();
    if needs_ordered {
        helpers.push(
            "const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();"
                .to_owned(),
        );
    }
    if needs_unordered {
        helpers.push(
            "const DeepCollectionEquality _unorderedDeepCollectionEquality = DeepCollectionEquality.unordered();"
                .to_owned(),
        );
    }

    helpers
}

pub(crate) fn emit_eq(class: &ClassIr) -> Option<String> {
    if !has_trait(class, EQ_SYMBOL) {
        return None;
    }

    if class.fields.is_empty() {
        return Some(format!(
            "@override\nbool operator ==(Object other) =>\n    identical(this, other) ||\n    other is {} &&\n        runtimeType == other.runtimeType;",
            class.name
        ));
    }

    Some(format!(
        "@override\nbool operator ==(Object other) {{\n  final self = this as {};\n  return identical(this, other) ||\n      other is {} &&\n          runtimeType == other.runtimeType{};\n}}",
        class.name,
        class.name,
        render_equality_comparisons(class)
    ))
}

pub(crate) fn emit_hash_code(class: &ClassIr) -> Option<String> {
    if !has_trait(class, EQ_SYMBOL) {
        return None;
    }

    let self_binding = if class.fields.is_empty() {
        String::new()
    } else {
        format!("  final self = this as {};\n", class.name)
    };
    Some(format!(
        "@override\nint get hashCode {{\n{}  return Object.hashAll([\n    runtimeType,\n{}  ]);\n}}",
        self_binding,
        render_hash_values(class)
    ))
}

fn render_equality_comparisons(class: &ClassIr) -> String {
    let mut out = String::with_capacity(class.fields.len() * 40);
    for field in &class.fields {
        write!(
            out,
            " &&\n          {}",
            render_equality_comparison(&field.name, &field.ty)
        )
        .expect("writing to String cannot fail");
    }
    out
}

fn render_hash_values(class: &ClassIr) -> String {
    let mut out = String::with_capacity(class.fields.len() * 24);
    for field in &class.fields {
        writeln!(out, "    {},", render_hash_value(&field.name, &field.ty))
            .expect("writing to String cannot fail");
    }
    out
}

fn render_equality_comparison(field_name: &str, ty: &TypeIr) -> String {
    match deep_helper_name(ty) {
        Some(helper) => format!("{helper}.equals(other.{field_name}, self.{field_name})"),
        None => format!("other.{field_name} == self.{field_name}"),
    }
}

fn render_hash_value(field_name: &str, ty: &TypeIr) -> String {
    match deep_helper_name(ty) {
        Some(helper) => format!("{helper}.hash(self.{field_name})"),
        None => format!("self.{field_name}"),
    }
}

fn deep_helper_name(ty: &TypeIr) -> Option<&'static str> {
    if needs_unordered_deep_helper(ty) {
        Some("_unorderedDeepCollectionEquality")
    } else if needs_ordered_deep_helper(ty) {
        Some("_deepCollectionEquality")
    } else {
        None
    }
}

fn needs_ordered_deep_helper(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Named { name, .. } if matches!(name.as_ref(), DART_LIST | DART_MAP | DART_ITERABLE))
}

fn needs_unordered_deep_helper(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Named { name, .. } if name.as_ref() == DART_SET)
}
