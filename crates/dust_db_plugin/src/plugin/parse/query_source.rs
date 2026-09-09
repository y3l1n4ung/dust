use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_EXEC_RESULT, DART_FUTURE, DART_INT, DART_LIST,
    DART_NUM, DART_RESULT, DART_ROW, DART_STRING, DART_UNIT,
};
use dust_ir::{ConfigApplicationIr, DbConfigIr, NormalizedConfigIr, TypeIr};

use crate::plugin::model::{FetchMode, QueryFunction};

use dust_dart_syntax::parse_static_dart_string_literal;

/// Parses SQL source from a `@Query` annotation.
pub(super) fn parse_query_config(config: &ConfigApplicationIr) -> (String, bool) {
    if let Some(NormalizedConfigIr::Db(DbConfigIr::Query(normalized))) = config.normalized.as_ref()
    {
        return (normalized.sql.clone(), normalized.sql_source_static);
    }
    if let Some(sql) = config.positional_string(0) {
        return (sql, true);
    }
    if let Some(sql) = config.named_string("sql") {
        return (sql, true);
    }
    if let Some(value) = config.named_expression_source("sql") {
        return parse_static_dart_string_literal(&value)
            .map(|sql| (sql, true))
            .unwrap_or_else(|| (String::new(), false));
    }
    (String::new(), false)
}

/// Infers query function and fetch mode from a DAO method `Result` type.
pub(super) fn query_shape_from_return(
    ok_type: Option<&TypeIr>,
) -> (QueryFunction, FetchMode, Option<String>, Option<TypeIr>) {
    let Some(ok_type) = ok_type else {
        return (
            QueryFunction::Unsupported,
            FetchMode::Unsupported,
            None,
            None,
        );
    };
    if ok_type.is_named(DART_EXEC_RESULT) || ok_type.is_named(DART_UNIT) {
        return (QueryFunction::Execute, FetchMode::Execute, None, None);
    }
    if ok_type.is_named(DART_LIST) {
        let Some(item) = ok_type.args().first() else {
            return (
                QueryFunction::Unsupported,
                FetchMode::Unsupported,
                None,
                None,
            );
        };
        if item.is_named(DART_ROW) {
            return (
                QueryFunction::Unsupported,
                FetchMode::Unsupported,
                None,
                None,
            );
        }
        return (
            QueryFunction::As,
            FetchMode::All,
            item.name().map(str::to_owned),
            None,
        );
    }
    if is_scalar_type(ok_type) {
        return (
            QueryFunction::Scalar,
            if ok_type.is_nullable() {
                FetchMode::Optional
            } else {
                FetchMode::One
            },
            None,
            Some(ok_type.clone()),
        );
    }
    (
        QueryFunction::As,
        if ok_type.is_nullable() {
            FetchMode::Optional
        } else {
            FetchMode::One
        },
        ok_type.name().map(str::to_owned),
        None,
    )
}

/// Extracts the `Ok` type from `Future<Result<T, E>>`.
pub(crate) fn result_ok_type(return_type: &TypeIr) -> Option<&TypeIr> {
    let future = return_type
        .is_named(DART_FUTURE)
        .then(|| return_type.args().first())
        .flatten()?;
    let result = future.is_named(DART_RESULT).then_some(future)?;
    result.args().first()
}

/// Returns true when a type can be fetched through scalar query helpers.
fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}

