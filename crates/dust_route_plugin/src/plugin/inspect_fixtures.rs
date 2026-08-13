use super::{
    inspect::route_name,
    inspect_fixture_values::{
        can_invalidate_path, can_invalidate_query, invalid_value, is_int_list, is_nullable,
        list_inner, sample_value, with_fragment,
    },
    model::{RouteFact, RouteParamFact},
};

/// One deep-link fixture row shown by route inspection tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteFixtureRow {
    /// Effective route name, or `-` for workspace-level examples.
    pub route: String,
    /// Fixture case name.
    pub case_name: String,
    /// Whether this URI should resolve to the typed route.
    pub valid: bool,
    /// Link shape, such as `path`, `web-url`, `app-link`, or `custom-scheme`.
    pub shape: String,
    /// Fixture URI text.
    pub uri: String,
    /// Expected routing outcome.
    pub expected: String,
}

/// Builds deterministic deep-link fixture rows from route facts.
pub(super) fn route_fixture_rows(facts: &[RouteFact]) -> Vec<RouteFixtureRow> {
    let mut rows = Vec::new();

    for fact in facts {
        let route = route_name(fact);
        let location = valid_location(fact);
        rows.push(fixture(
            &route,
            "path",
            true,
            "path",
            &location,
            "typed-route",
        ));
        rows.push(fixture(
            &route,
            "web-url",
            true,
            "web-url",
            &format!("https://shop.example{location}"),
            "normalize-then-typed-route",
        ));
        rows.push(fixture(
            &route,
            "app-link",
            true,
            "app-link",
            &format!("https://shop.example/app{location}"),
            "normalize-prefix-then-typed-route",
        ));
        rows.push(fixture(
            &route,
            "custom-scheme",
            true,
            "custom-scheme",
            &format!("shopping:///{}", location.trim_start_matches('/')),
            "normalize-scheme-then-typed-route",
        ));
        rows.push(fixture(
            &route,
            "fragment-preserved",
            true,
            "path",
            &with_fragment(&location, "details"),
            "typed-route-preserve-fragment",
        ));

        rows.extend(invalid_path_param_fixtures(fact, &route));
        rows.extend(invalid_query_param_fixtures(fact, &route));
    }

    if !facts.is_empty() {
        rows.push(fixture(
            "-",
            "not-found",
            false,
            "path",
            "/__dust_missing_route__",
            "not-found-route",
        ));
        rows.push(fixture(
            "-",
            "malformed-fragment-escape",
            false,
            "path",
            "/#%ZZ",
            "uri-parse-error-before-router",
        ));
    }

    rows
}

/// Creates one deep-link fixture row.
fn fixture(
    route: &str,
    case_name: &str,
    valid: bool,
    shape: &str,
    uri: &str,
    expected: &str,
) -> RouteFixtureRow {
    RouteFixtureRow {
        route: route.to_owned(),
        case_name: case_name.to_owned(),
        valid,
        shape: shape.to_owned(),
        uri: uri.to_owned(),
        expected: expected.to_owned(),
    }
}

