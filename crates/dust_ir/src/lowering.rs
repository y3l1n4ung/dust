use dust_diagnostics::Diagnostic;

/// A lowering result that carries a value plus non-fatal diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringOutcome<T> {
    /// The lowered value.
    pub value: T,
    /// Diagnostics emitted during lowering.
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> LoweringOutcome<T> {
    /// Creates a lowering outcome with no diagnostics.
    pub fn new(value: T) -> Self {
        Self {
            value,
            diagnostics: Vec::new(),
        }
    }

    /// Appends one diagnostic and returns the updated outcome.
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    /// Maps the wrapped value while preserving diagnostics.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LoweringOutcome<U> {
        LoweringOutcome {
            value: f(self.value),
            diagnostics: self.diagnostics,
        }
    }
}
