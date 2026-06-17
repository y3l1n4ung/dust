mod render;
mod support;

use std::borrow::Cow;

use dust_diagnostics::Diagnostic;
use dust_ir::ClassIr;

use crate::features::{
    COPY_WITH_SYMBOL,
    eq_hash::has_trait,
    writer::{build_constructor_call_multiline, find_clone_constructor, render_return_statement},
};

pub(crate) use self::render::CopyableTypes;

use self::{
    render::render_copied_value,
    support::{
        render_copy_with_params, render_copy_with_source_expr, render_setup_blocks,
        should_keep_source_local, temp_name,
    },
};

pub(crate) fn emit_copy_with(
    class: &ClassIr,
    copyable_types: &CopyableTypes<'_>,
) -> Option<String> {
    if !has_trait(class, COPY_WITH_SYMBOL) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let params = render_copy_with_params(class);
    let mut setup = Vec::new();
    let mut values: Vec<(&str, Cow<'_, str>)> = Vec::with_capacity(class.fields.len());

    for (depth, field) in class.fields.iter().enumerate() {
        let source_expr = render_copy_with_source_expr(field.name.as_str(), &field.ty);
        let copied_expr = render_copied_value(&field.ty, &source_expr, depth, copyable_types);

        if copied_expr.as_ref() == source_expr {
            if should_keep_source_local(&field.ty) {
                let next_var = temp_name("next", &field.name, "");
                setup.push(format!("final {next_var} = {source_expr};"));
                values.push((field.name.as_str(), Cow::Owned(next_var)));
            } else {
                values.push((field.name.as_str(), Cow::Owned(source_expr)));
            }
        } else if should_keep_source_local(&field.ty) {
            let source_var = temp_name("next", &field.name, "Source");
            let next_var = temp_name("next", &field.name, "");
            let copied_expr =
                render_copied_value(&field.ty, source_var.as_str(), depth, copyable_types);
            setup.push(format!("final {source_var} = {source_expr};"));
            setup.push(format!("final {next_var} = {};", copied_expr.as_ref()));
            values.push((field.name.as_str(), Cow::Owned(next_var)));
        } else {
            let next_var = temp_name("next", &field.name, "");
            setup.push(format!("final {next_var} = {};", copied_expr.as_ref()));
            values.push((field.name.as_str(), Cow::Owned(next_var)));
        }
    }

    let call = build_constructor_call_multiline(class, constructor, &values)?;
    let setup = render_setup_blocks(setup);
    let return_call = render_return_statement(&call, "  ");

    let self_binding = if class.fields.is_empty() {
        String::new()
    } else {
        format!("  final self = this as {};\n", class.name)
    };
    let body = if setup.is_empty() {
        return_call
    } else {
        format!("{setup}\n\n{return_call}")
    };

    Some(format!(
        "{} copyWith({}) {{\n{}{}\n}}",
        class.name, params, self_binding, body
    ))
}

pub(crate) fn validate_copy_with(class: &ClassIr) -> Vec<Diagnostic> {
    if !has_trait(class, COPY_WITH_SYMBOL) {
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
