/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// A hard failure that should stop successful generation.
    Error,
    /// A non-fatal problem that should still be shown to the user.
    Warning,
    /// Extra explanatory information attached to a result.
    Note,
}

impl Severity {
    /// Returns the stable lowercase display form used in rendered output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }

    /// Returns `true` if this severity represents a hard error.
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }
}
