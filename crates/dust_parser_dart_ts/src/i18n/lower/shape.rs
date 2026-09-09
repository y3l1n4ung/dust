//! The shape of one i18n call, as the lowering reads it.

use super::*;

/// Matched i18n call metadata.
pub(super) struct I18nCallShape {
    /// Recognized call kind.
    pub(super) kind: I18nCallKind,
    /// Full source span for the call.
    pub(super) span: TextRange,
}

impl I18nCallShape {
    /// Returns the public translation API kind when this is a translation call.
    pub(super) fn translation_kind(&self) -> Option<I18nTranslationKind> {
        match self.kind {
            I18nCallKind::Translation(kind) => Some(kind),
            I18nCallKind::HardcodedText => None,
        }
    }
}

/// Recognized call kinds relevant to i18n scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum I18nCallKind {
    /// A public i18n translation API call.
    Translation(I18nTranslationKind),
    /// A direct Flutter `Text("literal")` call.
    HardcodedText,
}
