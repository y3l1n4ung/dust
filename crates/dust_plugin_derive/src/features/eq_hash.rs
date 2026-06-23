use std::{collections::HashMap, fmt::Write};

use dust_dart_emit::{DART_ITERABLE, DART_LIST, DART_MAP, DART_SET};
use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, TypeIr};

use crate::features::{
    EQ_SYMBOL,
    names::{NameAllocator, library_declaration_names, lower_first, upper_first},
};

/// Planned equality helper names and declarations for a library.
#[derive(Default)]
pub(crate) struct EqualityPlan {
    /// Deep equality helper names keyed by class and field.
    helpers: HashMap<(String, String), String>,
    /// Generated top-level helper declarations.
    declarations: Vec<String>,
}

impl EqualityPlan {
    /// Returns the deep equality helper for a field, if one is needed.
    fn helper_name(&self, class_name: &str, field_name: &str) -> Option<&str> {
        self.helpers
            .get(&(class_name.to_owned(), field_name.to_owned()))
            .map(String::as_str)
    }

    /// Returns generated deep equality helper declarations.
    pub(crate) fn declarations(&self) -> &[String] {
        &self.declarations
    }
}

/// Returns true when a class has a resolved derive trait symbol.
pub(crate) fn has_trait(class: &ClassIr, symbol: &str) -> bool {
    class
        .traits
        .iter()
        .any(|trait_app| trait_app.symbol.0 == symbol)
}

/// Validates equality generation requirements for a class.
pub(crate) fn validate_eq_hash(class: &ClassIr) -> Vec<Diagnostic> {
    let _ = class;
    Vec::new()
}

/// Plans collection equality helper declarations for a library.
pub(crate) fn plan_equality(library: &DartFileIr) -> EqualityPlan {
    let mut allocator = NameAllocator::new(library_declaration_names(library));
    let mut plan = EqualityPlan::default();

    for class in library
        .classes
        .iter()
        .filter(|class| has_trait(class, EQ_SYMBOL))
    {
        for field in &class.fields {
            let Some(constructor) = deep_equality_constructor(&field.ty) else {
                continue;
            };
            let base = format!(
                "_{}{}Equality",
                lower_first(&class.name),
                upper_first(&field.name)
            );
            let helper_name = allocator.allocate(base);
            plan.helpers.insert(
                (class.name.clone(), field.name.clone()),
                helper_name.clone(),
            );
            plan.declarations.push(format!(
                "const DeepCollectionEquality {helper_name} = {constructor};"
            ));
        }
    }

    plan
}

/// Emits a generated equality operator for an `Eq` class.
pub(crate) fn emit_eq(class: &ClassIr, equality: &EqualityPlan) -> Option<String> {
    if !has_trait(class, EQ_SYMBOL) {
        return None;
    }

    if class.fields.is_empty() {
        return Some(format!(
            "@override\nbool operator ==(Object other) =>\n    identical(this, other) ||\n    other is {} &&\n        runtimeType == other.runtimeType;",
            class.name
        ));
    }

    let self_name = self_name(class);
    Some(format!(
        "@override\nbool operator ==(Object other) {{\n  final {self_name} = this as {};\n  return identical(this, other) ||\n      other is {} &&\n          runtimeType == other.runtimeType{};\n}}",
        class.name,
        class.name,
        render_equality_comparisons(class, equality, &self_name)
    ))
}

/// Emits a generated `hashCode` getter for an `Eq` class.
pub(crate) fn emit_hash_code(class: &ClassIr, equality: &EqualityPlan) -> Option<String> {
    if !has_trait(class, EQ_SYMBOL) {
        return None;
    }

    let self_name = self_name(class);
    let self_binding = if class.fields.is_empty() {
        String::new()
    } else {
        format!("  final {self_name} = this as {};\n", class.name)
    };
    Some(format!(
        "@override\nint get hashCode {{\n{}  return Object.hashAll([\n    runtimeType,\n{}  ]);\n}}",
        self_binding,
        render_hash_values(class, equality, &self_name)
    ))
}

/// Renders all field comparisons in the equality operator.
fn render_equality_comparisons(
    class: &ClassIr,
    equality: &EqualityPlan,
    self_name: &str,
) -> String {
    let mut out = String::with_capacity(class.fields.len() * 40);
    for field in &class.fields {
        write!(
            out,
            " &&\n          {}",
            render_equality_comparison(class, &field.name, equality, self_name)
        )
        .expect("writing to String cannot fail");
    }
    out
}

/// Renders all field values included in `hashCode`.
fn render_hash_values(class: &ClassIr, equality: &EqualityPlan, self_name: &str) -> String {
    let mut out = String::with_capacity(class.fields.len() * 24);
    for field in &class.fields {
        writeln!(
            out,
            "    {},",
            render_hash_value(class, &field.name, equality, self_name)
        )
        .expect("writing to String cannot fail");
    }
    out
}

/// Renders one equality comparison, using a deep helper when needed.
fn render_equality_comparison(
    class: &ClassIr,
    field_name: &str,
    equality: &EqualityPlan,
    self_name: &str,
) -> String {
    match equality.helper_name(&class.name, field_name) {
        Some(helper) => format!("{helper}.equals(other.{field_name}, {self_name}.{field_name})"),
        None => format!("other.{field_name} == {self_name}.{field_name}"),
    }
}

/// Renders one hash value, using a deep helper when needed.
fn render_hash_value(
    class: &ClassIr,
    field_name: &str,
    equality: &EqualityPlan,
    self_name: &str,
) -> String {
    match equality.helper_name(&class.name, field_name) {
        Some(helper) => format!("{helper}.hash({self_name}.{field_name})"),
        None => format!("{self_name}.{field_name}"),
    }
}

/// Returns the DeepCollectionEquality constructor needed by a type.
fn deep_equality_constructor(ty: &TypeIr) -> Option<&'static str> {
    if needs_unordered_deep_helper(ty) {
        Some("DeepCollectionEquality.unordered()")
    } else if needs_ordered_deep_helper(ty) {
        Some("DeepCollectionEquality()")
    } else {
        None
    }
}

/// Allocates a collision-safe local receiver name for generated equality.
fn self_name(class: &ClassIr) -> String {
    let mut allocator = NameAllocator::new(class.fields.iter().map(|field| field.name.as_str()));
    allocator.allocate("self")
}

/// Returns true when ordered deep equality is needed.
fn needs_ordered_deep_helper(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Named { name, .. } if matches!(name.as_ref(), DART_LIST | DART_MAP | DART_ITERABLE))
}

/// Returns true when unordered deep equality is needed.
fn needs_unordered_deep_helper(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Named { name, .. } if name.as_ref() == DART_SET)
}
