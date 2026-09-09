//! Rendering a method's parameter list, and the map entries a request body builds.

use super::*;

/// Renders a Dart method parameter list from positional and named parameters.
pub(super) fn render_method_parameters(
    params: &[dust_ir::MethodParamIr],
    return_type: &str,
    method_name: &str,
    async_marker: &str,
) -> String {
    let positional = params
        .iter()
        .filter(|param| param.kind == dust_ir::ParamKind::Positional)
        .map(render_method_parameter)
        .collect::<Vec<_>>();
    let named = params
        .iter()
        .filter(|param| param.kind == dust_ir::ParamKind::Named)
        .map(render_method_parameter)
        .collect::<Vec<_>>();

    let one_line = match (positional.is_empty(), named.is_empty()) {
        (true, true) => "()".to_owned(),
        (false, true) => format!("({})", positional.join(", ")),
        (true, false) => format!("({{{}}})", named.join(", ")),
        (false, false) => format!("({}, {{{}}})", positional.join(", "), named.join(", ")),
    };
    let signature_len = format!("  {return_type} {method_name}{one_line} {async_marker} {{").len();
    if signature_len <= 88 && params.len() <= 2 {
        return one_line;
    }

    match (positional.is_empty(), named.is_empty()) {
        (true, true) => "()".to_owned(),
        (false, true) => multiline_parameters("(", &positional, "  )"),
        (true, false) => multiline_parameters("({", &named, "  })"),
        (false, false) => {
            let mut lines = vec!["(".to_owned()];
            for (index, param) in positional.iter().enumerate() {
                let suffix = if index + 1 == positional.len() {
                    ", {"
                } else {
                    ","
                };
                lines.push(format!("    {param}{suffix}"));
            }
            lines.extend(named.iter().map(|param| format!("    {param},")));
            lines.push("  })".to_owned());
            lines.join("\n")
        }
    }
}

/// Renders a multi-line Dart parameter list with stable indentation.
pub(super) fn multiline_parameters(open: &str, params: &[String], close: &str) -> String {
    let mut lines = vec![open.to_owned()];
    lines.extend(params.iter().map(|param| format!("    {param},")));
    lines.push(close.to_owned());
    lines.join("\n")
}

/// Renders a single Dart method parameter with `required` when needed.
pub(super) fn render_method_parameter(param: &dust_ir::MethodParamIr) -> String {
    let default = param
        .default_value_source
        .as_deref()
        .map_or(String::new(), |source| format!(" = {source}"));
    if param.kind == dust_ir::ParamKind::Named && param.is_required {
        format!("required {} {}", render_type(&param.ty), param.name)
    } else {
        format!("{} {}{default}", render_type(&param.ty), param.name)
    }
}

/// Returns the user-supplied Dio options parameter, if present.
pub(super) fn option_param<'a>(
    endpoint: &'a EndpointSpec<'_>,
) -> Option<&'a dust_ir::MethodParamIr> {
    endpoint.params.iter().find_map(|param| match param {
        EndpointParam::Options { param } => Some(*param),
        _ => None,
    })
}

/// Finds the first special Dio parameter name matching a predicate.
pub(super) fn param_name<'a, F>(endpoint: &'a EndpointSpec<'_>, matches: F) -> Option<&'a str>
where
    F: Fn(&EndpointParam<'_>) -> bool,
{
    endpoint
        .params
        .iter()
        .find(|param| matches(param))
        .map(|param| match param {
            EndpointParam::CancelToken { param }
            | EndpointParam::Options { param }
            | EndpointParam::OnSendProgress { param }
            | EndpointParam::OnReceiveProgress { param } => param.name.as_str(),
            _ => unreachable!("filtered to special params"),
        })
}

/// Renders assignment for a single generated request map entry.
pub(super) fn render_map_entry(target: &str, name: &str, key: &str, nullable: bool) -> String {
    render_map_entry_with_guard(target, name, name, key, nullable)
}

/// Renders assignment for a single generated request map entry with a custom null guard.
pub(super) fn render_map_entry_with_guard(
    target: &str,
    name: &str,
    guard: &str,
    key: &str,
    nullable: bool,
) -> String {
    if nullable {
        render_template(
            "map_entry_nullable",
            include_str!("../templates/map_entry_nullable.jinja"),
            MapEntryContext {
                target,
                name,
                guard,
                key: crate::plugin::util::escape_single_quoted(key),
            },
        )
    } else {
        render_template(
            "map_entry",
            include_str!("../templates/map_entry.jinja"),
            MapEntryContext {
                target,
                name,
                guard,
                key: crate::plugin::util::escape_single_quoted(key),
            },
        )
    }
}

/// Renders merging of a generated request map.
pub(super) fn render_map_merge(target: &str, name: &str, nullable: bool) -> String {
    if nullable {
        render_template(
            "map_merge_nullable",
            include_str!("../templates/map_merge_nullable.jinja"),
            MapMergeContext { target, name },
        )
    } else {
        render_template(
            "map_merge",
            include_str!("../templates/map_merge.jinja"),
            MapMergeContext { target, name },
        )
    }
}

/// Ensures a rendered chunk ends with a newline when non-empty.
pub(super) fn chunk(mut value: String) -> String {
    if !value.is_empty() && !value.ends_with('\n') {
        value.push('\n');
    }
    value
}

/// Joins rendered chunks while normalizing their trailing newlines.
pub(super) fn join_chunks(chunks: Vec<String>) -> String {
    chunks.into_iter().map(chunk).collect()
}
