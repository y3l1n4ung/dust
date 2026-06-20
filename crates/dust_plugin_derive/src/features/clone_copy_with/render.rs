use std::fmt::Write;

use dust_ir::{ClassIr, FieldIr};

use crate::features::names::lower_first;

use super::{
    CopyWithNames, CopyWithPlan, ValueSampleKind, nested_target_type, sample_replacement_field,
    sample_value,
    support::{
        copy_with_impl_param_type, copy_with_interface_param_type, needs_copy_with_sentinel,
    },
};

/// Renders the per-class sentinel helper type and value.
pub(super) fn render_sentinel_helper(class_name: &str, value_name: &str) -> String {
    format!(
        "final class {class_name} {{\n  const {class_name}();\n}}\n\nconst {value_name} = {class_name}();"
    )
}

/// Renders the generated public `copyWith` getter for a class.
pub(super) fn render_copy_with_getter(
    class: &ClassIr,
    names: &CopyWithNames,
    plan: &CopyWithPlan,
) -> String {
    let docs = render_getter_docs(class, plan);
    format!(
        "{docs}\n@pragma('vm:prefer-inline')\n{}<{}> get copyWith => {}<{}>(this as {}, ({}) => {});",
        names.interface_name,
        class.name,
        names.impl_name,
        class.name,
        class.name,
        names.callback_value_name,
        names.callback_value_name
    )
}

/// Renders the copyWith callable interface and implementation types.
pub(super) fn render_copy_with_support(
    class: &ClassIr,
    names: &CopyWithNames,
    constructor_call: &str,
    plan: &CopyWithPlan,
    include_credit: bool,
) -> String {
    let interface_params = render_interface_params(class);
    let interface_getters = render_nested_interface_getters(class, plan);
    let impl_params = render_impl_params(class, names);
    let return_call = render_then_return(constructor_call, &names.then_name);
    let impl_getters = render_nested_impl_getters(class, names, plan);
    let credit = if include_credit {
        "// CopyWith API inspired by Freezed.\n\n"
    } else {
        ""
    };

    format!(
        "{credit}/// @nodoc\nabstract class {}<{}> {{\n  {} call({});{}\n}}\n\n/// @nodoc\nfinal class {}<{}> implements {}<{}> {{\n  const {}(this.{}, this.{});\n\n  final {} {};\n  final {} Function({}) {};\n\n  @override\n  @pragma('vm:prefer-inline')\n  {} call({}) {{\n{}\n  }}{}\n}}",
        names.interface_name,
        "$Res",
        "$Res",
        interface_params,
        interface_getters,
        names.impl_name,
        "$Res",
        names.interface_name,
        "$Res",
        names.impl_name,
        names.self_name,
        names.then_name,
        class.name,
        names.self_name,
        "$Res",
        class.name,
        names.then_name,
        "$Res",
        impl_params,
        return_call,
        impl_getters
    )
}

/// Renders the public call parameters for the copyWith interface.
fn render_interface_params(class: &ClassIr) -> String {
    render_call_params(class, |field| {
        format!(
            "{} {}",
            copy_with_interface_param_type(&field.ty),
            field.name
        )
    })
}

/// Renders implementation call parameters with sentinel defaults.
fn render_impl_params(class: &ClassIr, names: &CopyWithNames) -> String {
    render_call_params(class, |field| {
        let default = if needs_copy_with_sentinel(&field.ty) {
            names
                .sentinel
                .as_ref()
                .map(|sentinel| sentinel.value_name.as_str())
                .unwrap_or("null")
        } else {
            "null"
        };
        format!(
            "{} {} = {}",
            copy_with_impl_param_type(),
            field.name,
            default
        )
    })
}

/// Renders named call parameters for an interface or implementation.
fn render_call_params(class: &ClassIr, render_param: impl Fn(&FieldIr) -> String) -> String {
    if class.fields.is_empty() {
        return String::new();
    }

    let mut params = String::from("{\n");
    for field in &class.fields {
        writeln!(params, "    {},", render_param(field)).expect("writing to String cannot fail");
    }
    params.push_str("  }");
    params
}