/// Builds the canonical valid app-relative location for one route.
fn valid_location(fact: &RouteFact) -> String {
    let path = fill_path_params(fact, false);
    let query = valid_query(fact);
    if query.is_empty() {
        path
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

/// Fills path placeholders with deterministic typed sample values.
fn fill_path_params(fact: &RouteFact, invalid_first_typed_param: bool) -> String {
    let mut invalid_used = false;
    fact.path
        .split('/')
        .map(|segment| {
            let Some(name) = segment.strip_prefix(':') else {
                return segment.to_owned();
            };
            let param = fact.params.iter().find(|param| param.name == name);
            if invalid_first_typed_param && !invalid_used && param.is_some_and(can_invalidate_path)
            {
                invalid_used = true;
                return invalid_value(param.and_then(|param| param.type_source.as_deref()));
            }
            param
                .map(|param| sample_value(&param.name, param.type_source.as_deref()))
                .unwrap_or_else(|| sample_value(name, None))
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Builds valid query pairs for constructor params that are not path params.
fn valid_query(fact: &RouteFact) -> Vec<String> {
    query_params(fact)
        .iter()
        .flat_map(|param| query_pairs(param, false))
        .collect()
}

/// Builds invalid path parameter fixtures for typed path params.
fn invalid_path_param_fixtures(fact: &RouteFact, route: &str) -> Vec<RouteFixtureRow> {
    if !fact
        .params
        .iter()
        .filter(|param| path_param_names(&fact.path).contains(&param.name))
        .any(can_invalidate_path)
    {
        return Vec::new();
    }

    let path = fill_path_params(fact, true);
    let query = valid_query(fact);
    let uri = if query.is_empty() {
        path
    } else {
        format!("{path}?{}", query.join("&"))
    };
    vec![fixture(
        route,
        "invalid-path-param",
        false,
        "path",
        &uri,
        "not-found-route",
    )]
}

/// Builds invalid query parameter fixtures for required or typed query params.
fn invalid_query_param_fixtures(fact: &RouteFact, route: &str) -> Vec<RouteFixtureRow> {
    let params = query_params(fact);
    let mut rows = Vec::new();
    for param in params {
        if is_required_query(param) {
            let uri = location_without_query_param(fact, &param.name);
            rows.push(fixture(
                route,
                &format!("missing-query-{}", param.name),
                false,
                "path",
                &uri,
                "not-found-route",
            ));
        }
        if can_invalidate_query(param) {
            let uri = location_with_invalid_query_param(fact, &param.name);
            rows.push(fixture(
                route,
                &format!("invalid-query-{}", param.name),
                false,
                "path",
                &uri,
                "not-found-route",
            ));
        }
    }
    rows
}

/// Returns constructor params encoded as query parameters.
fn query_params(fact: &RouteFact) -> Vec<&RouteParamFact> {
    let path_names = path_param_names(&fact.path);
    fact.params
        .iter()
        .filter(|param| !path_names.contains(&param.name))
        .collect()
}

/// Returns route path placeholder names.
fn path_param_names(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns true when a query parameter must be present in valid URLs.
fn is_required_query(param: &RouteParamFact) -> bool {
    !is_nullable(param.type_source.as_deref()) && !param.has_default
}

/// Returns a valid location with one query parameter removed.
fn location_without_query_param(fact: &RouteFact, omitted: &str) -> String {
    let path = fill_path_params(fact, false);
    let query = query_params(fact)
        .iter()
        .filter(|param| param.name != omitted)
        .flat_map(|param| query_pairs(param, false))
        .collect::<Vec<_>>();
    if query.is_empty() {
        path
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

/// Returns a valid location with one query parameter replaced by an invalid value.
fn location_with_invalid_query_param(fact: &RouteFact, invalid: &str) -> String {
    let path = fill_path_params(fact, false);
    let query = query_params(fact)
        .iter()
        .flat_map(|param| query_pairs(param, param.name == invalid))
        .collect::<Vec<_>>();
    if query.is_empty() {
        path
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

/// Returns query pairs for a parameter.
fn query_pairs(param: &RouteParamFact, invalid: bool) -> Vec<String> {
    let type_source = param.type_source.as_deref();
    if list_inner(type_source).is_some() {
        let values = if invalid {
            vec![invalid_value(type_source)]
        } else if is_int_list(type_source) {
            vec!["1".to_owned(), "2".to_owned()]
        } else {
            vec!["sale".to_owned(), "new".to_owned()]
        };
        return values
            .into_iter()
            .map(|value| format!("{}={value}", param.name))
            .collect();
    }

    let value = if invalid {
        invalid_value(type_source)
    } else {
        sample_value(&param.name, type_source)
    };
    vec![format!("{}={value}", param.name)]
}
