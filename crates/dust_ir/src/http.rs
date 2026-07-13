/// Resolver-normalized HTTP configuration attached to a class, method, or
/// parameter annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpConfigIr {
    /// Options from a class-level `HttpClient` annotation.
    Client(HttpClientConfigIr),
    /// An HTTP method annotation and its relative path.
    Verb {
        /// HTTP verb name as written by the annotation.
        verb: HttpVerbIr,
        /// Relative endpoint path template.
        path: String,
    },
    /// A method-level response parsing thread override.
    Parse(HttpParseConfigIr),
    /// Static headers supplied by a `Headers` annotation.
    Headers(Vec<(String, String)>),
    /// Request body encoding mode supplied by `FormUrlEncoded` or `MultiPart`.
    RequestMode(HttpRequestModeIr),
    /// A parameter binding annotation.
    Parameter(HttpParameterConfigIr),
}

/// Options from a class-level `HttpClient` annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientConfigIr {
    /// Optional default base URL.
    pub base_url: Option<String>,
    /// Runtime target used by generated helpers.
    pub target: HttpTargetIr,
    /// Default response parsing thread.
    pub parse_thread: HttpParseThreadIr,
    /// Static headers inherited by every endpoint.
    pub headers: Vec<(String, String)>,
    /// Whether an auxiliary request fixture should be generated.
    pub generate_test: bool,
}

/// HTTP runtime target selected by `HttpClient(target: ...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpTargetIr {
    /// Dart-only runtime.
    Dart,
    /// Flutter runtime.
    Flutter,
}

/// Response parsing execution thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpParseThreadIr {
    /// Parse on the caller's isolate.
    Main,
    /// Parse through a generated background helper.
    Isolate,
}

/// Method-level response parsing override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpParseConfigIr {
    /// Selected parsing thread.
    pub thread: HttpParseThreadIr,
}

/// Supported HTTP method names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVerbIr {
    /// GET request.
    Get,
    /// POST request.
    Post,
    /// PUT request.
    Put,
    /// PATCH request.
    Patch,
    /// DELETE request.
    Delete,
    /// HEAD request.
    Head,
    /// OPTIONS request.
    Options,
}

/// Request body encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRequestModeIr {
    /// Standard body or no body.
    Standard,
    /// Form URL encoded fields.
    FormUrlEncoded,
    /// Multipart form data.
    MultiPart,
}

/// Parameter-level HTTP binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpParameterConfigIr {
    /// URI path placeholder binding.
    Path(Option<String>),
    /// Single query parameter binding.
    Query(String),
    /// Merged query map binding.
    Queries,
    /// Single request header binding.
    Header(String),
    /// Merged request header map binding.
    HeaderMap,
    /// Raw request body binding.
    Body,
    /// Form field binding.
    Field(String),
    /// Multipart field binding.
    Part(String),
    /// Dio extra metadata binding.
    Extra(String),
}
