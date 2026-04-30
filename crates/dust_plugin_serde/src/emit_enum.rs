use dust_ir::EnumIr;

pub(crate) fn emit_enum_from_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = variant_wire_name(e, &variant.name);
        cases.push(format!("    '{}' => {}.{},", key, e.name, variant.name));
    }

    format!(
        "{} _${}FromJson(Object? json) {{\n  return switch (json) {{\n{}\n    _ => throw ArgumentError.value(json, 'json', 'unknown value for {}'),\n  }};\n}}",
        e.name,
        e.name,
        cases.join("\n"),
        e.name
    )
}

pub(crate) fn emit_enum_to_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = variant_wire_name(e, &variant.name);
        cases.push(format!("    {}.{} => '{}',", e.name, variant.name, key));
    }

    format!(
        "Object? _${}ToJson({} instance) {{\n  return switch (instance) {{\n{}\n  }};\n}}",
        e.name,
        e.name,
        cases.join("\n")
    )
}

fn variant_wire_name(e: &EnumIr, variant_name: &str) -> String {
    match e.serde.as_ref().and_then(|s| s.rename_all) {
        Some(rule) => crate::writer::apply_rename_rule(variant_name, rule),
        None => variant_name.to_owned(),
    }
}
