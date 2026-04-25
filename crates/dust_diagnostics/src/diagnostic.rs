use crate::{Severity, SourceLabel};

/// A structured diagnostic emitted by one Dust pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The severity level of this diagnostic.
    pub severity: Severity,
    /// The main human-readable message.
    pub message: String,
    /// Source labels attached to the diagnostic.
    pub labels: Vec<SourceLabel>,
    /// Additional notes shown after the main message.
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Creates a diagnostic with the given severity and message.
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Creates an error diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    /// Creates a warning diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    /// Creates a note diagnostic.
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    /// Attaches one source label and returns the updated diagnostic.
    pub fn with_label(mut self, label: SourceLabel) -> Self {
        self.labels.push(label);
        self
    }

    /// Attaches one note and returns the updated diagnostic.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Returns `true` if the diagnostic contains at least one source label.
    pub fn has_labels(&self) -> bool {
        !self.labels.is_empty()
    }

    /// Returns `true` if this diagnostic is an error.
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }
}
