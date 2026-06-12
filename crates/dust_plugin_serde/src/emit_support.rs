use dust_dart_syntax::split_top_level_items;

pub(crate) fn format_prefixed_expr(
    indent: usize,
    prefix: &str,
    expr: &str,
    suffix: &str,
) -> String {
    let pad = " ".repeat(indent);
    let continuation = " ".repeat(indent + 4);
    let mut lines = expr.lines();
    let Some(first) = lines.next() else {
        return format!("{pad}{prefix}{suffix}");
    };

    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        let candidate = format!("{pad}{prefix}{first}{suffix}");
        if candidate.len() <= 80 {
            return candidate;
        }
        if let Some(wrapped) = wrap_call_expr(first, indent, prefix, suffix) {
            return wrapped;
        }
        return candidate;
    }

    let common_indent = rest
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| *ch == ' ').count())
        .min()
        .unwrap_or(0);
    let mut rendered = Vec::with_capacity(rest.len() + 1);
    rendered.push(format!("{pad}{prefix}{first}"));
    for (index, line) in rest.iter().enumerate() {
        let tail = if index + 1 == rest.len() { suffix } else { "" };
        let stripped = line
            .get(common_indent..)
            .unwrap_or_else(|| line.trim_start());
        rendered.push(format!("{continuation}{stripped}{tail}"));
    }
    rendered.join("\n")
}

fn wrap_call_expr(expr: &str, indent: usize, prefix: &str, suffix: &str) -> Option<String> {
    let open = expr.find('(')?;
    let close = expr.rfind(')')?;
    if close + 1 != expr.len() {
        return None;
    }
    let callee = &expr[..open];
    let args = split_top_level_items(&expr[open + 1..close]);
    if args.len() <= 1 {
        return None;
    }

    let pad = " ".repeat(indent);
    let arg_pad = " ".repeat(indent + 2);
    let mut rendered = vec![format!("{pad}{prefix}{callee}(")];
    rendered.extend(args.into_iter().map(|arg| format!("{arg_pad}{arg},")));
    rendered.push(format!("{pad}){suffix}"));
    Some(rendered.join("\n"))
}
