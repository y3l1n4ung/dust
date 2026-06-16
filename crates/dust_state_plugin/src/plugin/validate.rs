use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr};

use super::parse::{parse_view_model_config, view_model_config};

pub(crate) fn validate_library_state(library: &DartFileIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let local_classes = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();
    let local_enums = library
        .enums
        .iter()
        .map(|enum_ir| enum_ir.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        let Some(config) = view_model_config(&class.configs) else {
            continue;
        };
        let Some(annotation) = parse_view_model_config(config) else {
            diagnostics.push(Diagnostic::error(format!(
                "`@ViewModel` on `{}` requires `state: SomeState`",
                class.name
            )));
            continue;
        };

        let expected_base = format!("${}", class.name);
        if class.superclass_name.as_deref() != Some(expected_base.as_str()) {
            diagnostics.push(Diagnostic::error(format!(
                "view model `{}` must extend `{expected_base}`",
                class.name
            )));
        }

        if !local_classes.contains(annotation.state_type.as_str())
            && !local_enums.contains(annotation.state_type.as_str())
            && !has_imports(library)
        {
            diagnostics.push(Diagnostic::error(format!(
                "view model `{}` state `{}` must be declared in the same library or imported",
                class.name, annotation.state_type
            )));
        }

        if annotation.initial_source.is_none()
            && local_enums.contains(annotation.state_type.as_str())
        {
            diagnostics.push(Diagnostic::error(format!(
                "view model `{}` enum state `{}` requires `initial: {}.someValue`",
                class.name, annotation.state_type, annotation.state_type
            )));
        }

        if let Some(state_class) = library
            .classes
            .iter()
            .find(|candidate| candidate.name == annotation.state_type)
            && annotation.initial_source.is_none()
        {
            validate_default_initial_state(class, state_class, &mut diagnostics);
        }

        if let Some(args_type) = annotation.args_type.as_deref() {
            validate_args_type(library, class, args_type, &mut diagnostics);
        }
    }

    diagnostics
}

fn validate_default_initial_state(
    view_model: &ClassIr,
    state_class: &ClassIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if state_class.constructors.is_empty() {
        if state_class.fields.iter().all(|field| field.has_default) {
            return;
        }
        diagnostics.push(Diagnostic::error(format!(
            "view model `{}` state `{}` needs `initial:` because Dust cannot prove `const {}()` is valid",
            view_model.name, state_class.name, state_class.name
        )));
        return;
    }

    let Some(constructor) = state_class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
    else {
        diagnostics.push(Diagnostic::error(format!(
            "view model `{}` state `{}` needs an unnamed generative constructor or explicit `initial:`",
            view_model.name, state_class.name
        )));
        return;
    };

    if constructor
        .params
        .iter()
        .any(|param| !param.has_default && !param.ty.is_nullable())
    {
        diagnostics.push(Diagnostic::error(format!(
            "view model `{}` state `{}` default constructor has required params; add `initial:`",
            view_model.name, state_class.name
        )));
    }
}

fn validate_args_type(
    library: &DartFileIr,
    class: &ClassIr,
    args_type: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(args_class) = library
        .classes
        .iter()
        .find(|candidate| candidate.name == args_type)
    else {
        if !has_imports(library) {
            diagnostics.push(Diagnostic::error(format!(
                "view model `{}` args `{args_type}` must be declared in the same library or imported",
                class.name
            )));
        }
        return;
    };
    if args_class.superclass_name.as_deref() != Some("ViewModelArgs") {
        diagnostics.push(Diagnostic::error(format!(
            "view model `{}` args `{args_type}` must extend `ViewModelArgs`",
            class.name
        )));
    }
}

fn has_imports(library: &DartFileIr) -> bool {
    !library.imports.is_empty()
}
