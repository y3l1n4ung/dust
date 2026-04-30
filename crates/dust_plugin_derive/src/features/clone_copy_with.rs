mod render;
mod support;

use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, SymbolId};

use crate::features::{
    eq_hash::has_trait,
    writer::{build_constructor_call_multiline, find_clone_constructor, render_return_statement},
};

use self::{
    render::render_copied_value,
    support::{
        render_copy_with_params, render_copy_with_source_expr, render_setup_blocks,
        should_keep_source_local, temp_name, uses_undefined_sentinel,
    },
};

pub(crate) fn copy_with_requires_undefined(class: &ClassIr) -> bool {
    class
        .fields
        .iter()
        .any(|field| uses_undefined_sentinel(&field.ty))
}

pub(crate) fn emit_copy_with(class: &ClassIr, copyable_types: &HashSet<String>) -> Option<String> {
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &copy_with) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let params = render_copy_with_params(class);
    let mut setup = Vec::new();
    let mut values = Vec::with_capacity(class.fields.len());

    for (depth, field) in class.fields.iter().enumerate() {
        let source_expr = render_copy_with_source_expr(field.name.as_str(), &field.ty);
        let copied_expr = render_copied_value(&field.ty, &source_expr, depth, copyable_types);

        if copied_expr == source_expr {
            values.push((field.name.as_str(), source_expr));
        } else if should_keep_source_local(&field.ty) {
            let source_var = temp_name("next", &field.name, "Source");
            let next_var = temp_name("next", &field.name, "");
            let copied_expr =
                render_copied_value(&field.ty, source_var.as_str(), depth, copyable_types);
            setup.push(format!("final {source_var} = {source_expr};"));
            setup.push(format!("final {next_var} = {copied_expr};"));
            values.push((field.name.as_str(), next_var));
        } else {
            let next_var = temp_name("next", &field.name, "");
            setup.push(format!("final {next_var} = {copied_expr};"));
            values.push((field.name.as_str(), next_var));
        }
    }

    let call = build_constructor_call_multiline(class, constructor, &values)?;
    let setup = render_setup_blocks(setup);
    let return_call = render_return_statement(&call, "  ");

    let body = if setup.is_empty() {
        return_call
    } else {
        format!("{setup}\n\n{return_call}")
    };

    Some(format!(
        "{name} copyWith({params}) {{\n{body}\n}}",
        name = class.name,
        params = params,
        body = body
    ))
}

pub(crate) fn validate_copy_with(class: &ClassIr) -> Vec<Diagnostic> {
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &copy_with) {
        return Vec::new();
    }

    if class.is_abstract {
        return vec![Diagnostic::error(format!(
            "`CopyWith` cannot target abstract class `{}` because Dust cannot instantiate it",
            class.name
        ))];
    }

    if find_clone_constructor(class).is_some() {
        Vec::new()
    } else {
        vec![Diagnostic::error(format!(
            "`CopyWith` requires a constructor that accepts every field on class `{}`",
            class.name
        ))]
    }
}
