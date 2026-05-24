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
    values: &[(&str, String)],
) -> Option<String> {
    let args = constructor_args(constructor, values)?;
    let ctor = constructor_name(class, constructor);

    if args.is_empty() {
        return Some(format!("{ctor}()"));
    }

    let lines = args
        .into_iter()
        .map(|arg| render_constructor_arg(&arg))
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!("{ctor}(\n{lines}\n)"))
}

fn render_constructor_arg(arg: &str) -> String {
    let mut lines = arg.lines();
    let Some(first) = lines.next() else {
        return "  ,".to_owned();
    };
    let mut rendered = vec![format!("  {first}")];
    for line in lines {
        rendered.push(format!("      {}", line.trim_start()));
    }
    if let Some(last) = rendered.last_mut() {
        last.push(',');
    }
    rendered.join("\n")
}

pub(crate) fn render_return_statement(call: &str, indent: &str) -> String {
    let mut lines = call.lines();
    let Some(first_line) = lines.next() else {
        return format!("{indent}return;");
    };

    let mut rendered = vec![format!("{indent}return {first_line}")];
    let remaining = lines.collect::<Vec<_>>();

    if remaining.is_empty() {
        rendered[0].push(';');
        return rendered.join("\n");
    }

    for line in remaining {
        if line == ")" {
            rendered.push(format!("{indent}{line};"));
        } else {
            rendered.push(format!("{indent}{line}"));
        }
    }

    rendered.join("\n")
}

fn constructor_args(constructor: &ConstructorIr, values: &[(&str, String)]) -> Option<Vec<String>> {
    let mut positional = Vec::new();
    let mut named = Vec::new();

    for param in &constructor.params {
        let value = values
            .iter()
            .find(|(name, _)| *name == param.name)
            .map(|(_, value)| value.clone())?;

        match param.kind {
            ParamKind::Positional => positional.push(value),
            ParamKind::Named => named.push(format!("{}: {}", param.name, value)),
        }
    }

    let mut args = positional;
    args.extend(named);
    Some(args)
}

fn constructor_name(class: &ClassIr, constructor: &ConstructorIr) -> String {
    match &constructor.name {
        Some(name) => format!("{}.{}", class.name, name),
        None => class.name.clone(),
    }
}
