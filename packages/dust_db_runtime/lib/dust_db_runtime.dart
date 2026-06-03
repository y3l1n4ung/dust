/// Compatibility wrapper for Dust DB runtime. Use `package:dust_dart/db.dart`.
library;

export 'package:dust_dart/db.dart'
    show
        Driver,
        Err,
        ExecResult,
        Ok,
        Pool,
        QueryAs,
        QueryExecute,
        QueryRaw,
        QueryScalar,
        RawSql,
        RawSqlx,
        Result,
        Row,
        RowMapper,
        RowMapperRegistry,
        SqlxCardinalityError,
        SqlxDecodeError,
        SqlxDriver,
        SqlxDriverError,
        SqlxError,
        Transaction,
        Unit,
        decodeJsonObject,
        queryAs,
        queryExecute,
        queryRaw,
        queryScalar,
        registerRowMapper,
        unit;