/// Wraps the constructor call with the generated return callback.
fn render_then_return(constructor_call: &str, then_name: &str) -> String {
    let mut out = format!("    return {then_name}(\n");
    for line in constructor_call.lines() {
        out.push_str("      ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("    );");
    out
}

/// Renders nested copyWith getters on the public interface.
fn render_nested_interface_getters(class: &ClassIr, plan: &CopyWithPlan) -> String {
    let mut out = String::new();
    for field in &class.fields {
        let Some(target) = nested_target(field, plan) else {
            continue;
        };

        write!(
            out,
            "\n\n  {}<$Res>{} get {};",
            target.names.interface_name,
            if target.nullable { "?" } else { "" },
            field.name
        )
        .expect("writing to String cannot fail");
    }
    out
}

/// Renders nested copyWith getter implementations.
fn render_nested_impl_getters(
    class: &ClassIr,
    names: &CopyWithNames,
    plan: &CopyWithPlan,
) -> String {
    let mut out = String::new();
    for field in &class.fields {
        let Some(target) = nested_target(field, plan) else {
            continue;
        };

        if target.nullable {
            let value_name = names
                .nested_value_names
                .get(&field.name)
                .map(String::as_str)
                .unwrap_or(field.name.as_str());
            write!(
                out,
                "\n\n  @override\n  @pragma('vm:prefer-inline')\n  {}<$Res>? get {} {{\n    final {} = {}.{};\n    if ({} == null) {{\n      return null;\n    }}\n\n    return {}<$Res>(\n      {},\n      ({}) => call({}: {}),\n    );\n  }}",
                target.names.interface_name,
                field.name,
                value_name,
                names.self_name,
                field.name,
                value_name,
                target.names.impl_name,
                value_name,
                names.callback_value_name,
                field.name,
                names.callback_value_name,
            )
            .expect("writing to String cannot fail");
        } else {
            write!(
                out,
                "\n\n  @override\n  @pragma('vm:prefer-inline')\n  {}<$Res> get {} {{\n    return {}<$Res>(\n      {}.{},\n      ({}) => call({}: {}),\n    );\n  }}",
                target.names.interface_name,
                field.name,
                target.names.impl_name,
                names.self_name,
                field.name,
                names.callback_value_name,
                field.name,
                names.callback_value_name,
            )
            .expect("writing to String cannot fail");
        }
    }
    out
}

/// Renders Dartdoc for the generated copyWith getter.
fn render_getter_docs(class: &ClassIr, plan: &CopyWithPlan) -> String {
    let receiver = lower_first(&class.name);
    let mut docs = format!(
        "/// Creates a copy of this `{}` with selected fields replaced.\n///\n/// Usage:\n/// ```dart",
        class.name
    );

    if let Some(field) = sample_replacement_field(class) {
        writeln!(
            docs,
            "\n/// final updated = {receiver}.copyWith({}: {});",
            field.name,
            sample_value(&field.ty, ValueSampleKind::Replacement)
        )
        .expect("writing to String cannot fail");
    } else {
        writeln!(docs, "\n/// final updated = {receiver}.copyWith();")
            .expect("writing to String cannot fail");
    }

    if let Some(field) = class
        .fields
        .iter()
        .find(|field| needs_copy_with_sentinel(&field.ty))
    {
        writeln!(
            docs,
            "/// final cleared = {receiver}.copyWith({}: null);",
            field.name
        )
        .expect("writing to String cannot fail");
    }

    if let Some((field, target)) = class
        .fields
        .iter()
        .filter_map(|field| {
            nested_target_type(&field.ty).map(|(target_name, _)| (field, target_name))
        })
        .find_map(|(field, target_name)| {
            let sample = plan.samples_by_class.get(target_name)?;
            Some((field, (sample.field_name.as_str(), sample.nested_value)))
        })
    {
        writeln!(
            docs,
            "/// final nested = {receiver}.copyWith.{}({}: {});",
            field.name, target.0, target.1
        )
        .expect("writing to String cannot fail");
    }

    docs.push_str("/// ```");
    docs
}

/// Resolves the nested copyWith target for a field.
fn nested_target<'a>(field: &FieldIr, plan: &'a CopyWithPlan) -> Option<NestedTarget<'a>> {
    let (name, nullable) = nested_target_type(&field.ty)?;
    let names = plan.names_for(name)?;
    Some(NestedTarget { names, nullable })
}

/// Nested copyWith target metadata for one field.
struct NestedTarget<'a> {
    /// Generated names for the nested target class.
    names: &'a CopyWithNames,
    /// Whether the source field is nullable.
    nullable: bool,
}