/// Parses an explicit query fetch method or returns the default for a helper.
pub(super) fn parse_fetch_method(function: QueryFunction, method: Option<&str>) -> FetchMode {
    // The `With` terminals differ from the generated ones only in taking the row
    // mapping as an argument; the cardinality they ask for is the same.
    match method {
        Some("fetchOptional" | "fetchOptionalWith") => return FetchMode::Optional,
        Some("fetchOne" | "fetchOneWith") => return FetchMode::One,
        Some("fetchAll" | "fetchAllWith") => return FetchMode::All,
        Some("execute") => return FetchMode::Execute,
        _ => {}
    }
    match function {
        QueryFunction::Execute => FetchMode::Execute,
        QueryFunction::Unsupported => FetchMode::Unsupported,
        _ => FetchMode::One,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `Ok` type of a DAO method's `Result`, which is what decides the
    /// terminal the generator emits.
    fn list_of(item: TypeIr) -> TypeIr {
        TypeIr::generic(DART_LIST, vec![item])
    }

    #[test]
    fn a_method_with_no_ok_type_is_unsupported() {
        let (function, fetch, row, scalar) = query_shape_from_return(None);

        assert_eq!(function, QueryFunction::Unsupported);
        assert_eq!(fetch, FetchMode::Unsupported);
        assert!(row.is_none());
        assert!(scalar.is_none());
    }

    #[test]
    fn exec_result_and_unit_both_mean_execute() {
        for ok in [TypeIr::named(DART_EXEC_RESULT), TypeIr::named(DART_UNIT)] {
            let (function, fetch, ..) = query_shape_from_return(Some(&ok));
            assert_eq!(function, QueryFunction::Execute, "{ok:?}");
            assert_eq!(fetch, FetchMode::Execute, "{ok:?}");
        }
    }

    #[test]
    fn a_list_of_a_row_type_fetches_all_of_them() {
        let ok = list_of(TypeIr::named("Order"));

        let (function, fetch, row, _) = query_shape_from_return(Some(&ok));

        assert_eq!(function, QueryFunction::As);
        assert_eq!(fetch, FetchMode::All);
        assert_eq!(row.as_deref(), Some("Order"));
    }

    #[test]
    fn a_list_with_nothing_in_it_is_unsupported() {
        // `List` with no type argument names no row class, so there is nothing
        // to generate a mapping for.
        let ok = TypeIr::named(DART_LIST);

        let (function, fetch, ..) = query_shape_from_return(Some(&ok));

        assert_eq!(function, QueryFunction::Unsupported);
        assert_eq!(fetch, FetchMode::Unsupported);
    }

    #[test]
    fn a_list_of_raw_rows_is_unsupported() {
        // `List<Row>` is the unchecked shape: it asks for no mapping, so a
        // checked terminal cannot be generated for it.
        let ok = list_of(TypeIr::named(DART_ROW));

        let (function, fetch, ..) = query_shape_from_return(Some(&ok));

        assert_eq!(function, QueryFunction::Unsupported);
        assert_eq!(fetch, FetchMode::Unsupported);
    }

    #[test]
    fn every_scalar_dart_type_reads_one_column() {
        for name in [
            DART_STRING,
            DART_INT,
            DART_DOUBLE,
            DART_NUM,
            DART_BOOL,
            DART_DATE_TIME,
        ] {
            let ok = TypeIr::named(name);

            let (function, _, _, scalar) = query_shape_from_return(Some(&ok));

            assert_eq!(function, QueryFunction::Scalar, "{name}");
            assert_eq!(
                scalar
                    .and_then(|ty| ty.name().map(str::to_owned))
                    .as_deref(),
                Some(name)
            );
        }
    }

    #[test]
    fn a_named_terminal_wins_over_the_helper_default() {
        for (method, expected) in [
            ("fetchOptional", FetchMode::Optional),
            ("fetchOptionalWith", FetchMode::Optional),
            ("fetchOne", FetchMode::One),
            ("fetchOneWith", FetchMode::One),
            ("fetchAll", FetchMode::All),
            ("fetchAllWith", FetchMode::All),
            ("execute", FetchMode::Execute),
        ] {
            assert_eq!(
                parse_fetch_method(QueryFunction::As, Some(method)),
                expected,
                "{method}",
            );
        }
    }

    #[test]
    fn without_a_named_terminal_the_helper_decides() {
        assert_eq!(
            parse_fetch_method(QueryFunction::Execute, None),
            FetchMode::Execute,
        );
        assert_eq!(
            parse_fetch_method(QueryFunction::Unsupported, None),
            FetchMode::Unsupported,
        );
        // Everything else reads one row unless the call says otherwise.
        assert_eq!(parse_fetch_method(QueryFunction::As, None), FetchMode::One);
        assert_eq!(
            parse_fetch_method(QueryFunction::Scalar, Some("somethingElse")),
            FetchMode::One,
        );
    }
}
