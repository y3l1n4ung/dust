pub(super) const PACKAGE: &str = "dust_http_client_annotation";
pub(super) const HTTP_CLIENT: &str = "HttpClient";
pub(super) const GENERATE_TEST: &str = "GenerateTest";
pub(super) const HTTP_PARSE: &str = "HttpParse";
pub(super) const HEADERS: &str = "Headers";
pub(super) const FORM_URL_ENCODED: &str = "FormUrlEncoded";
pub(super) const MULTI_PART: &str = "MultiPart";
pub(super) const PATH: &str = "Path";
pub(super) const QUERY: &str = "Query";
pub(super) const QUERIES: &str = "Queries";
pub(super) const HEADER: &str = "Header";
pub(super) const HEADER_MAP: &str = "HeaderMap";
pub(super) const BODY: &str = "Body";
pub(super) const FIELD: &str = "Field";
pub(super) const PART: &str = "Part";
pub(super) const EXTRA: &str = "Extra";

const CLAIMED_CONFIGS: &[&str] = &[
    HTTP_CLIENT,
    GENERATE_TEST,
    "GET",
    "POST",
    "PUT",
    "PATCH",
    "DELETE",
    "HEAD",
    "OPTIONS",
    PATH,
    QUERY,
    QUERIES,
    HEADER,
    HEADERS,
    HEADER_MAP,
    BODY,
    FIELD,
    PART,
    EXTRA,
    FORM_URL_ENCODED,
    MULTI_PART,
    HTTP_PARSE,
];

pub(super) fn claimed_config_names() -> &'static [&'static str] {
    CLAIMED_CONFIGS
}
