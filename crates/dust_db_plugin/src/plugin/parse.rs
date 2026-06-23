use dust_ir::{DartFileIr, QueryCallIr, QueryFunctionIr};

use super::{
    constants::{FROM_ROW, FROM_ROW_SYMBOL, QUERY},
    model::{DaoClass, DaoMethod, DatabaseClass, QueryFunction, QuerySpec, RowClass},
};

/// Parses annotation configs into DB plugin model settings.
mod annotations;
/// Scans imports for row classes used by generated DAOs.
mod imports;
/// Parses SQL query source and return-shape metadata.
mod query_source;

pub(crate) use annotations::{config_name, effective_column_name, has_config, sqlx_config};
pub(crate) use imports::imported_row_names;
pub(crate) use query_source::result_ok_type;

use annotations::{is_dao_config, is_database_config, parse_database_config};
use query_source::{parse_fetch_method, parse_query_config, query_shape_from_return};

/// Returns database classes annotated for DB generation.
pub(crate) fn database_classes(library: &DartFileIr) -> Vec<DatabaseClass<'_>> {
    library
        .classes
        .iter()
        .filter_map(|class| {
            let config = class
                .configs
                .iter()
                .find(|config| is_database_config(config_name(&config.symbol)))?;
            let parsed = parse_database_config(config)?;
            Some(DatabaseClass {
                class,
                driver: parsed.driver,
                migrations: parsed.migrations,
            })
        })
        .collect()
}

/// Returns DAO classes annotated for generated SQL methods.
pub(crate) fn dao_classes(library: &DartFileIr) -> Vec<DaoClass<'_>> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .configs
                .iter()
                .any(|config| is_dao_config(config_name(&config.symbol)))
        })
        .map(|class| DaoClass {
            class,
            methods: class
                .methods
                .iter()
                .filter_map(|method| {
                    let config = method
                        .configs
                        .iter()
                        .find(|config| config_name(&config.symbol) == QUERY)?;
                    let (sql, sql_source_static) = parse_query_config(config);
                    Some(DaoMethod {
                        method,
                        sql,
                        sql_source_static,
                        return_ok_type: result_ok_type(&method.return_type).cloned(),
                    })
                })
                .collect(),
        })
        .collect()
}

/// Returns row classes annotated for `FromRow` generation.
pub(crate) fn row_classes(library: &DartFileIr) -> Vec<RowClass<'_>> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .traits
                .iter()
                .any(|item| item.symbol.0 == FROM_ROW_SYMBOL)
                || has_config(&class.configs, FROM_ROW)
        })
        .map(|class| RowClass {
            class,
            config: sqlx_config(&class.configs),
        })
        .collect()
}

/// Returns all SQL query specs discovered in a library.
pub(crate) fn query_specs(library: &DartFileIr) -> Vec<QuerySpec> {
    let mut specs = standalone_query_specs(library);
    specs.extend(dao_query_specs(library));
    specs.sort_by_key(|spec| spec.span.range.start());
    specs
}

/// Returns standalone query call specs from lowered query call IR.
fn standalone_query_specs(library: &DartFileIr) -> Vec<QuerySpec> {
    library
        .query_calls
        .iter()
        .map(query_spec_from_call)
        .collect()
}

/// Converts one lowered query call into a query spec.
fn query_spec_from_call(call: &QueryCallIr) -> QuerySpec {
    let function = match call.function {
        QueryFunctionIr::As => QueryFunction::As,
        QueryFunctionIr::Scalar => QueryFunction::Scalar,
        QueryFunctionIr::Raw => QueryFunction::Raw,
        QueryFunctionIr::Execute => QueryFunction::Execute,
    };
    QuerySpec {
        function,
        fetch: parse_fetch_method(function, call.fetch_method.as_deref()),
        sql: call.sql.clone(),
        sql_source_static: call.sql_source_static,
        row_type: matches!(call.function, QueryFunctionIr::As)
            .then(|| call.type_arg_source.clone())
            .flatten(),
        scalar_type: matches!(call.function, QueryFunctionIr::Scalar)
            .then(|| call.type_arg.clone())
            .flatten(),
        parameter_count: call.parameter_count,
        params_source_is_list: call.params_source_is_list,
        span: call.span,
        display_name: None,
    }
}

/// Returns query specs implied by annotated DAO methods.
pub(crate) fn dao_query_specs(library: &DartFileIr) -> Vec<QuerySpec> {
    dao_classes(library)
        .into_iter()
        .flat_map(|dao| {
            dao.methods.into_iter().map(move |method| {
                let (function, fetch, row_type, scalar_type) =
                    query_shape_from_return(method.return_ok_type.as_ref());
                QuerySpec {
                    function,
                    fetch,
                    sql: method.sql,
                    sql_source_static: method.sql_source_static,
                    row_type,
                    scalar_type,
                    parameter_count: method.method.params.len(),
                    params_source_is_list: true,
                    span: method.method.span,
                    display_name: Some(format!("{}.{}", dao.class.name, method.method.name)),
                }
            })
        })
        .collect()
}
