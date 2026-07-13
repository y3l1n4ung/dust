use dust_dart_emit::{DART_DOUBLE, DART_INT, DART_NUM, DART_STRING};
use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, FieldIr, TypeIr};

use super::{
    model::{ValidateConfig, field_validations, has_validate_trait},
    shape::validate_config_shape,
    type_source::{input_kind, supports_length},
};
use crate::features::{
    VALIDATE_SYMBOL,
    names::{library_declaration_names, upper_first},
};

/// Validates `Validate` derive usage for one class.
pub(crate) fn validate_validate(
    library: &DartFileIr,
    class: &ClassIr,
    emit_form_helpers: bool,
) -> Vec<Diagnostic> {
    if !has_validate_trait(class) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    if emit_form_helpers {
        validate_public_validator_names(library, class, &mut diagnostics);
    }
    for validation in field_validations(class) {
        for config in &validation.annotations {
            validate_field_config(library, class, validation.field, config, &mut diagnostics);
        }
    }
    for field in &class.fields {
        for config in field
            .configs
            .iter()
            .filter(|config| config.symbol.0 == VALIDATE_SYMBOL)
        {
            validate_config_shape(config, &mut diagnostics);
        }
    }
    diagnostics
}

/// Ensures generated public validator helper names do not collide.
fn validate_public_validator_names(
    library: &DartFileIr,
    class: &ClassIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let declaration_names = library_declaration_names(library);
    for validation in field_validations(class) {
        if input_kind(&validation.field.ty).is_none() {
            continue;
        }

        let validator_name = format!(
            "validate{}{}Input",
            class.name,
            upper_first(&validation.field.name)
        );
        if declaration_names.contains(&validator_name) {
            diagnostics.push(Diagnostic::error(format!(
                "generated validator `{validator_name}` for `{}.{}` conflicts with an existing top-level declaration",
                class.name, validation.field.name
            )));
        }
    }
}

/// Validates one field's parsed validation config against its type.
fn validate_field_config(
    library: &DartFileIr,
    class: &ClassIr,
    field: &FieldIr,
    config: &ValidateConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if (config.email
        || config.url
        || config.contains.is_some()
        || config.does_not_contain.is_some()
        || config.regex.is_some()
        || config.must_match.is_some())
        && !field.ty.is_named(DART_STRING)
    {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate` string validators on `{}` require `String` or `String?`",
            field.name
        )));
    }
    if config.length.is_some() && !supports_length(&field.ty) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(length: ...)` on `{}` requires String, List, Set, or Map",
            field.name
        )));
    }
    if config.range.is_some() && !is_numeric(&field.ty) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(range: ...)` on `{}` requires int, double, or num",
            field.name
        )));
    }
    if let Some(other) = &config.must_match {
        validate_must_match(class, field, other, diagnostics);
    }
    if config.nested {
        validate_nested(library, field, diagnostics);
    }
}

/// Validates that a must-match target exists with matching type.
fn validate_must_match(
    class: &ClassIr,
    field: &FieldIr,
    other: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match class
        .fields
        .iter()
        .find(|candidate| candidate.name == other)
    {
        Some(candidate) if candidate.ty != field.ty => {
            diagnostics.push(Diagnostic::error(format!(
                "`@Validate(mustMatch: '{other}')` type must match `{}`",
                field.name
            )))
        }
        Some(_) => {}
        None => diagnostics.push(Diagnostic::error(format!(
            "`@Validate(mustMatch: '{other}')` references a missing field"
        ))),
    }
}

/// Validates that nested validation targets another `Validate` class.
fn validate_nested(library: &DartFileIr, field: &FieldIr, diagnostics: &mut Vec<Diagnostic>) {
    let Some(name) = field.ty.name() else {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(nested: true)` on `{}` requires a named type",
            field.name
        )));
        return;
    };
    let Some(target) = library.classes.iter().find(|class| class.name == name) else {
        return;
    };
    if !has_validate_trait(target) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(nested: true)` target `{name}` must derive `Validate()`"
        )));
    }
}

/// Returns true when a type supports range validation.
fn is_numeric(ty: &TypeIr) -> bool {
    ty.is_named(DART_INT) || ty.is_named(DART_DOUBLE) || ty.is_named(DART_NUM)
}
