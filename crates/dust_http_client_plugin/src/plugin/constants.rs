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
    "dust_dart::HttpClient",
    "dust_dart::GenerateTest",
    "dust_dart::GET",
    "dust_dart::POST",
    "dust_dart::PUT",
    "dust_dart::PATCH",
    "dust_dart::DELETE",
    "dust_dart::HEAD",
    "dust_dart::OPTIONS",
    "dust_dart::Path",
    "dust_dart::Query",
    "dust_dart::Queries",
    "dust_dart::Header",
    "dust_dart::Headers",
    "dust_dart::HeaderMap",
    "dust_dart::Body",
    "dust_dart::Field",
    "dust_dart::Part",
    "dust_dart::Extra",
    "dust_dart::FormUrlEncoded",
    "dust_dart::MultiPart",
    "dust_dart::HttpParse",
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
