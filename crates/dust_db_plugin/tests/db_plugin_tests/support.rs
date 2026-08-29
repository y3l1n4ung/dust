#[path = "support/fixtures.rs"]
mod fixtures;
#[path = "support/ir.rs"]
mod ir;
#[path = "support/package.rs"]
mod package;
#[path = "support/project.rs"]
mod project;
#[path = "support/query.rs"]
mod query;

pub(crate) use fixtures::*;
pub(crate) use ir::*;
pub(crate) use package::*;
pub(crate) use project::*;
pub(crate) use query::*;
