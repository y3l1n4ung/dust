use dust_dart_emit::render_template;
use dust_ir::EnumIr;
use serde::Serialize;

#[derive(Serialize)]
struct EnumTemplateContext<'a> {
    enum_name: &'a str,
    cases: String,
}

pub(crate) fn emit_enum_from_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = variant_wire_name(e, &variant.name);
        cases.push(format!("    '{}' => {}.{},", key, e.name, variant.name));
    }

    render_template(
        "enum_from_json",
        include_str!("templates/enum_from_json.jinja"),
        EnumTemplateContext {
            enum_name: &e.name,
            cases: cases.join("\n"),
        },
    )
}

pub(crate) fn emit_enum_to_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = variant_wire_name(e, &variant.name);
        cases.push(format!("    {}.{} => '{}',", e.name, variant.name, key));
    }

    render_template(
        "enum_to_json",
        include_str!("templates/enum_to_json.jinja"),
        EnumTemplateContext {
            enum_name: &e.name,
            cases: cases.join("\n"),
        },
    )
}

fn variant_wire_name(e: &EnumIr, variant_name: &str) -> String {
    match e.serde.as_ref().and_then(|s| s.rename_all) {
        Some(rule) => crate::writer::apply_rename_rule(variant_name, rule),
        None => variant_name.to_owned(),
    }
}
