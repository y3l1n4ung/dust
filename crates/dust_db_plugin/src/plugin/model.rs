use dust_dart_emit::DART_DYNAMIC;
use dust_ir::{ClassIr, FieldIr, MethodIr, SpanIr, TypeIr};

/// Database driver targeted by generated DB code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbDriver {
    /// SQLite through the `dust_db_sqlite3` runtime.
    Sqlite3,
    /// PostgreSQL driver placeholder for future support.
    Postgres,
}

/// Rename rule for SQLx row field names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SqlxRenameRule {
    /// Lowercase field names.
    Lower,
    /// Uppercase field names.
    Upper,
    /// PascalCase field names.
    Pascal,
    /// camelCase field names.
    Camel,
    /// snake_case field names.
    Snake,
    /// SCREAMING_SNAKE_CASE field names.
    ScreamingSnake,
    /// kebab-case field names.
    Kebab,
    /// SCREAMING-KEBAB-CASE field names.
    ScreamingKebab,
}

/// Parsed `@Sqlx` field or class configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SqlxConfig {
    /// Explicit column name override.
    pub(crate) rename: Option<String>,
    /// Bulk rename rule applied to field names.
    pub(crate) rename_all: Option<SqlxRenameRule>,
    /// Whether this field flattens another row-mapped value.
    pub(crate) flatten: bool,
    /// Default expression used when a column is absent or null.
    pub(crate) default_value_source: Option<String>,
    /// Whether this field is skipped by row mapping.
    pub(crate) skip: bool,
    /// Whether this field is decoded from JSON.
    pub(crate) json: bool,
    /// Source expression for try-conversion after row decoding.
    pub(crate) try_from_source: Option<String>,
}

/// Query helper family used by a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryFunction {
    /// Typed row query, `queryAs<T>`.
    As,
    /// Scalar query, `queryScalar<T>`.
    Scalar,
    /// Raw row query, `queryRaw`.
    Raw,
    /// Execute-only query, `queryExecute`.
    Execute,
}

/// Fetch cardinality used by generated DB calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FetchMode {
    /// Require exactly one row.
    One,
    /// Return zero or one row.
    Optional,
    /// Return all rows.
    All,
    /// Return raw row data.
    Raw,
    /// Execute without row decoding.
    Execute,
}

impl FetchMode {
    /// Returns the cache key string used by query metadata.
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

/// Parsed SQL query call ready for validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuerySpec {
    /// Query helper family used by the call site.
    pub(crate) function: QueryFunction,
    /// Fetch cardinality requested by the call.
    pub(crate) fetch: FetchMode,
    /// SQL source text.
    pub(crate) sql: String,
    /// Whether SQL came from a static string literal.
    pub(crate) sql_source_static: bool,
    /// Row type name for typed row queries.
    pub(crate) row_type: Option<String>,
    /// Scalar result type for scalar queries.
    pub(crate) scalar_type: Option<TypeIr>,
    /// Number of user-supplied bind parameters.
    pub(crate) parameter_count: usize,
    /// Whether bind parameters were supplied as a list expression.
    pub(crate) params_source_is_list: bool,
    /// Source span of the query call.
    pub(crate) span: SpanIr,
    /// Optional display name for diagnostics and cache keys.
    pub(crate) display_name: Option<String>,
}

impl QuerySpec {
    /// Returns a human-readable query name for diagnostics and cache metadata.
    pub(crate) fn display_name(&self) -> String {
        if let Some(display_name) = &self.display_name {
            return display_name.clone();
        }
        match self.function {
            QueryFunction::As => {
                format!(
                    "queryAs<{}>",
                    self.row_type.as_deref().unwrap_or(DART_DYNAMIC)
                )
            }
            QueryFunction::Scalar => format!(
                "queryScalar<{}>",
                self.scalar_type
                    .as_ref()
                    .and_then(TypeIr::name)
                    .unwrap_or(DART_DYNAMIC)
            ),
            QueryFunction::Raw => "queryRaw".to_owned(),
            QueryFunction::Execute => "queryExecute".to_owned(),
        }
    }
}

/// Database class annotated for generated database code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DatabaseClass<'a> {
    /// Source database class IR.
    pub(crate) class: &'a ClassIr,
    /// Target database driver.
    pub(crate) driver: DbDriver,
    /// Migration directory source from the annotation.
    pub(crate) migrations: String,
}

/// DAO class annotated for generated SQL methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaoClass<'a> {
    /// Source DAO class IR.
    pub(crate) class: &'a ClassIr,
    /// DAO methods with parsed SQL annotations.
    pub(crate) methods: Vec<DaoMethod<'a>>,
}

/// DAO method with parsed SQL query metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaoMethod<'a> {
    /// Source method IR.
    pub(crate) method: &'a MethodIr,
    /// SQL source text.
    pub(crate) sql: String,
    /// Whether SQL came from a static string literal.
    pub(crate) sql_source_static: bool,
    /// `Ok` type extracted from a `Result` return type.
    pub(crate) return_ok_type: Option<TypeIr>,
}

/// Row class annotated for `FromRow` generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowClass<'a> {
    /// Source row class IR.
    pub(crate) class: &'a ClassIr,
    /// Class-level SQLx row mapping config.
    pub(crate) config: SqlxConfig,
}

/// Row field participating in generated row mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowField<'a> {
    /// Source field IR.
    pub(crate) field: &'a FieldIr,
    /// Field-level SQLx config.
    pub(crate) config: SqlxConfig,
    /// SQL column name after rename rules are applied.
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
