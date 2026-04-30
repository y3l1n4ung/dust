use std::collections::HashSet;

use dust_ir::{ClassIr, ConstructorIr, ParamKind, SerdeFieldConfigIr};

pub(crate) fn find_deserialize_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

pub(crate) fn json_key(
    class: &ClassIr,
    field_name: &str,
    field_serde: Option<&SerdeFieldConfigIr>,
) -> String {
    if let Some(rename) = field_serde.and_then(|serde| serde.rename.as_deref()) {
        return rename.to_owned();
    }

    match class.serde.as_ref().and_then(|serde| serde.rename_all) {
        Some(rule) => crate::writer_type::apply_rename_rule(field_name, rule),
        None => field_name.to_owned(),
    }
}

pub(crate) fn all_allowed_keys(class: &ClassIr) -> Vec<String> {
    let mut keys = Vec::new();
    let mut seen = HashSet::new();
    for field in &class.fields {
        let Some(serde) = &field.serde else {
            let key = json_key(class, &field.name, None);
            if seen.insert(key.clone()) {
                keys.push(key);
            }
            continue;
        };

        let key = json_key(class, &field.name, Some(serde));
        if seen.insert(key.clone()) {
            keys.push(key);
        }
        for alias in &serde.aliases {
            if seen.insert(alias.clone()) {
                keys.push(alias.clone());
            }
        }
    }
    keys
}

pub(crate) fn render_constructor_call(
    class: &ClassIr,
    constructor: &ConstructorIr,
    values: &[(&str, String)],
) -> Option<String> {
    let mut positional = Vec::new();
    let mut named = Vec::new();

    for param in &constructor.params {
        let value = values
            .iter()
            .find(|(name, _)| *name == param.name)
            .map(|(_, value)| value.clone());

        let Some(value) = value else {
            if param.has_default {
                continue;
            }
            return None;
        };

        match param.kind {
            ParamKind::Positional => positional.push(value),
            ParamKind::Named => named.push(format!("{}: {}", param.name, value)),
        }
    }

    let ctor = match &constructor.name {
        Some(name) => format!("{}.{}", class.name, name),
        None => class.name.clone(),
    };

    let mut args = positional;
    args.extend(named);
    if args.is_empty() {
        return Some(format!("{ctor}()"));
    }

    let rendered_args = args
        .into_iter()
        .map(|arg| format!("  {arg},"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("{ctor}(\n{rendered_args}\n)"))
}
