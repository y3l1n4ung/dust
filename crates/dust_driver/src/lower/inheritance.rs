use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, FieldIr, LoweringOutcome, TypeIr};

pub(crate) fn infer_param_type(name: &str, fields: &[FieldIr]) -> LoweringOutcome<TypeIr> {
    if let Some(field) = fields.iter().find(|field| field.name == name) {
        LoweringOutcome::new(field.ty.clone())
    } else {
        LoweringOutcome::new(TypeIr::unknown())
    }
}

pub(crate) fn merged_fields_for_class(
    index: usize,
    classes: &[ClassIr],
    index_by_name: &HashMap<String, usize>,
    cache: &mut HashMap<usize, Vec<FieldIr>>,
    active_stack: &mut Vec<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<FieldIr> {
    if let Some(cached) = cache.get(&index) {
        return cached.clone();
    }

    if active_stack.contains(&index) {
        diagnostics.push(Diagnostic::error(format!(
            "cyclic superclass chain detected while lowering `{}`",
            classes[index].name
        )));
        return classes[index].fields.clone();
    }

    active_stack.push(index);

    let mut fields = if let Some(superclass_name) = classes[index].superclass_name.as_ref() {
        if let Some(super_index) = index_by_name.get(superclass_name) {
            merged_fields_for_class(
                *super_index,
                classes,
                index_by_name,
                cache,
                active_stack,
                diagnostics,
            )
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    for field in &classes[index].fields {
        if let Some(existing) = fields
            .iter_mut()
            .find(|existing| existing.name == field.name)
        {
            *existing = field.clone();
        } else {
            fields.push(field.clone());
        }
    }

    active_stack.pop();
    cache.insert(index, fields.clone());
    fields
}

pub(crate) fn resolve_constructor_param_types(
    class: &mut ClassIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for constructor in &mut class.constructors {
        for param in &mut constructor.params {
            if matches!(param.ty, TypeIr::Unknown) {
                if let Some(field) = class.fields.iter().find(|field| field.name == param.name) {
                    param.ty = field.ty.clone();
                } else {
                    diagnostics.push(Diagnostic::warning(format!(
                        "could not infer constructor parameter type for `{}`",
                        param.name
                    )));
                }
            }
        }
    }
}
