use std::collections::BTreeMap;

use crate::plugin::model::{EndpointParam, EndpointSpec};

pub(super) enum PathSegment<'a> {
    Literal(&'a str),
    Binding { key: &'a str, param_name: &'a str },
}

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
