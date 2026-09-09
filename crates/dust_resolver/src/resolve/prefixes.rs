//! Checking that annotations reached through an import prefix are spelled with it.

use super::*;

/// Reports annotation prefixes that are not declared by an import directive.
pub(super) fn validate_annotation_prefixes(
    file_id: FileId,
    library: &ParsedDartFileSurface,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let prefixes = library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import {
                prefix: Some(prefix),
                ..
            } => Some(prefix.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    let mut check = |annotations: &[ParsedAnnotation]| {
        for annotation in annotations {
            let Some(prefix) = annotation.prefix.as_deref() else {
                continue;
            };
            if prefixes.contains(prefix) {
                continue;
            }
            diagnostics.push(
                Diagnostic::error(format!(
                    "annotation prefix `{prefix}` is not declared by an import"
                ))
                .with_label(SourceLabel::new(
                    file_id,
                    annotation.span,
                    format!("add `as {prefix}` to the matching import or remove the prefix"),
                )),
            );
        }
    };

    for directive in &library.directives {
        if let ParsedDirective::Library { annotations, .. } = directive {
            check(annotations);
        }
    }
    for class in &library.classes {
        check(&class.annotations);
        for field in &class.fields {
            check(&field.annotations);
        }
        for constructor in &class.constructors {
            check(&constructor.annotations);
            for param in &constructor.params {
                check(&param.annotations);
            }
        }
        for method in &class.methods {
            check(&method.annotations);
            for param in &method.params {
                check(&param.annotations);
            }
        }
    }
    for enum_surface in &library.enums {
        check(&enum_surface.annotations);
        for variant in &enum_surface.variants {
            check(&variant.annotations);
        }
    }
    for mixin in &library.mixins {
        check(&mixin.annotations);
        for field in &mixin.fields {
            check(&field.annotations);
        }
    }
    for extension in &library.extensions {
        check(&extension.annotations);
    }
    for extension_type in &library.extension_types {
        check(&extension_type.annotations);
    }
    for function in &library.functions {
        check(&function.annotations);
        for param in &function.params {
            check(&param.annotations);
        }
    }
    for variable in &library.variables {
        check(&variable.annotations);
    }
    for typedef in &library.typedefs {
        check(&typedef.annotations);
    }
}
