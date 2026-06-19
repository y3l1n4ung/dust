use std::{borrow::Cow, fmt::Write};

use dust_ir::{ClassIr, ConstructorIr, ParamKind};

pub(crate) fn find_clone_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class.constructors.iter().find(|constructor| {
        class.fields.iter().all(|field| {
            constructor
                .params
                .iter()
                .any(|param| param.name == field.name)
        })
    })
}

pub(crate) fn build_constructor_call_multiline(
    class: &ClassIr,
    constructor: &ConstructorIr,
    values: &[(&str, Cow<'_, str>)],
) -> Option<String> {
    let ctor = constructor_name(class, constructor);

    if constructor.params.is_empty() {
        return Some(format!("{ctor}()"));
    }

    let mut out = String::with_capacity(ctor.len() + constructor.params.len() * 32);
    writeln!(&mut out, "{ctor}(").ok()?;
    for param in &constructor.params {
        let value = values
            .iter()
            .find(|(name, _)| *name == param.name)
            .map(|(_, value)| value.as_ref())?;
        match param.kind {
            ParamKind::Positional => render_constructor_arg(&mut out, value),
            ParamKind::Named => {
                write!(&mut out, "  {}: ", param.name).ok()?;
                render_arg_value(&mut out, value, "      ");
                out.push(',');
                out.push('\n');
            }
        }
    }
    out.push(')');
    Some(out)
}

fn render_constructor_arg(out: &mut String, arg: &str) {
    let mut lines = arg.lines();
    let Some(first) = lines.next() else {
        out.push_str("  ,\n");
        return;
    };
    out.push_str("  ");
    out.push_str(first);
    for line in lines {
        out.push('\n');
        out.push_str("      ");
        out.push_str(line.trim_start());
    }
    out.push(',');
    out.push('\n');
}

fn render_arg_value(out: &mut String, value: &str, continuation_indent: &str) {
    let mut lines = value.lines();
    let Some(first) = lines.next() else {
        return;
    };
    out.push_str(first);
    for line in lines {
        out.push('\n');
        out.push_str(continuation_indent);
        out.push_str(line.trim_start());
    }
}

fn constructor_name(class: &ClassIr, constructor: &ConstructorIr) -> String {
    match &constructor.name {
        Some(name) => format!("{}.{}", class.name, name),
        None => class.name.clone(),
    }
}
