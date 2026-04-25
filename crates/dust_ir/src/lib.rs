#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Semantic intermediate representation for Dust."]

mod class;
mod constructor;
mod field;
mod library;
mod lowering;
mod serde;
mod traits;
mod types;
mod workspace;

pub use class::{ClassIr, ClassKindIr};
pub use constructor::{ConstructorIr, ConstructorParamIr, ParamKind};
pub use field::FieldIr;
pub use library::{LibraryIr, SpanIr};
pub use lowering::LoweringOutcome;
pub use serde::{SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr};
pub use traits::{ConfigApplicationIr, SymbolId, TraitApplicationIr};
pub use types::{BuiltinType, TypeIr};
pub use workspace::WorkspaceIr;
