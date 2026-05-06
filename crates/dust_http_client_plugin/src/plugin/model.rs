use dust_ir::{MethodIr, MethodParamIr, TypeIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParseThreadMode {
    Main,
    Isolate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HttpTargetMode {
    Dart,
    Flutter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HttpVerb {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpVerb {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum RequestMode {
    #[default]
    Standard,
    FormUrlEncoded,
    MultiPart,
}

#[derive(Debug, Clone)]
pub(super) struct ClientSpec<'a> {
    pub(super) class_name: &'a str,
    pub(super) base_url: Option<String>,
    pub(super) endpoints: Vec<EndpointSpec<'a>>,
}

#[derive(Debug, Clone)]
pub(super) struct EndpointSpec<'a> {
    pub(super) method: &'a MethodIr,
    pub(super) verb: HttpVerb,
    pub(super) path: String,
    pub(super) parse_thread: ParseThreadMode,
    pub(super) headers: Vec<(String, String)>,
    pub(super) request_mode: RequestMode,
    pub(super) params: Vec<EndpointParam<'a>>,
    pub(super) return_spec: ReturnSpec,
}

#[derive(Debug, Clone)]
pub(super) enum EndpointParam<'a> {
    Path {
        param: &'a MethodParamIr,
        key: String,
    },
    Query {
        param: &'a MethodParamIr,
        key: String,
    },
    Queries {
        param: &'a MethodParamIr,
    },
    Header {
        param: &'a MethodParamIr,
        key: String,
    },
    HeaderMap {
        param: &'a MethodParamIr,
    },
    Body {
        param: &'a MethodParamIr,
    },
    Field {
        param: &'a MethodParamIr,
        key: String,
    },
    Part {
        param: &'a MethodParamIr,
        key: String,
    },
    Extra {
        param: &'a MethodParamIr,
        key: String,
    },
    CancelToken {
        param: &'a MethodParamIr,
    },
    Options {
        param: &'a MethodParamIr,
    },
    OnSendProgress {
        param: &'a MethodParamIr,
    },
    OnReceiveProgress {
        param: &'a MethodParamIr,
    },
}

#[derive(Debug, Clone)]
pub(super) struct ReturnSpec {
    pub(super) mode: ReturnMode,
    pub(super) raw_response: bool,
    pub(super) ty: TypeIr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReturnMode {
    Unary,
    ByteStream,
    TextStream,
}

impl ReturnSpec {
    pub(super) fn is_stream(&self) -> bool {
        self.mode != ReturnMode::Unary
    }
}

impl<'a> EndpointSpec<'a> {
    pub(super) fn binding_for_param(&self, name: &str) -> Option<&EndpointParam<'a>> {
        self.params
            .iter()
            .find(|binding| binding.param().name == name)
    }
}

impl<'a> EndpointParam<'a> {
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
