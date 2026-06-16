use crate::{
    AnnotationIr, ConstructorParamIr, ExprSourceIr, FieldIr, MethodParamIr, NameIr, SpanIr, TypeIr,
};

/// One Dart class modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassModifierIr {
    /// `abstract`.
    Abstract,
    /// `base`.
    Base,
    /// `final`.
    Final,
    /// `interface`.
    Interface,
    /// `mixin`.
    Mixin,
    /// `sealed`.
    Sealed,
}

/// One generic type parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParamIr {
    /// The parameter name.
    pub name: NameIr,
    /// The optional upper bound.
    pub bound: Option<TypeIr>,
    /// The source span for this type parameter.
    pub span: SpanIr,
}

/// One primary-constructor declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryConstructorIr {
    /// Constructor parameters in source order.
    pub params: Vec<ConstructorParamIr>,
    /// The source span for the primary constructor parameter list.
    pub span: SpanIr,
}

/// One lowered getter declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetterIr {
    /// The getter name.
    pub name: NameIr,
    /// The declared return type.
    pub return_type: TypeIr,
    /// Metadata annotations attached to the getter.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the getter.
    pub span: SpanIr,
}

/// One lowered setter declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetterIr {
    /// The setter name.
    pub name: NameIr,
    /// The setter parameter.
    pub param: MethodParamIr,
    /// Metadata annotations attached to the setter.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the setter.
    pub span: SpanIr,
}

/// One Dart mixin declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixinIr {
    /// The mixin name.
    pub name: NameIr,
    /// Metadata annotations attached to the mixin.
    pub annotations: Vec<AnnotationIr>,
    /// Fields declared by the mixin.
    pub fields: Vec<FieldIr>,
    /// The source span for the mixin.
    pub span: SpanIr,
}

/// One Dart extension declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionIr {
    /// The optional extension name.
    pub name: Option<NameIr>,
    /// The extended type.
    pub on_type: TypeIr,
    /// Metadata annotations attached to the extension.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the extension.
    pub span: SpanIr,
}

/// One Dart extension type declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionTypeIr {
    /// The extension type name.
    pub name: NameIr,
    /// Metadata annotations attached to the extension type.
    pub annotations: Vec<AnnotationIr>,
    /// The representation field.
    pub representation: FieldIr,
    /// The source span for the extension type.
    pub span: SpanIr,
}

/// One top-level function declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionIr {
    /// The function name.
    pub name: NameIr,
    /// The function return type.
    pub return_type: TypeIr,
    /// The function parameters.
    pub params: Vec<MethodParamIr>,
    /// Metadata annotations attached to the function.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the function.
    pub span: SpanIr,
}

/// One top-level variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopLevelVariableIr {
    /// The variable name.
    pub name: NameIr,
    /// The variable type.
    pub ty: TypeIr,
    /// The optional initializer expression.
    pub initializer: Option<ExprSourceIr>,
    /// Metadata annotations attached to the variable.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the variable.
    pub span: SpanIr,
}

/// One Dart typedef declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedefIr {
    /// The typedef name.
    pub name: NameIr,
    /// The aliased type or function signature source.
    pub aliased_type: TypeIr,
    /// Metadata annotations attached to the typedef.
    pub annotations: Vec<AnnotationIr>,
    /// The source span for the typedef.
    pub span: SpanIr,
}
