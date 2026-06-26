use std::collections::HashSet;

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, DartFileIr, FieldIr, ParamKind,
    SerdeClassConfigIr, SerdeFieldConfigIr,
};

/// A concrete sealed variant class generated from a redirecting factory.
pub(crate) struct GeneratedVariantClass {
    /// The synthesized class model used by existing SerDe helpers.
    pub(crate) class: ClassIr,
    /// The sealed base class this variant extends.
    pub(crate) base_class_name: String,
    /// Whether to emit serialization helpers for this variant.
    pub(crate) serializable: bool,
    /// Whether to emit deserialization helpers for this variant.
    pub(crate) deserializable: bool,
}

/// Returns variant classes missing from user source and therefore generated.
pub(crate) fn generated_variant_classes(library: &DartFileIr) -> Vec<GeneratedVariantClass> {
    let existing = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();
    let mut generated = Vec::new();

    for base in &library.classes {
        let Some(serde) = &base.serde else {
            continue;
        };
        for variant in &serde.variants {
            if existing.contains(variant.target_class_name.as_str()) {
                continue;
            }
            generated.push(GeneratedVariantClass {
                class: synthesize_class(base, serde, variant),
                base_class_name: base.name.clone(),
                serializable: wants_trait(base, "dust_dart::Serialize"),
                deserializable: wants_trait(base, "dust_dart::Deserialize"),
            });
        }
    }

    generated
}

/// Renders the Dart source for one generated concrete variant class.
pub(crate) fn emit_generated_variant_class(variant: &GeneratedVariantClass) -> String {
    let class = &variant.class;
    let mut lines = vec![format!(
        "final class {} extends {} {{",
        class.name, variant.base_class_name
    )];
    lines.extend(render_constructor(class));
    if variant.deserializable {
        lines.push(String::new());
        lines.push(format!(
            "  factory {}.fromJson(Map<String, Object?> json) =>",
            class.name
        ));
        lines.push(format!("      _${}FromJson(json);", class.name));
    }
    if !class.fields.is_empty() {
        lines.push(String::new());
    }
    for field in &class.fields {
        lines.push(format!(
            "  final {} {};",
            DYNAMIC_TYPES.render(&field.ty),
            field.name
        ));
    }
    lines.push("}".to_owned());
    lines.join("\n")
}

/// Builds a class model for an omitted redirect target.
fn synthesize_class(
    base: &ClassIr,
    base_serde: &SerdeClassConfigIr,
    variant: &dust_ir::SerdeVariantConfigIr,
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: variant.target_class_name.clone(),
        is_abstract: false,
        is_interface: false,
        superclass_name: Some(base.name.clone()),
        span: base.span,
        fields: variant.params.iter().map(field_from_param).collect(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: base.span,
            params: variant.params.clone(),
        }],
        methods: Vec::new(),
        traits: base.traits.clone(),
        configs: Vec::new(),
        serde: Some(SerdeClassConfigIr {
            rename_all: base_serde.rename_all,
            disallow_unrecognized_keys: base_serde.disallow_unrecognized_keys,
            ..SerdeClassConfigIr::default()
        }),
    }
}

/// Converts one factory parameter into a generated final field.
fn field_from_param(param: &ConstructorParamIr) -> FieldIr {
    FieldIr {
        name: param.name.clone(),
        ty: param.ty.clone(),
        span: param.span,
        has_default: false,
        serde: param
            .default_value_source
            .as_ref()
            .map(|default| SerdeFieldConfigIr {
                default_value_source: Some(default.clone()),
                ..SerdeFieldConfigIr::default()
            }),
        configs: Vec::new(),
    }
}

/// Renders the generated const constructor.
fn render_constructor(class: &ClassIr) -> Vec<String> {
    let params = class
        .constructors
        .first()
        .map(|constructor| constructor.params.as_slice())
        .unwrap_or(&[]);
    if params.is_empty() {
        return vec![format!("  const {}() : super();", class.name)];
    }

    let positional = params
        .iter()
        .filter(|param| param.kind == ParamKind::Positional)
        .map(|param| format!("    this.{},", param.name))
        .collect::<Vec<_>>();
    let named_params = params
        .iter()
        .filter(|param| param.kind == ParamKind::Named)
        .collect::<Vec<_>>();

    if positional.is_empty() && !named_params.is_empty() {
        let mut lines = vec![format!("  const {}({{", class.name)];
        lines.extend(
            named_params
                .iter()
                .map(|param| render_named_param(param, 4)),
        );
        lines.push("  }) : super();".to_owned());
        return lines;
    }

    let mut lines = vec![format!("  const {}(", class.name)];
    lines.extend(positional);
    if !named_params.is_empty() {
        lines.push("    {".to_owned());
        lines.extend(
            named_params
                .iter()
                .map(|param| render_named_param(param, 6)),
        );
        lines.push("    },".to_owned());
    }
    lines.push("  ) : super();".to_owned());
    lines
}

/// Renders one named constructor parameter.
fn render_named_param(param: &ConstructorParamIr, indent: usize) -> String {
    let prefix = if !param.has_default && !param.ty.is_nullable() {
        "required "
    } else {
        ""
    };
    let padding = " ".repeat(indent);
    match param.default_value_source.as_deref() {
        Some(default) => format!("{padding}{prefix}this.{} = {default},", param.name),
        None => format!("{padding}{prefix}this.{},", param.name),
    }
}

/// Returns whether a class has a resolved trait.
fn wants_trait(class: &ClassIr, symbol: &str) -> bool {
    class.traits.iter().any(|item| item.symbol.0 == symbol)
}
