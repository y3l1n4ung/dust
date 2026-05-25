use dust_ir::{ClassIr, FieldIr, MethodIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SqlxRenameRule {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SqlxConfig {
    pub(crate) rename: Option<String>,
    pub(crate) rename_all: Option<SqlxRenameRule>,
    pub(crate) flatten: bool,
    pub(crate) default_value_source: Option<String>,
    pub(crate) skip: bool,
    pub(crate) json: bool,
    pub(crate) try_from_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FetchKind {
    One,
    All,
    Scalar,
    InsertOne,
    Execute,
    Stream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuerySpec<'a> {
    pub(crate) method: &'a MethodIr,
    pub(crate) sql: String,
    pub(crate) fetch: FetchKind,
    pub(crate) args: Vec<String>,
    pub(crate) transaction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbClass<'a> {
    pub(crate) class: &'a ClassIr,
    pub(crate) migrations: String,
    pub(crate) queries: Vec<QuerySpec<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowClass<'a> {
    pub(crate) class: &'a ClassIr,
    pub(crate) config: SqlxConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowField<'a> {
    pub(crate) field: &'a FieldIr,
    pub(crate) config: SqlxConfig,
    pub(crate) column: String,
}
