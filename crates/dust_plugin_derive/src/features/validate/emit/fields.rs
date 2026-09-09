//! Rendering the per-field context a validator template reads.

use super::*;

/// Builds template contexts for all validated fields on a class.
pub(super) fn render_fields(class: &ClassIr, emit_form_helpers: bool) -> Vec<FieldContext> {
    field_validations(class)
        .into_iter()
        .map(|validation| {
            let input_kind = input_kind(&validation.field.ty);
            let can_validate_input = emit_form_helpers && input_kind.is_some();
            let uses_self = validation
                .annotations
                .iter()
                .any(|config| config.must_match.is_some());
            let field_name = &validation.field.name;
            let field_type = render_type(&validation.field.ty);
            let mut allocator = NameAllocator::new(std::iter::empty::<&str>());
            let self_name = if uses_self {
                allocator.allocate("self")
            } else {
                "self".to_owned()
            };
            let value_name = allocator.allocate(field_name);
            let errors_name = allocator.allocate("errors");
            let nested_validation_name = allocator.allocate(format!("{value_name}Validation"));
            let nested_errors_name = allocator.allocate("nestedErrors");
            let nested_error_name = allocator.allocate("error");
            let custom_error_name = allocator.allocate(format!("{value_name}CustomError"));
            let mut input_allocator = NameAllocator::new(["value"]);
            let parsed_name = input_allocator.allocate(field_name);
            let helper_name = format!("_validate{}", upper_first(field_name));
            let input_helper_name = format!("validate{}Input", upper_first(field_name));
            let public_input_helper_name =
                format!("validate{}{}Input", class.name, upper_first(field_name));
            FieldContext {
                name: validation.field.name.clone(),
                literal: dart_string_literal(&validation.field.name),
                value_name: value_name.clone(),
                self_name: self_name.clone(),
                errors_name: errors_name.clone(),
                parsed_name: parsed_name.clone(),
                nested_validation_name: nested_validation_name.clone(),
                nested_errors_name: nested_errors_name.clone(),
                nested_error_name: nested_error_name.clone(),
                custom_error_name: custom_error_name.clone(),
                helper_signature: helper_signature(
                    &helper_name,
                    &class.name,
                    &self_name,
                    &value_name,
                    &errors_name,
                    &field_type,
                    uses_self,
                ),
                input_signature: input_signature(
                    &input_helper_name,
                    &class.name,
                    &self_name,
                    uses_self,
                ),
                public_input_signature: public_input_signature(
                    &public_input_helper_name,
                    &class.name,
                    uses_self,
                ),
                helper_name,
                input_helper_name,
                public_input_helper_name,
                type_source: field_type,
                nullable: validation.field.ty.is_nullable(),
                can_validate_input,
                input_kind,
                parse_error_message: parse_error_message(&validation.annotations),
                uses_self,
                configs: validation
                    .annotations
                    .iter()
                    .map(|config| render_config(validation.field, config))
                    .collect(),
            }
        })
        .collect()
}
