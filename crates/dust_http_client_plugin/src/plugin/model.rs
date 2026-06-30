use dust_ir::{MethodIr, MethodParamIr, TypeIr};

/// Controls where JSON response parsing runs for a generated endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParseThreadMode {
    /// Parse the response on the caller's isolate.
    Main,
    /// Parse the response through a generated `Isolate.run` helper.
    Isolate,
}

/// Selects the runtime package surface required by a generated HTTP client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HttpTargetMode {
    /// Generate code for a Dart-only package.
    Dart,
    /// Generate code for a Flutter package.
    Flutter,
}

/// HTTP verb emitted into the generated Dio request options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HttpVerb {
    /// HTTP GET.
    Get,
    /// HTTP POST.
    Post,
    /// HTTP PUT.
    Put,
    /// HTTP PATCH.
    Patch,
    /// HTTP DELETE.
    Delete,
    /// HTTP HEAD.
    Head,
    /// HTTP OPTIONS.
    Options,
}

impl HttpVerb {
    /// Returns the uppercase method string expected by Dio.
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

/// Request body encoding selected by method-level annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum RequestMode {
    /// Send either the `@Body` value or no body.
    #[default]
    Standard,
    /// Send `@Field` parameters as a form-url-encoded map.
    FormUrlEncoded,
    /// Send `@Part` parameters through `FormData.fromMap`.
    MultiPart,
}

/// Validated HTTP client class ready for Dart emission.
#[derive(Debug, Clone)]
pub(super) struct ClientSpec<'a> {
    /// Dart class name for the generated client implementation.
    pub(super) class_name: &'a str,
    /// Optional class-level base URL from `@HttpClient`.
    pub(super) base_url: Option<String>,
    /// Runtime target selected by `@HttpClient(target: ...)`.
    pub(super) target: HttpTargetMode,
    /// Methods that Dust can lower into generated endpoints.
    pub(super) endpoints: Vec<EndpointSpec<'a>>,
}

/// Validated HTTP endpoint method ready for Dart emission.
#[derive(Debug, Clone)]
pub(super) struct EndpointSpec<'a> {
    /// Original method IR used for names, parameters, and return type rendering.
    pub(super) method: &'a MethodIr,
    /// HTTP verb selected by method annotation.
    pub(super) verb: HttpVerb,
    /// Relative path template, including `{placeholder}` segments.
    pub(super) path: String,
    /// Parse-thread mode after method overrides are applied.
    pub(super) parse_thread: ParseThreadMode,
    /// Static headers inherited from the class and method annotations.
    pub(super) headers: Vec<(String, String)>,
    /// Request body encoding mode for this endpoint.
    pub(super) request_mode: RequestMode,
    /// Bound endpoint parameters in source order.
    pub(super) params: Vec<EndpointParam<'a>>,
    /// Return-shape metadata used for fetch and decode rendering.
    pub(super) return_spec: ReturnSpec,
}

/// Binding between a method parameter and its HTTP request role.
#[derive(Debug, Clone)]
pub(super) enum EndpointParam<'a> {
    /// Parameter substituted into a URI path placeholder.
    Path {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Placeholder key in the path template.
        key: String,
    },
    /// Parameter rendered as a single query parameter.
    Query {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Query-string key.
        key: String,
    },
    /// Parameter merged into the query parameter map.
    Queries {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Parameter rendered as a single request header.
    Header {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Header name.
        key: String,
    },
    /// Parameter merged into the request header map.
    HeaderMap {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Parameter rendered as the raw request body.
    Body {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Parameter rendered into a form-url-encoded body.
    Field {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Form field name.
        key: String,
    },
    /// Parameter rendered into multipart form data.
    Part {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Multipart field name.
        key: String,
    },
    /// Parameter rendered into Dio `extra` metadata.
    Extra {
        /// Original method parameter.
        param: &'a MethodParamIr,
        /// Extra map key.
        key: String,
    },
    /// Dio cancellation token parameter.
    CancelToken {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Dio request options parameter.
    Options {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Dio upload progress callback parameter.
    OnSendProgress {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
    /// Dio download progress callback parameter.
    OnReceiveProgress {
        /// Original method parameter.
        param: &'a MethodParamIr,
    },
}

/// Return metadata after validating supported Future, Response, and Stream shapes.
#[derive(Debug, Clone)]
pub(super) struct ReturnSpec {
    /// Whether the endpoint returns a unary value or a stream.
    pub(super) mode: ReturnMode,
    /// Whether the generated method should wrap decoded data in `Response<T>`.
    pub(super) raw_response: bool,
    /// Payload type decoded from the Dio response body.
    pub(super) ty: TypeIr,
}

/// Supported generated endpoint return execution modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReturnMode {
    /// A single Future-based response.
    Unary,
    /// A byte stream response represented by `Stream<List<int>>`.
    ByteStream,
    /// A text stream response represented by `Stream<String>`.
    TextStream,
}

impl ReturnSpec {
    /// Returns true when the endpoint response is streamed.
    pub(super) fn is_stream(&self) -> bool {
        self.mode != ReturnMode::Unary
    }
}

impl<'a> EndpointSpec<'a> {
    /// Finds the HTTP binding for a method parameter by source name.
    pub(super) fn binding_for_param(&self, name: &str) -> Option<&EndpointParam<'a>> {
        self.params
            .iter()
            .find(|binding| binding.param().name == name)
    }
}

impl<'a> EndpointParam<'a> {
    /// Returns the original method parameter behind any endpoint binding.
    pub(super) fn param(&self) -> &'a MethodParamIr {
        match self {
            Self::Path { param, .. }
            | Self::Query { param, .. }
            | Self::Queries { param }
            | Self::Header { param, .. }
            | Self::HeaderMap { param }
            | Self::Body { param }
            | Self::Field { param, .. }
            | Self::Part { param, .. }
            | Self::Extra { param, .. }
            | Self::CancelToken { param }
            | Self::Options { param }
            | Self::OnSendProgress { param }
            | Self::OnReceiveProgress { param } => param,
        }
    }
}
