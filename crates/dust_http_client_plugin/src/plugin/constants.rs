/// Class-level annotation that enables HTTP client generation.
pub(super) const HTTP_CLIENT: &str = "HttpClient";
/// Class-level annotation that requests generated test fixtures.
pub(super) const GENERATE_TEST: &str = "GenerateTest";
/// Method-level annotation that opts response parsing into isolate execution.
pub(super) const HTTP_PARSE: &str = "HttpParse";
/// Annotation that provides static HTTP headers.
pub(super) const HEADERS: &str = "Headers";
/// Annotation that marks request data as form-url-encoded.
pub(super) const FORM_URL_ENCODED: &str = "FormUrlEncoded";
/// Annotation that marks request data as multipart form data.
pub(super) const MULTI_PART: &str = "MultiPart";
/// Parameter annotation for URI path placeholders.
pub(super) const PATH: &str = "Path";
/// Parameter annotation for a single query entry.
pub(super) const QUERY: &str = "Query";
/// Parameter annotation for merging a query map.
pub(super) const QUERIES: &str = "Queries";
/// Parameter annotation for a single HTTP header.
pub(super) const HEADER: &str = "Header";
/// Parameter annotation for merging a header map.
pub(super) const HEADER_MAP: &str = "HeaderMap";
/// Parameter annotation for the request body value.
pub(super) const BODY: &str = "Body";
/// Parameter annotation for a form-url-encoded field.
pub(super) const FIELD: &str = "Field";
/// Parameter annotation for a multipart field.
pub(super) const PART: &str = "Part";
/// Parameter annotation for Dio request extras.
pub(super) const EXTRA: &str = "Extra";

/// Fully qualified Dust symbols claimed by the HTTP plugin.
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

/// Short annotation names supported by the HTTP plugin.
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
