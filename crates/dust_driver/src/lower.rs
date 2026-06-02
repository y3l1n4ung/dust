mod inheritance;
mod parse_support;
#[path = "lower/query_calls.rs"]
mod query_calls;
mod serde;
mod serde_parse;
mod tests_inheritance;
mod tests_serde;
mod tests_type;
mod type_parse;

use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;
use dust_ir::{
    ClassIr, ConstructorIr, ConstructorParamIr, EnumIr, EnumVariantIr, FieldIr, LibraryIr,
    LoweringOutcome, MethodIr, MethodParamIr, ParamKind, SpanIr,
};
use dust_parser_dart::{ParameterKind, ParsedDirective};
use dust_resolver::{ResolvedClass, ResolvedLibrary};

use self::{
    inheritance::{infer_param_type, merged_fields_for_class, resolve_constructor_param_types},
    query_calls::lower_query_calls,
    serde::{lower_class_serde_config, lower_field_serde_config},
    type_parse::lower_type,
};

/// Lowers one resolved library into semantic IR.
pub(crate) fn lower_library(library: &ResolvedLibrary) -> LoweringOutcome<LibraryIr> {
    let mut diagnostics = Vec::new();
    let required_classes = lowering_required_class_names(library);
    let mut classes = library
        .classes
        .iter()
        .filter(|class| required_classes.contains(class.name.as_str()))
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
            package_root: String::new(),
            package_name: String::new(),
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            imports: library_imports(library),
            span: library.span,
            classes,
            enums,
            query_calls: lower_query_calls(library, &mut diagnostics),
        },
        diagnostics,
    }
}

fn lowering_required_class_names(library: &ResolvedLibrary) -> HashSet<&str> {
    let mut names = library
        .classes
        .iter()
        .filter(|class| !class.traits.is_empty() || !class.configs.is_empty())
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        for field in &class.fields {
            for config in &field.configs {
                if let Some(converter) = config
                    .named_argument_source("tryFrom")
                    .and_then(try_from_converter_name)
                {
                    names.insert(converter);
                }
            }
        }
    }

    names
}

fn try_from_converter_name(source: &str) -> Option<&str> {
    let value = source.trim();
    let value = value.strip_prefix("const ").unwrap_or(value).trim();
    let before_args = value.split_once('(').map_or(value, |(name, _)| name).trim();
    before_args
        .rsplit('.')
        .next()
        .filter(|name| !name.is_empty())
}

fn library_imports(library: &ResolvedLibrary) -> Vec<String> {
    library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import { uri, .. } => Some(uri.clone()),
            _ => None,
        })
        .collect()
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
                configs: field.configs.clone(),
            }
        })
        .collect::<Vec<_>>();

    let methods = class
        .methods
        .iter()
        .map(|method| {
            let return_type_outcome = lower_type(method.surface.return_type_source.as_deref());
            diagnostics.extend(return_type_outcome.diagnostics);

            let params = method
                .params
                .iter()
                .map(|param| {
                    let type_outcome = lower_type(param.surface.type_source.as_deref());
                    diagnostics.extend(type_outcome.diagnostics);

                    MethodParamIr {
                        name: param.surface.name.clone(),
                        ty: type_outcome.value,
                        span: param.span,
                        kind: match param.surface.kind {
                            ParameterKind::Positional => ParamKind::Positional,
                            ParameterKind::Named => ParamKind::Named,
                        },
                        has_default: param.surface.has_default,
                        default_value_source: param.surface.default_value_source.clone(),
                        traits: param.traits.clone(),
                        configs: param.configs.clone(),
                    }
                })
                .collect();

            MethodIr {
                name: method.surface.name.clone(),
                is_static: method.surface.is_static,
                is_external: method.surface.is_external,
                return_type: return_type_outcome.value,
                has_body: method.surface.has_body,
                body_source: method.surface.body_source.clone(),
                params,
                span: method.span,
                traits: method.traits.clone(),
                configs: method.configs.clone(),
            }
        })
        .collect();

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
                        default_value_source: param.default_value_source.clone(),
                    }
                })
                .collect();

            ConstructorIr {
                name: constructor.name.clone(),
                is_factory: constructor.is_factory,
                redirected_target_source: constructor.redirected_target_source.clone(),
                redirected_target_name: constructor.redirected_target_name.clone(),
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
            is_interface: class.is_interface,
            superclass_name: class.superclass_name.clone(),
            span: class.span,
            fields,
            constructors,
            methods,
            traits: class.traits.clone(),
            configs: class.configs.clone(),
            serde,
        },
        diagnostics,
    }
}
