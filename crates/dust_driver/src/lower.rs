mod inheritance;
mod serde;
mod serde_parse;
mod tests_inheritance;
mod tests_serde;
mod tests_type;
mod type_parse;

use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{
    ClassIr, ConstructorIr, ConstructorParamIr, EnumIr, EnumVariantIr, FieldIr, LibraryIr,
    LoweringOutcome, ParamKind, SpanIr,
};
use dust_parser_dart::ParameterKind;
use dust_resolver::{ResolvedClass, ResolvedLibrary};

use self::{
    inheritance::{infer_param_type, merged_fields_for_class, resolve_constructor_param_types},
    serde::{lower_class_serde_config, lower_field_serde_config},
    type_parse::lower_type,
};

/// Lowers one resolved library into semantic IR.
pub(crate) fn lower_library(library: &ResolvedLibrary) -> LoweringOutcome<LibraryIr> {
    let mut diagnostics = Vec::new();
    let mut classes = library
        .classes
        .iter()
        .filter(|class| !class.traits.is_empty() || !class.configs.is_empty())
        .map(|class| {
            let outcome = lower_class(class);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect::<Vec<_>>();
    let enums = library
        .enums
        .iter()
        .map(|e| {
            let outcome = lower_enum(e);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect();

    let index_by_name = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.name.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut merged_cache = HashMap::new();
    let mut active_stack = Vec::new();
    for index in 0..classes.len() {
        let merged_fields = merged_fields_for_class(
            index,
            &classes,
            &index_by_name,
            &mut merged_cache,
            &mut active_stack,
            &mut diagnostics,
        );
        classes[index].fields = merged_fields;
        resolve_constructor_param_types(&mut classes[index], &mut diagnostics);
    }

    LoweringOutcome {
        value: LibraryIr {
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            span: library.span,
            classes,
            enums,
        },
        diagnostics,
    }
}

fn lower_enum(e: &dust_resolver::ResolvedEnum) -> LoweringOutcome<EnumIr> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let serde = lower_class_serde_config(&e.name, &e.configs, &mut diagnostics);
    let variants: Vec<EnumVariantIr> = e
        .variants
        .iter()
        .map(|v| EnumVariantIr {
            name: v.name.clone(),
            span: v.span,
        })
        .collect();
    LoweringOutcome {
        value: EnumIr {
            name: e.name.clone(),
            span: e.span,
            variants,
            traits: e.traits.clone(),
            serde,
        },
        diagnostics,
    }
}

fn lower_class(class: &ResolvedClass) -> LoweringOutcome<ClassIr> {
    let mut diagnostics = Vec::new();
    let serde = lower_class_serde_config(&class.name, &class.configs, &mut diagnostics);

    let fields = class
        .fields
        .iter()
        .map(|field| {
            let outcome = lower_type(field.type_source.as_deref());
            diagnostics.extend(outcome.diagnostics);
            FieldIr {
                name: field.name.clone(),
                ty: outcome.value,
                span: field.span,
                has_default: field.has_default,
                serde: lower_field_serde_config(&field.name, &field.configs, &mut diagnostics),
            }
        })
        .collect::<Vec<_>>();

    let constructors = class
        .constructors
        .iter()
        .map(|constructor| {
            let params = constructor
                .params
                .iter()
                .map(|param| {
                    let outcome = param
                        .type_source
                        .as_deref()
                        .map(|source| lower_type(Some(source)))
                        .unwrap_or_else(|| infer_param_type(param.name.as_str(), &fields));
                    diagnostics.extend(outcome.diagnostics);
                    ConstructorParamIr {
                        name: param.name.clone(),
                        ty: outcome.value,
                        span: SpanIr::new(class.span.file_id, param.span),
                        kind: match param.kind {
                            ParameterKind::Positional => ParamKind::Positional,
                            ParameterKind::Named => ParamKind::Named,
                        },
                        has_default: param.has_default,
                    }
                })
                .collect();

            ConstructorIr {
                name: constructor.name.clone(),
                span: SpanIr::new(class.span.file_id, constructor.span),
                params,
            }
        })
        .collect();

    LoweringOutcome {
        value: ClassIr {
            kind: class.kind,
            name: class.name.clone(),
            is_abstract: class.is_abstract,
            superclass_name: class.superclass_name.clone(),
            span: class.span,
            fields,
            constructors,
            traits: class.traits.clone(),
            serde,
        },
        diagnostics,
    }
}
