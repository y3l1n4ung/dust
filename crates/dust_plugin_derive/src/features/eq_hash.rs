use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, LibraryIr, SymbolId, TypeIr};

pub(crate) fn has_trait(class: &ClassIr, symbol: &SymbolId) -> bool {
    class
        .traits
        .iter()
        .any(|trait_app| trait_app.symbol == *symbol)
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
            "const DeepCollectionEquality _dustDeepCollectionEquality = DeepCollectionEquality();"
                .to_owned(),
        );
    }
    if needs_unordered {
        helpers.push(
            "const DeepCollectionEquality _dustUnorderedDeepCollectionEquality = DeepCollectionEquality.unordered();"
                .to_owned(),
        );
    }

    helpers
}

pub(crate) fn emit_eq(class: &ClassIr) -> Option<String> {
    let eq = SymbolId::new("derive_annotation::Eq");
    if !has_trait(class, &eq) {
        return None;
    }

    let comparisons = class
        .fields
        .iter()
        .map(|field| render_equality_comparison(&field.name, &field.ty))
        .collect::<Vec<_>>();

    let tail = if comparisons.is_empty() {
        String::new()
    } else {
        format!(" &&\n        {}", comparisons.join(" &&\n        "))
    };

    Some(format!(
        "@override\nbool operator ==(Object other) =>\n    identical(this, other) ||\n    other is {} &&\n        runtimeType == other.runtimeType{};",
        class.name, tail
    ))
}

pub(crate) fn emit_hash_code(class: &ClassIr) -> Option<String> {
    let eq = SymbolId::new("derive_annotation::Eq");
    if !has_trait(class, &eq) {
        return None;
    }

    let values = class
        .fields
        .iter()
        .map(|field| render_hash_value(&field.name, &field.ty))
        .collect::<Vec<_>>();

    let mut lines = vec!["runtimeType,".to_owned()];
    lines.extend(values.into_iter().map(|value| format!("{value},")));
    let list = lines
        .into_iter()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        "@override\nint get hashCode => Object.hashAll([\n{list}\n]);"
    ))
}

fn render_equality_comparison(field_name: &str, ty: &TypeIr) -> String {
    let left = format!("other.{field_name}");
    let right = format!("_dustSelf.{field_name}");

    match deep_helper_name(ty) {
        Some(helper) => format!("{helper}.equals({left}, {right})"),
        None => format!("{left} == {right}"),
    }
}

fn render_hash_value(field_name: &str, ty: &TypeIr) -> String {
    let value = format!("_dustSelf.{field_name}");

    match deep_helper_name(ty) {
        Some(helper) => format!("{helper}.hash({value})"),
        None => value,
    }
}

fn deep_helper_name(ty: &TypeIr) -> Option<&'static str> {
    if needs_unordered_deep_helper(ty) {
        Some("_dustUnorderedDeepCollectionEquality")
    } else if needs_ordered_deep_helper(ty) {
        Some("_dustDeepCollectionEquality")
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
