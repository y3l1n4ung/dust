use dust_text::TextRange;

/// One supported Dust DB query helper call found in a Dart library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQueryCallSurface {
    /// The query helper function name.
    pub function: ParsedQueryFunction,
    /// The raw generic type argument source, if one was provided.
    pub type_arg_source: Option<String>,
    /// The parsed SQL text for static string literals.
    pub sql: String,
    /// Whether SQL came from a static string literal accepted by Dust.
    pub sql_source_static: bool,
    /// Number of parameters in the literal list argument.
    pub parameter_count: usize,
    /// Whether the parameter argument is a list literal.
    pub params_source_is_list: bool,
    /// The terminal fetch method name, if the call is chained.
    pub fetch_method: Option<String>,
    /// The source span for the query helper invocation.
    pub span: TextRange,
}

/// Dust DB query helper function shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedQueryFunction {
    /// `queryAs<T>(...)`.
    As,
    /// `queryScalar<T>(...)`.
    Scalar,
    /// `queryRaw(...)`.
    Raw,
    /// `queryExecute(...)`.
    Execute,
}
