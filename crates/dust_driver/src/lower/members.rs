//! Lowering for what lives inside a class: fields, methods, constructors,
//! parameters and the enum and class bodies that hold them.

use super::*;

/// Lowers a parsed field before resolver trait/config data exists.
pub(super) fn lower_unresolved_field(
    file_id: dust_text::FileId,
    field: &ParsedFieldSurface,
    diagnostics: &mut Vec<Diagnostic>,
) -> FieldIr {
    let ty = lower_type(field.parsed_type.as_ref(), field.type_source.as_deref());
    diagnostics.extend(ty.diagnostics);

    FieldIr {
        name: field.name.clone(),
        ty: ty.value,
        span: SpanIr::new(file_id, field.span),
        has_default: field.has_default,
        serde: None,
        configs: Vec::new(),
    }
}

/// Lowers parsed method parameters before resolver trait/config data exists.
pub(super) fn lower_unresolved_method_params(
    file_id: dust_text::FileId,
    params: &[ParsedMethodParamSurface],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MethodParamIr> {
    params
        .iter()
        .map(|param| {
            let ty = lower_type(param.parsed_type.as_ref(), param.type_source.as_deref());
            diagnostics.extend(ty.diagnostics);

            MethodParamIr {
                name: param.name.clone(),
                ty: ty.value,
                span: SpanIr::new(file_id, param.span),
                kind: lower_parameter_kind(param.kind),
                is_required: param.is_required,
                has_default: param.has_default,
                default_value_source: param.default_value_source.clone(),
                traits: Vec::new(),
                configs: Vec::new(),
            }
        })
        .collect()
}

/// Maps parser parameter kind to IR parameter kind.
pub(super) fn lower_parameter_kind(kind: ParameterKind) -> ParamKind {
    match kind {
        ParameterKind::Positional => ParamKind::Positional,
        ParameterKind::Named => ParamKind::Named,
    }
}

/// Lowers a parsed annotation into resolver-compatible annotation IR.
pub(super) fn lower_annotation_ir(
    file_id: dust_text::FileId,
    annotation: &ParsedAnnotation,
    catalog: &SymbolCatalog,
) -> AnnotationIr {
    dust_resolver::resolve_annotation_ir(file_id, annotation, catalog)
}

/// Builds a name IR value from raw source and source span.
pub(super) fn lower_name_ir(
    file_id: dust_text::FileId,
    source: &str,
    span: dust_text::TextRange,
) -> NameIr {
    let source = source.trim().to_owned();
    let (prefix, short) = source
        .rsplit_once('.')
        .map(|(prefix, short)| (Some(prefix.to_owned()), short.to_owned()))
        .unwrap_or_else(|| (None, source.clone()));

    NameIr {
        source,
        short,
        prefix,
        span: SpanIr::new(file_id, span),
    }
}

/// Lowers one resolved class into semantic IR.
pub(super) struct ClassLoweringInput<'a> {
    /// Declaration kind.
    pub(super) kind: ClassKindIr,
    /// Class name.
    pub(super) name: &'a str,
    /// Whether the class is abstract.
    pub(super) is_abstract: bool,
    /// Whether the class is an interface class.
    pub(super) is_interface: bool,
    /// Immediate superclass name.
    pub(super) superclass_name: Option<&'a str>,
    /// Class source span.
    pub(super) span: SpanIr,
    /// Resolved fields.
    pub(super) fields: &'a mut [ResolvedField],
    /// Resolved constructors.
    pub(super) constructors: &'a [ResolvedConstructor],
    /// Resolved methods.
    pub(super) methods: &'a [ResolvedMethod],
    /// Resolved trait applications.
    pub(super) traits: &'a [TraitApplicationIr],
    /// Resolved configuration applications.
    pub(super) configs: &'a [ConfigApplicationIr],
    /// Resolver-normalized SerDe configuration.
    pub(super) serde_value: &'a mut Option<SerdeClassConfigIr>,
}

