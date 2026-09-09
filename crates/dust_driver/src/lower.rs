/// Lowering for top-level declarations.
mod declarations;
/// Lowering for library, import, export and part directives.
mod directives;
/// Inherited field and constructor parameter lowering helpers.
mod inheritance;
/// Lowering for class members.
mod members;
mod tests_declarations;
mod tests_directives;
mod tests_inheritance;

use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationIr, ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr,
    DartFileIr, EnumIr, EnumVariantIr, ExportIr, ExprSourceIr, ExtensionIr, ExtensionTypeIr,
    FieldIr, FunctionIr, ImportIr, LibraryDeclIr, LoweringOutcome, MethodIr, MethodParamIr,
    MixinIr, NameIr, ParamKind, PartIr, PartOfIr, SerdeClassConfigIr, SpanIr, TopLevelVariableIr,
    TraitApplicationIr, TypedefIr,
};
use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedDirective, ParsedExtensionSurface,
    ParsedExtensionTypeSurface, ParsedFieldSurface, ParsedFunctionSurface,
    ParsedMethodParamSurface, ParsedMixinSurface, ParsedTopLevelVariableSurface,
    ParsedTypedefSurface,
};
use dust_resolver::{
    ResolvedConstructor, ResolvedField, ResolvedLibrary, ResolvedMethod, SymbolCatalog,
    lower_type_ir as lower_type,
};

use self::inheritance::{
    infer_param_type, merged_fields_for_class, resolve_constructor_param_types,
};
use self::{declarations::*, directives::*, members::*};

/// Lowers one resolved library into semantic IR.
#[cfg(test)]
pub(crate) fn lower_library(library: &mut ResolvedLibrary) -> LoweringOutcome<DartFileIr> {
    lower_library_with_catalog(library, &SymbolCatalog::new())
}

/// Lowers one resolved library and attaches registered annotation symbols.
pub(crate) fn lower_library_with_catalog(
    library: &mut ResolvedLibrary,
    catalog: &SymbolCatalog,
) -> LoweringOutcome<DartFileIr> {
    let mut diagnostics = Vec::new();
    let collect_diagnostics_by_class = library
        .classes
        .iter()
        .map(|class| class.requires_lowering_diagnostics)
        .collect::<Vec<_>>();
    let mut classes = library
        .classes
        .iter_mut()
        .enumerate()
        .map(|(index, class)| {
            let collect_diagnostics = collect_diagnostics_by_class[index];
            let outcome = lower_class_from_parts(ClassLoweringInput {
                kind: class.kind,
                name: &class.name,
                is_abstract: class.is_abstract,
                is_interface: class.is_interface,
                superclass_name: class.superclass_name.as_deref(),
                span: class.span,
                fields: &mut class.fields,
                constructors: &class.constructors,
                methods: &class.methods,
                traits: &class.traits,
                configs: &class.configs,
                serde_value: &mut class.serde,
            });
            if collect_diagnostics {
                diagnostics.extend(outcome.diagnostics);
            }
            outcome.value
        })
        .collect::<Vec<_>>();
    let enums = library
        .enums
        .iter_mut()
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
        let mut constructor_diagnostics = Vec::new();
        resolve_constructor_param_types(&mut classes[index], &mut constructor_diagnostics);
        if collect_diagnostics_by_class[index] {
            diagnostics.extend(constructor_diagnostics);
        }
    }

    LoweringOutcome {
        value: DartFileIr {
            package_root: String::new(),
            package_name: String::new(),
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            imports: library_imports(&library.directives),
            library: lower_library_directive(library.span.file_id, &library.directives),
            library_annotations: lower_library_annotations(
                library.span.file_id,
                &library.directives,
                catalog,
            ),
            import_directives: lower_import_directives(library.span.file_id, &library.directives),
            export_directives: lower_export_directives(library.span.file_id, &library.directives),
            part_directives: lower_part_directives(library.span.file_id, &library.directives),
            part_of: lower_part_of_directive(library.span.file_id, &library.directives),
            span: library.span,
            classes,
            mixins: lower_mixins(
                library.span.file_id,
                &library.mixins,
                catalog,
                &mut diagnostics,
            ),
            extensions: lower_extensions(
                library.span.file_id,
                &library.extensions,
                catalog,
                &mut diagnostics,
            ),
            extension_types: lower_extension_types(
                library.span.file_id,
                &library.extension_types,
                catalog,
                &mut diagnostics,
            ),
            functions: lower_functions(
                library.span.file_id,
                &library.functions,
                catalog,
                &mut diagnostics,
            ),
            variables: lower_variables(
                library.span.file_id,
                &library.variables,
                catalog,
                &mut diagnostics,
            ),
            typedefs: lower_typedefs(
                library.span.file_id,
                &library.typedefs,
                catalog,
                &mut diagnostics,
            ),
            enums,
            query_calls: std::mem::take(&mut library.query_calls),
        },
        diagnostics,
    }
}
