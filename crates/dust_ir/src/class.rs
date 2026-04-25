use crate::{ConstructorIr, FieldIr, SerdeClassConfigIr, SpanIr, TraitApplicationIr};

/// The lowered declaration kind of a Dart class-like target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassKindIr {
    /// A normal `class` declaration.
    Class,
    /// A `mixin class` declaration.
    MixinClass,
}

/// One lowered class declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassIr {
    /// The declaration kind.
    pub kind: ClassKindIr,
    /// The class name.
    pub name: String,
    /// Whether the declaration is marked `abstract`.
    pub is_abstract: bool,
    /// The immediate superclass name, if one was declared.
    pub superclass_name: Option<String>,
    /// The source span for the class.
    pub span: SpanIr,
    /// The lowered fields in declaration order.
    pub fields: Vec<FieldIr>,
    /// The lowered constructors in declaration order.
    pub constructors: Vec<ConstructorIr>,
    /// Trait applications resolved for this class.
    pub traits: Vec<TraitApplicationIr>,
    /// Normalized serde-related class configuration.
    pub serde: Option<SerdeClassConfigIr>,
}
