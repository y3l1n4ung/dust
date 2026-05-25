use std::fmt::Write;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, LibraryIr, TypeIr};

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

pub(crate) fn emit_shared_helpers(library: &LibraryIr) -> Vec<String> {
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

    let mut out = String::with_capacity(class.name.len() * 2 + class.fields.len() * 48 + 128);
    write!(
        &mut out,
        "@override\nbool operator ==(Object other) {{\n  final self = this as {};\n  return identical(this, other) ||\n      other is {} &&\n          runtimeType == other.runtimeType",
        class.name, class.name,
    )
    .ok()?;
    for field in &class.fields {
        out.push_str(" &&\n          ");
        render_equality_comparison(&mut out, &field.name, &field.ty);
    }
    out.push_str(";\n}");
    Some(out)
}

pub(crate) fn emit_hash_code(class: &ClassIr) -> Option<String> {
    if !has_trait(class, EQ_SYMBOL) {
        return None;
    }

    let mut out = String::with_capacity(class.name.len() + class.fields.len() * 32 + 96);
    out.push_str("@override\nint get hashCode {\n");
    if !class.fields.is_empty() {
        writeln!(&mut out, "  final self = this as {};", class.name).ok()?;
    }
    out.push_str("  return Object.hashAll([\n    runtimeType,\n");
    for field in &class.fields {
        out.push_str("    ");
        render_hash_value(&mut out, &field.name, &field.ty);
        out.push_str(",\n");
    }
    out.push_str("  ]);\n}");
    Some(out)
}

fn render_equality_comparison(out: &mut String, field_name: &str, ty: &TypeIr) {
    match deep_helper_name(ty) {
        Some(helper) => {
            write!(
                out,
                "{helper}.equals(other.{field_name}, self.{field_name})"
            )
            .ok();
        }
        None => {
            write!(out, "other.{field_name} == self.{field_name}").ok();
        }
    }
}

fn render_hash_value(out: &mut String, field_name: &str, ty: &TypeIr) {
    match deep_helper_name(ty) {
        Some(helper) => {
            write!(out, "{helper}.hash(self.{field_name})").ok();
        }
        None => {
            write!(out, "self.{field_name}").ok();
        }
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
    matches!(ty, TypeIr::Named { name, .. } if matches!(name.as_ref(), "List" | "Map" | "Iterable"))
}

fn needs_unordered_deep_helper(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Named { name, .. } if name.as_ref() == "Set")
}
