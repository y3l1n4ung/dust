use crate::{ConfigApplicationIr, ParamKind, SpanIr, TraitApplicationIr, TypeIr};

/// One lowered method on a class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodIr {
    /// The method name.
    pub name: String,
    /// Whether the method is marked `static`.
    pub is_static: bool,
    /// Whether the method is marked `external`.
    pub is_external: bool,
    /// The normalized return type.
    pub return_type: TypeIr,
    /// Whether the method includes an implementation body.
    pub has_body: bool,
    /// The lowered parameters in declaration order.
    pub params: Vec<MethodParamIr>,
    /// The source span for the method.
    pub span: SpanIr,
    /// Trait applications resolved for this method.
    pub traits: Vec<TraitApplicationIr>,
    /// Config applications resolved for this method.
    pub configs: Vec<ConfigApplicationIr>,
}

/// One lowered parameter on a method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodParamIr {
    /// The parameter name.
    pub name: String,
    /// The normalized parameter type.
    pub ty: TypeIr,
    /// The source span for the parameter.
    pub span: SpanIr,
    /// The parameter kind.
    pub kind: ParamKind,
    /// Whether the parameter has a default value or initializer.
    pub has_default: bool,
    /// Trait applications resolved for this parameter.
    pub traits: Vec<TraitApplicationIr>,
    /// Config applications resolved for this parameter.
    pub configs: Vec<ConfigApplicationIr>,
}
