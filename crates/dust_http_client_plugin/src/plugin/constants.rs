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

pub(super) const CLAIMED_CONFIG_SYMBOLS: &[&str] = &[
    "dust_http_client_annotation::HttpClient",
    "dust_http_client_annotation::GenerateTest",
    "dust_http_client_annotation::GET",
    "dust_http_client_annotation::POST",
    "dust_http_client_annotation::PUT",
    "dust_http_client_annotation::PATCH",
    "dust_http_client_annotation::DELETE",
    "dust_http_client_annotation::HEAD",
    "dust_http_client_annotation::OPTIONS",
    "dust_http_client_annotation::Path",
    "dust_http_client_annotation::Query",
    "dust_http_client_annotation::Queries",
    "dust_http_client_annotation::Header",
    "dust_http_client_annotation::Headers",
    "dust_http_client_annotation::HeaderMap",
    "dust_http_client_annotation::Body",
    "dust_http_client_annotation::Field",
    "dust_http_client_annotation::Part",
    "dust_http_client_annotation::Extra",
    "dust_http_client_annotation::FormUrlEncoded",
    "dust_http_client_annotation::MultiPart",
    "dust_http_client_annotation::HttpParse",
];

pub(super) const SUPPORTED_ANNOTATIONS: &[&str] = &[
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
