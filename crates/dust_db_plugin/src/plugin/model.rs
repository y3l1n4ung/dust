use dust_ir::{ClassIr, FieldIr, MethodIr, SpanIr, TypeIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbDriver {
    Sqlite3,
    Postgres,
}

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
pub(crate) enum QueryFunction {
    As,
    Scalar,
    Raw,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FetchMode {
    One,
    Optional,
    All,
    Raw,
    Execute,
}

impl FetchMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Optional => "optional",
            Self::All => "all",
            Self::Raw => "raw",
            Self::Execute => "execute",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuerySpec {
    pub(crate) function: QueryFunction,
    pub(crate) fetch: FetchMode,
    pub(crate) sql: String,
    pub(crate) sql_source_static: bool,
    pub(crate) row_type: Option<String>,
    pub(crate) scalar_type: Option<TypeIr>,
    pub(crate) parameter_count: usize,
    pub(crate) params_source_is_list: bool,
    pub(crate) span: SpanIr,
    pub(crate) display_name: Option<String>,
}

impl QuerySpec {
    pub(crate) fn display_name(&self) -> String {
        if let Some(display_name) = &self.display_name {
            return display_name.clone();
        }
        match self.function {
            QueryFunction::As => {
                format!("queryAs<{}>", self.row_type.as_deref().unwrap_or("dynamic"))
            }
            QueryFunction::Scalar => format!(
                "queryScalar<{}>",
                self.scalar_type
                    .as_ref()
                    .and_then(TypeIr::name)
                    .unwrap_or("dynamic")
            ),
            QueryFunction::Raw => "queryRaw".to_owned(),
            QueryFunction::Execute => "queryExecute".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatabaseClass<'a> {
    pub(crate) class: &'a ClassIr,
    pub(crate) driver: DbDriver,
    pub(crate) migrations: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaoClass<'a> {
    pub(crate) class: &'a ClassIr,
    pub(crate) methods: Vec<DaoMethod<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaoMethod<'a> {
    pub(crate) method: &'a MethodIr,
    pub(crate) sql: String,
    pub(crate) sql_source_static: bool,
    pub(crate) return_ok_type: Option<TypeIr>,
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

#[cfg(test)]
mod tests {
    use dust_ir::{SpanIr, TypeIr};
    use dust_text::{FileId, TextRange};

    use super::{FetchMode, QueryFunction, QuerySpec};

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    fn query(function: QueryFunction) -> QuerySpec {
        QuerySpec {
            function,
            fetch: FetchMode::One,
            sql: "SELECT 1".to_owned(),
            sql_source_static: true,
            row_type: None,
            scalar_type: None,
            parameter_count: 0,
            params_source_is_list: true,
            span: span(),
            display_name: None,
        }
    }

    #[test]
    fn fetch_mode_names_match_cache_contract() {
        assert_eq!(FetchMode::One.as_str(), "one");
        assert_eq!(FetchMode::Optional.as_str(), "optional");
        assert_eq!(FetchMode::All.as_str(), "all");
        assert_eq!(FetchMode::Raw.as_str(), "raw");
        assert_eq!(FetchMode::Execute.as_str(), "execute");
    }

    #[test]
    fn query_display_name_prefers_explicit_source_name() {
        let mut spec = query(QueryFunction::Raw);
        spec.display_name = Some("UserDao.findById".to_owned());

        assert_eq!(spec.display_name(), "UserDao.findById");
    }

    #[test]
    fn query_display_name_renders_function_shape() {
        let mut as_query = query(QueryFunction::As);
        as_query.row_type = Some("UserRow".to_owned());
        assert_eq!(as_query.display_name(), "queryAs<UserRow>");

        let dynamic_as = query(QueryFunction::As);
        assert_eq!(dynamic_as.display_name(), "queryAs<dynamic>");

        let mut scalar = query(QueryFunction::Scalar);
        scalar.scalar_type = Some(TypeIr::int());
        assert_eq!(scalar.display_name(), "queryScalar<int>");

        let dynamic_scalar = query(QueryFunction::Scalar);
        assert_eq!(dynamic_scalar.display_name(), "queryScalar<dynamic>");

        assert_eq!(query(QueryFunction::Raw).display_name(), "queryRaw");
        assert_eq!(query(QueryFunction::Execute).display_name(), "queryExecute");
    }
}
