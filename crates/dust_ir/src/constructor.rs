use crate::{FieldIr, SpanIr, TypeIr};

/// The parameter style used by a lowered constructor parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamKind {
    /// A positional parameter.
    Positional,
    /// A named parameter.
    Named,
}

/// One lowered constructor parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructorParamIr {
    /// The parameter name.
    pub name: String,
    /// The normalized parameter type.
    pub ty: TypeIr,
    /// The source span for the parameter.
    pub span: SpanIr,
    /// The parameter style.
    pub kind: ParamKind,
    /// Whether the parameter has a default value.
    pub has_default: bool,
}

/// One lowered constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructorIr {
    /// The named constructor suffix, if present.
    pub name: Option<String>,
    /// The source span for the constructor.
    pub span: SpanIr,
    /// The lowered constructor parameters.
    pub params: Vec<ConstructorParamIr>,
}

impl ConstructorIr {
    /// Returns `true` if this constructor can initialize every field.
    ///
    /// The current rule is intentionally simple:
    /// a field is considered constructible if it either has a default value
    /// or there is a parameter with the same name.
    pub fn can_construct_all_fields(&self, fields: &[FieldIr]) -> bool {
        fields.iter().all(|field| {
            field.has_default || self.params.iter().any(|param| param.name == field.name)
        })
    }
}
