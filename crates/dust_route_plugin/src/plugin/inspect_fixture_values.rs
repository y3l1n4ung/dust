use super::model::RouteParamFact;

/// Returns true when a path parameter can fail typed decoding.
pub(super) fn can_invalidate_path(param: &RouteParamFact) -> bool {
    !matches!(
        base_type(param.type_source.as_deref()).as_deref(),
        Some("String")
    )
}

/// Returns true when a query parameter can fail typed decoding.
pub(super) fn can_invalidate_query(param: &RouteParamFact) -> bool {
    !matches!(
        base_type(param.type_source.as_deref()).as_deref(),
        Some("String")
    ) && !is_string_list(param.type_source.as_deref())
}

/// Returns a deterministic valid sample value for one supported URL type.
pub(super) fn sample_value(name: &str, source: Option<&str>) -> String {
    match base_type(source).as_deref() {
        Some("int") => "42".to_owned(),
        Some("double") => "19.99".to_owned(),
        Some("bool") => "true".to_owned(),
        Some("DateTime") => "2026-08-10T09%3A30%3A00.000Z".to_owned(),
        Some("Uri") => "https%3A%2F%2Fexample.com%2Freceipt".to_owned(),
        Some("String") | None => sample_string(name),
        Some(_) => "active".to_owned(),
    }
}

/// Returns a deterministic invalid sample value for one URL type.
pub(super) fn invalid_value(source: Option<&str>) -> String {
    match base_type(source).as_deref() {
        Some("int") => "not-an-int".to_owned(),
        Some("double") => "not-a-double".to_owned(),
        Some("bool") => "not-a-bool".to_owned(),
        Some("DateTime") => "not-a-date".to_owned(),
        Some("Uri") => "%ZZ".to_owned(),
        Some(_) => "not-a-valid-value".to_owned(),
        None => "invalid".to_owned(),
    }
}

/// Returns true for nullable type source.
pub(super) fn is_nullable(source: Option<&str>) -> bool {
    source.is_some_and(|source| source.trim().ends_with('?'))
}

/// Returns true for `List<int>`.
pub(super) fn is_int_list(source: Option<&str>) -> bool {
    matches!(list_inner(source), Some("int"))
}

/// Appends or replaces a URI fragment.
pub(super) fn with_fragment(location: &str, fragment: &str) -> String {
    let without_fragment = location.split_once('#').map_or(location, |(base, _)| base);
    format!("{without_fragment}#{fragment}")
}

/// Returns a stable name-based sample for string-like values.
fn sample_string(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("order") {
        "ORDER-1001".to_owned()
    } else if lower.contains("product") {
        "SKU-1001".to_owned()
    } else if lower.contains("tab") {
        "reviews".to_owned()
    } else {
        format!("{name}-sample")
    }
}

/// Returns the non-nullable base type name from a Dart type source.
fn base_type(source: Option<&str>) -> Option<String> {
    let raw = source?.trim().trim_end_matches('?').trim();
    let list_inner = list_inner(Some(raw));
    let base = list_inner.unwrap_or(raw).trim();
    Some(base.split('<').next().unwrap_or(base).trim().to_owned())
}

/// Returns the inner source for `List<T>`.
pub(super) fn list_inner(source: Option<&str>) -> Option<&str> {
    let raw = source?.trim().trim_end_matches('?').trim();
    raw.strip_prefix("List<")?.strip_suffix('>').map(str::trim)
}

/// Returns true for `List<String>`.
fn is_string_list(source: Option<&str>) -> bool {
    matches!(list_inner(source), Some("String"))
}
