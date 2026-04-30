use crate::{SerdeClassConfigIr, SpanIr, TraitApplicationIr};

/// One lowered enum declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumIr {
    /// The enum name.
    pub name: String,
    /// The source span of the enum.
    pub span: SpanIr,
    /// The lowered variants in declaration order.
    pub variants: Vec<EnumVariantIr>,
    /// Trait applications resolved for this enum.
    pub traits: Vec<TraitApplicationIr>,
    /// Normalized serde-related configuration (reusing class config for now).
    pub serde: Option<SerdeClassConfigIr>,
}

/// One lowered enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantIr {
    /// The variant name.
    pub name: String,
    /// The source span for the variant.
    pub span: SpanIr,
}
