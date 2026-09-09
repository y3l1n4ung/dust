//! The normalized config forms a plugin reads instead of raw annotation values.

use super::*;

/// Resolver-normalized payload for a known configuration annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedConfigIr {
    /// Typed `AppRoute` configuration.
    Route(RouteConfigIr),
    /// Typed `AppRouter` configuration.
    Router(RouterConfigIr),
    /// Typed `ViewModel` configuration.
    State(StateConfigIr),
    /// Typed HTTP client, endpoint, and binding configuration.
    Http(HttpConfigIr),
    /// Typed database, row, and query configuration.
    Db(DbConfigIr),
}

/// Returns source text for annotation values that are backed by raw source.
pub(super) fn annotation_value_source(value: &AnnotationValueIr) -> Option<&str> {
    match value {
        AnnotationValueIr::Number { source, .. } => Some(source),
        AnnotationValueIr::Member(name) => Some(name.source.as_str()),
        AnnotationValueIr::Expression(source) => Some(source.source.as_str()),
        AnnotationValueIr::Null
        | AnnotationValueIr::Bool(_)
        | AnnotationValueIr::String(_)
        | AnnotationValueIr::List(_)
        | AnnotationValueIr::Set(_)
        | AnnotationValueIr::Map(_)
        | AnnotationValueIr::Record(_)
        | AnnotationValueIr::Constructor { .. } => None,
    }
}

/// Returns a string list when every structured item is a string literal.
pub(super) fn string_list_value(value: &AnnotationValueIr) -> Option<Vec<String>> {
    let AnnotationValueIr::List(values) = value else {
        return None;
    };
    values
        .iter()
        .map(|value| match value {
            AnnotationValueIr::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

/// Returns a string map when every structured key and value is a string literal.
pub(super) fn string_map_value(value: &AnnotationValueIr) -> Option<Vec<(String, String)>> {
    let AnnotationValueIr::Map(entries) = value else {
        return None;
    };
    entries
        .iter()
        .map(|(key, value)| match (key, value) {
            (AnnotationValueIr::String(key), AnnotationValueIr::String(value)) => {
                Some((key.clone(), value.clone()))
            }
            _ => None,
        })
        .collect()
}

/// Returns member names when every structured list item is a member reference.
pub(super) fn type_list_value(value: &AnnotationValueIr) -> Option<Vec<String>> {
    let AnnotationValueIr::List(values) = value else {
        return None;
    };
    values
        .iter()
        .map(|value| match value {
            AnnotationValueIr::Member(name) => Some(name.source.clone()),
            _ => None,
        })
        .collect()
}
