use dust_diagnostics::Diagnostic;
use dust_ir::{BuiltinType, ClassIr, ClassKindIr, LibraryIr, TypeIr};

pub(crate) fn validate_library(library: &LibraryIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let known_models = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<Vec<_>>();

    for class in &library.classes {
        let serialize = wants_serialize(class);
        let deserialize = wants_deserialize(class);
        if !serialize && !deserialize {
            continue;
        }

        if matches!(class.kind, ClassKindIr::MixinClass) {
            diagnostics.push(Diagnostic::error(format!(
                "Dust serde generation does not support `mixin class` targets like `{}`",
                class.name
            )));
            continue;
        }

        if deserialize && class.is_abstract {
            diagnostics.push(Diagnostic::error(format!(
                "`Deserialize` cannot target abstract class `{}`",
                class.name
            )));
        }

        if deserialize
            && !class
                .constructors
                .iter()
                .any(|ctor| ctor.can_construct_all_fields(&class.fields))
        {
            diagnostics.push(Diagnostic::error(format!(
                "`Deserialize` requires a constructor that can initialize every field on class `{}`",
                class.name
            )));
        }

        if class
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_ref())
            .is_some()
        {
            diagnostics.push(Diagnostic::error(format!(
                "class `{}` does not support `SerDe(rename: ...)` in Dust serde generation",
                class.name
            )));
        }

        for field in &class.fields {
            if let Some(serde) = &field.serde {
                if serde.skip_deserializing && serde.default_value_source.is_none() {
                    diagnostics.push(Diagnostic::error(format!(
                        "field `{}` on class `{}` uses `skipDeserializing` without a `defaultValue`",
                        field.name, class.name
                    )));
                }
            }

            let uses_codec = field
                .serde
                .as_ref()
                .and_then(|serde| serde.codec_source.as_ref())
                .is_some();

            if serialize && !uses_codec {
                validate_type_supported(
                    &field.ty,
                    known_models.as_slice(),
                    &class.name,
                    &field.name,
                    "Serialize",
                    &mut diagnostics,
                );
            }
            if deserialize && !uses_codec {
                validate_type_supported(
                    &field.ty,
                    known_models.as_slice(),
                    &class.name,
                    &field.name,
                    "Deserialize",
                    &mut diagnostics,
                );
            }
        }
    }

    diagnostics
}

fn validate_type_supported(
    ty: &TypeIr,
    known_models: &[&str],
    class_name: &str,
    field_name: &str,
    direction: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic => {}
        TypeIr::Unknown => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support unresolved type on `{class_name}.{field_name}`"
        ))),
        TypeIr::Function { .. } => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support function types on `{class_name}.{field_name}`"
        ))),
        TypeIr::Record { .. } => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support record types on `{class_name}.{field_name}`"
        ))),
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" || name.as_ref() == "Set" => {
            if let Some(item) = args.first() {
                validate_type_supported(
                    item,
                    known_models,
                    class_name,
                    field_name,
                    direction,
                    diagnostics,
                );
            } else {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` requires one type argument for `{name}` on `{class_name}.{field_name}`"
                )));
            }
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => {
            if args.len() != 2 {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` requires two type arguments for `Map` on `{class_name}.{field_name}`"
                )));
                return;
            }
            if !args[0].is_builtin(BuiltinType::String) {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` only supports `Map<String, T>` on `{class_name}.{field_name}`"
                )));
            }
            validate_type_supported(
                &args[1],
                known_models,
                class_name,
                field_name,
                direction,
                diagnostics,
            );
        }
        TypeIr::Named { name, args, .. } => {
            if !args.is_empty() {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` does not yet support generic named type `{name}` on `{class_name}.{field_name}`"
                )));
            } else if name.as_ref() == "Object" {
                // handled by builtins in most cases, but keep as supported fallback
            } else if !known_models.iter().any(|item| *item == name.as_ref()) {
                // External models are allowed; callers can provide `fromJson` / `toJson`.
            }
        }
    }
}

fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
}

fn wants_deserialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
}
