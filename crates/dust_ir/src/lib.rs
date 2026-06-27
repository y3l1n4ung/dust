#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Semantic intermediate representation for Dust."]

/// Annotation metadata in lowered form.
mod annotation;
/// Class declarations.
mod class;
/// Constructor declarations.
mod constructor;
/// Non-class declaration types.
mod declaration;
/// Dart library directives.
mod directive;
/// Enum declarations.
mod enum_type;
/// Class and enum fields.
mod field;
/// Library and file containers.
mod library;
/// Lowering output containers.
mod lowering;
/// Method declarations.
mod method;
/// Query function metadata.
mod query_call;
/// Serialization configuration.
mod serde;
/// Applied plugin traits and configs.
mod traits;
/// Type metadata.
mod types;
/// Workspace-level IR.
mod workspace;

pub use annotation::{
    AnnotationIr, AnnotationNumberKindIr, AnnotationValueIr, ExprSourceIr, NameIr,
};
pub use class::{ClassIr, ClassKindIr};
pub use constructor::{ConstructorIr, ConstructorParamIr, ParamKind};
pub use declaration::{
    ClassModifierIr, ExtensionIr, ExtensionTypeIr, FunctionIr, GetterIr, MixinIr,
    PrimaryConstructorIr, SetterIr, TopLevelVariableIr, TypeParamIr, TypedefIr,
};
pub use directive::{ExportIr, ImportIr, LibraryDeclIr, PartIr, PartOfIr};
pub use enum_type::{EnumIr, EnumVariantIr};
pub use field::FieldIr;
pub use library::{DartFileIr, LibraryIr, SpanIr};
pub use lowering::LoweringOutcome;
pub use method::{MethodIr, MethodParamIr};
pub use query_call::{QueryCallIr, QueryFunctionIr};
pub use serde::{SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, SerdeVariantConfigIr};
pub use traits::{ConfigApplicationIr, SymbolId, TraitApplicationIr};
pub use types::{BuiltinType, TypeIr};
pub use workspace::WorkspaceIr;
