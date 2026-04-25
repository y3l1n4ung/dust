use crate::{SerdeFieldConfigIr, SpanIr, TypeIr};

/// One lowered field on a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldIr {
    /// The field name.
    pub name: String,
    /// The normalized field type.
    pub ty: TypeIr,
    /// The source span for the field.
    pub span: SpanIr,
    /// Whether the source field has a default value or initializer.
    pub has_default: bool,
    /// Normalized serde-related field configuration.
    pub serde: Option<SerdeFieldConfigIr>,
}
