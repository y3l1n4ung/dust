use dust_dart_syntax::{balanced_parenthesized, split_top_level_items};

/// Renders an expression after a prefix while preserving readable wrapping.
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

/// Wraps a simple call expression into multiple lines when it is too long.
fn wrap_call_expr(expr: &str, indent: usize, prefix: &str, suffix: &str) -> Option<String> {
    let open = expr.find('(')?;
    let args_source = balanced_parenthesized(&expr[open..])?;
    if open + args_source.len() != expr.len() {
        return None;
    }
    let callee = &expr[..open];
    let args = split_top_level_items(&args_source[1..args_source.len() - 1]);
    if args.is_empty() {
        return None;
    }

    let pad = " ".repeat(indent);
    let mut rendered = vec![format!("{pad}{prefix}{callee}(")];
    rendered.extend(args.into_iter().map(|arg| wrap_arg_expr(arg, indent + 2)));
    rendered.push(format!("{pad}){suffix}"));
    Some(rendered.join("\n"))
}

/// Wraps one call argument if the argument is itself too long.
fn wrap_arg_expr(arg: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let candidate = format!("{pad}{arg},");
    if candidate.len() <= 80 {
        return candidate;
    }

    wrap_call_expr(arg, indent, "", ",").unwrap_or(candidate)
}

#[cfg(test)]
mod tests {
    use super::format_prefixed_expr;

    #[test]
    fn leaves_chained_call_expressions_unwrapped() {
        let expr = "JsonHelper.as<num>(json['subtotal'], 'subtotal', 'num').toDouble()";

        assert_eq!(
            format_prefixed_expr(2, "final subtotalValue = ", expr, ";"),
            "  final subtotalValue = JsonHelper.as<num>(json['subtotal'], 'subtotal', 'num').toDouble();"
        );
    }
}
