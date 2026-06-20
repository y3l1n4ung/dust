use std::collections::BTreeMap;

use crate::plugin::model::{EndpointParam, EndpointSpec};

/// Segment in a generated HTTP path expression.
pub(super) enum PathSegment<'a> {
    /// Literal path text copied from the annotation template.
    Literal(&'a str),
    /// Placeholder replaced by an encoded Dart parameter value.
    Binding {
        /// Placeholder key from the path template.
        key: &'a str,
        /// Dart parameter name bound to the placeholder.
        param_name: &'a str,
    },
}

/// Splits an endpoint path template into literals and parameter bindings.
pub(super) fn path_segments<'a>(endpoint: &'a EndpointSpec<'a>) -> Vec<PathSegment<'a>> {
    let path_bindings = endpoint
        .params
        .iter()
        .filter_map(|param| match param {
            EndpointParam::Path { param, key } => Some((key.as_str(), param.name.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();

    let mut segments = Vec::new();
    let mut cursor = 0_usize;
    while let Some(start) = endpoint.path[cursor..].find('{') {
        let start = cursor + start;
        let Some(end) = endpoint.path[start + 1..].find('}') else {
            break;
        };
        let end = start + 1 + end;
        if start > cursor {
            segments.push(PathSegment::Literal(&endpoint.path[cursor..start]));
        }
        let key = &endpoint.path[start + 1..end];
        let param_name = path_bindings.get(key).copied().unwrap_or(key);
        segments.push(PathSegment::Binding { key, param_name });
        cursor = end + 1;
    }

    if cursor < endpoint.path.len() {
        segments.push(PathSegment::Literal(&endpoint.path[cursor..]));
    }

    segments
}