/// Lowers explicit class inputs into semantic IR.
pub(super) fn lower_class_from_parts(input: ClassLoweringInput<'_>) -> LoweringOutcome<ClassIr> {
    let ClassLoweringInput {
        kind,
        name,
        is_abstract,
        is_interface,
        superclass_name,
        span,
        fields,
        constructors,
        methods,
        traits,
        configs,
        serde_value,
    } = input;
    let mut diagnostics = Vec::new();
    let serde = serde_value.take();

    let fields = lower_resolved_fields(fields, &mut diagnostics);
    let methods = lower_resolved_methods(methods, &mut diagnostics);
    let constructors =
        lower_resolved_constructors(span.file_id, constructors, &fields, &mut diagnostics);

    LoweringOutcome {
        value: ClassIr {
            kind,
            name: name.to_owned(),
            is_abstract,
            is_interface,
            superclass_name: superclass_name.map(str::to_owned),
            span,
            fields,
            constructors,
            methods,
            traits: traits.to_vec(),
            configs: configs.to_vec(),
            serde,
        },
        diagnostics,
    }
}

/// Lowers resolved fields without requiring the owning class model.
pub(super) fn lower_resolved_fields(
    fields: &mut [ResolvedField],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<FieldIr> {
    fields
        .iter_mut()
        .map(|field| {
            let outcome = lower_type(field.parsed_type.as_ref(), field.type_source.as_deref());
            diagnostics.extend(outcome.diagnostics);
            FieldIr {
                name: field.name.clone(),
                ty: outcome.value,
                span: field.span,
                has_default: field.has_default,
                serde: field.serde.take(),
                configs: field.configs.clone(),
            }
        })
        .collect()
}

/// Lowers resolved methods without requiring the owning class model.
pub(super) fn lower_resolved_methods(
    methods: &[ResolvedMethod],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MethodIr> {
    methods
        .iter()
        .map(|method| {
            let return_type = lower_type(
                method.surface.parsed_return_type.as_ref(),
                method.surface.return_type_source.as_deref(),
            );
            diagnostics.extend(return_type.diagnostics);
            let params = method
                .params
                .iter()
                .map(|param| {
                    let ty = lower_type(
                        param.surface.parsed_type.as_ref(),
                        param.surface.type_source.as_deref(),
                    );
                    diagnostics.extend(ty.diagnostics);
                    MethodParamIr {
                        name: param.surface.name.clone(),
                        ty: ty.value,
                        span: param.span,
                        kind: lower_parameter_kind(param.surface.kind),
                        is_required: param.surface.is_required,
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
                return_type: return_type.value,
                has_body: method.surface.has_body,
                body_source: method.surface.body_source.clone(),
                params,
                span: method.span,
                traits: method.traits.clone(),
                configs: method.configs.clone(),
            }
        })
        .collect()
}

/// Lowers resolved constructors using lowered fields for type inference.
pub(super) fn lower_resolved_constructors(
    file_id: dust_text::FileId,
    constructors: &[ResolvedConstructor],
    fields: &[FieldIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ConstructorIr> {
    constructors
        .iter()
        .map(|constructor| {
            let params = constructor
                .surface
                .params
                .iter()
                .map(|param| {
                    let ty = param
                        .type_source
                        .as_deref()
                        .map(|source| lower_type(param.parsed_type.as_ref(), Some(source)))
                        .unwrap_or_else(|| infer_param_type(param.name.as_str(), fields));
                    diagnostics.extend(ty.diagnostics);
                    ConstructorParamIr {
                        name: param.name.clone(),
                        ty: ty.value,
                        span: SpanIr::new(file_id, param.span),
                        kind: lower_parameter_kind(param.kind),
                        has_default: param.has_default,
                        default_value_source: param.default_value_source.clone(),
                    }
                })
                .collect();
            ConstructorIr {
                name: constructor.surface.name.clone(),
                is_factory: constructor.surface.is_factory,
                redirected_target_source: constructor.surface.redirected_target_source.clone(),
                redirected_target_name: constructor.surface.redirected_target_name.clone(),
                span: SpanIr::new(file_id, constructor.surface.span),
                params,
            }
        })
        .collect()
}
