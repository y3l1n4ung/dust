# Database Query Design

Status: settled.

**Dust is a bridge.** It owns neither side of it.

```
  build time                                       runtime
  ──────────                                       ───────

  Dart source                ┌──────────────────┐
    @Derive([FromRow()]) ───▶│                  │──▶ diagnostics
    queryAs<T>(sql, args) ──▶│  dust_db_plugin  │──▶ row decoders
                             │                  │    and terminals
                             └────────┬─────────┘
                                      │
                                      ▼
                             SQLx  describe
                             against ./migrations

                                                  package:sqlite3
                                                  package:postgres
                                                  ── Dust ships none of it
```

On one side, SQLx checks the SQL against the project's own migrations. On the
other, an existing Dart package runs it. In between, Dust reads Dart source,
reports what SQLx found, and generates the one thing neither side can supply:
decoding a `Row` into a Dart class, and the typed terminals that use it.

So Dust does exactly two things:

1. **Validates SQL at build time**, through SQLx `describe`.
2. **Generates row mapping** — the decoder, and the terminals bound to it.

Connecting, pooling, transactions, encoding, argument types, streaming, and
statement caching belong to the Dart driver package.
[`dust_db_sqlite3`](../../packages/dust_db_sqlite3) is a thin adapter over
`package:sqlite3`; Postgres gets the same treatment over `package:postgres`.

Every rejected alternative in this document is Dust trying to be more than a
bridge — see [Rejected](#rejected).

Sections marked **verified** were checked by probing `fixtures/server_app` in
this worktree; each says what was run and what came back. That distinction
earned its keep — it turned up SQL validation that silently does not run, and a
placeholder bug that corrupts generated code.

SQLx references are read from `sqlx-0.8.6` and `sqlx-core-0.8.6` in the local
registry, the version
[`dust_db_plugin/Cargo.toml`](../../crates/dust_db_plugin/Cargo.toml) depends
on.

For the feature as it works today, see [Database usage](../usage/db.md).

## Goals

- Keep SQLx's shape: a query is an expression, written where it is used, with
  its arguments beside it.
- Check what build time can check: SQL against the schema, placeholder arity,
  columns and nullability against the row type.
- `Result`-returning throughout, matching the rest of `dust_dart`.
- No registry, no decoder argument, no type argument restating what `describe`
  already knows. Annotating the row class is the whole declaration.
- One escape hatch, named as one, unreachable from a request handler.
- Same query text on SQLite and Postgres.

## Non-goals

- **Generating query functions.** An earlier draft had `@SqlxQuery()` on a const
  whose type named the terminal and row type, generating a typed function per
  query. It bought build-time argument types and cost three carriers per query,
  a parameter-name heuristic, and SQL that no longer sits in the expression.
  Recorded in [Rejected](#rejected) rather than pursued.
- **A Rust engine over FFI.** Also in [Rejected](#rejected).
- An ORM or query DSL. SQL stays visible and hand-written.
- Changing the driver contract. Everything layers over the five primitives on
  `Executor`.

## Division of responsibility

| Concern | Owner |
| :--- | :--- |
| SQL is valid against the schema | Dust, build time |
| Placeholder arity and gaps | Dust, build time |
| Selected columns cover the row type | Dust, build time |
| Column types and nullability | Dust, build time |
| Row decoding, and the terminals that use it | Dust, generated code |
| Encoding a bound value; argument types | driver |
| `$1` versus `?` placeholder form | driver |
| Binding a `List` for `ANY($1)` / `json_each($1)` | driver |
| `lastInsertId`, `rowsAffected` | driver |
| Pooling, connections, transactions, savepoints | driver |
| Streaming, multi-statement, statement caching | driver |

`dust_dart/db` keeps contracts and one small value type — `Executor`,
`Connection`, `Transaction`, `Row`, `ExecResult`, `SqlxError`, the
`RowMapper<T>` typedef, `Query<T>`, and the build-time annotations. That is the
bridge's entire runtime footprint. Anything that would *decide* something at
runtime is either generated code or driver code.

## What exists today, and what it misses

### The inline path already works (verified)

`queryAs<T>`, `queryScalar<T>`, `queryRaw`, and `queryExecute` are in
[`query.dart`](../../packages/dust_dart/lib/src/db/query.dart). The tree-sitter
parser walks every named node collecting their call sites
([`queries.rs:34`](../../crates/dust_parser_dart_ts/src/queries.rs)), the driver
lowers them into `library.query_calls`, and the DB plugin turns them into the
same `QuerySpec` that DAO methods produce
([`parse.rs:104`](../../crates/dust_db_plugin/src/plugin/parse.rs)). Both reach
the same `describe`.

Verified by appending to `fixtures/server_app/lib/src/shared/db/database.dart` a
plain top-level function holding two inline queries, the second naming a column
that does not exist, then running `dust build --root . --db`:

```
error: SQLx rejected `queryExecute`: error returned from database: (code: 1) no such column: no_such_column
```

Arity is checked (`query binds 1 args but SQL expects 2 parameters`), and
non-constant SQL is rejected rather than skipped (`Database query SQL must be a
static string literal`).

The surface this document wants is largely the surface that ships. What follows
is where it stops short.

### Validation resolved per file (fixed, [#499](https://github.com/y3l1n4ung/dust/issues/499))

`validate.rs` used to return early when the library being validated declared no
`@SqlxDatabase` class, and the row-column map was built from
`row_classes(library)` — the row classes in that same file.

Three consequences, all verified:

1. **A DAO in a file without the database class is not described.** Injecting
   `no_such_column` into two queries in `orders_repo.dart` gave a clean
   `dust build` and a clean `dust build --db`; the broken SQL was written into
   `orders_repo.g.dart` at lines 25 and 39, and `dart test test/orders_test.dart`
   then failed 7 tests.
2. **The same holds for call sites.** The identical inline `queryExecute` that
   errored from `database.dart` produced no diagnostic from `orders_repo.dart`.
3. **Row coverage is unchecked across files.** `queryAs<Order>` selecting only
   `id, item` — missing `account_id`, `quantity`, `placed_at` — passed, because
   `Order` lives in `order_model.dart` and the lookup misses.

A realistic layout puts the database class, the row classes, and the queries in
three different files, so in `fixtures/server_app` this turned SQL validation off
everywhere. It was the most important item in this document: without it nothing
else here was actually checked in a normal project.

The database class and the row classes are now collected during the shared
workspace scan, before any library is validated, and every library validates
against the whole package. See [Validation scope](#validation-scope).

Discovery had to widen with it. It read only the name written after the `@`, and
a row mapper is normally declared `@Derive([FromRow()])`, so `dust db build`
never opened the file declaring the row type a query is checked against. It now
reads the constructor calls nested in an argument list too — skipping string
literals, so `@Query` SQL contributes nothing — which finds the file by the name
the DB plugin actually owns rather than by teaching it another plugin's wrapper.

That exposed one more. A focused registry runs one plugin and registers the rest
for symbol ownership only, so a library discovered for its row mapper was
emitted from serde markers belonging to a plugin that was not running, replacing
a full build's output with a bare header. Emission acts on those markers only
when every registered plugin executes.

### Describe results are read for column names only (verified)

`validate_described_columns`
([`validate/sqlx.rs`](../../crates/dust_db_plugin/src/plugin/validate/sqlx.rs))
checks that required columns are present. `sqlx::Describe` also carries
`nullable(i)` and `column.type_info()`; neither appears anywhere in
`crates/dust_db_plugin`. So `final int quantity` against a `TEXT` column passes,
and a nullable column decoded into a non-nullable field passes.

### Placeholder scanning ignored SQL comments (fixed, [#500](https://github.com/y3l1n4ung/dust/issues/500))

`rewrite_sqlite_placeholders`
([`sql.rs`](../../crates/dust_db_plugin/src/plugin/sql.rs)) tracked single and
double quotes, so a `$1` inside a string literal was not mistaken for a
placeholder. It did not track `--` line comments or `/* */` block comments.

**False rejection**, in a file that is described:

```sql
-- filter by $9 owner
SELECT count(*) FROM orders WHERE account_id = $1
```

```
error: SQL placeholders must not skip `$2`
```

**Silent corruption**, in a file that is not. Adding `/* owner is $2 */` to
`deleteOrder` in `orders_repo.dart` built clean and generated:

```dart
return _db.execute(
  r'''
DELETE FROM orders WHERE id = ? /* owner is ? */ AND account_id = ?
''',
  [id, accountId, accountId],
);
```

Three arguments bound to a statement SQLite sees as having two placeholders, and
the comment text rewritten. `dart test test/orders_test.dart` then failed.

When the file *is* described, an expanded-count cross-check catches it — by
accident, with a diagnostic that blames the rewriter rather than the comment.

The scanner now copies through everything that can hold a `$1` without meaning
one: string literals, quoted identifiers, both comment forms, and Postgres
dollar-quoted bodies, which a `CREATE FUNCTION` body in a migration would
otherwise trip over. Block comments do not nest, which is what SQLite does. The
count cross-check that used to catch this by accident now names what went wrong
and prints the SQL the database parsed.

### The two surfaces disagree on errors (verified)

DAO methods return `Future<Result<T, SqlxError>>`. `QueryAs.fetchAll` calls
`_unwrap` and throws. The rest of `dust_dart` is `Result`-first.

### `queryRaw` is four holes wearing one name

It returns untyped `List<Row>`; its shape invites `'SELECT * FROM $table'`; and
`raw` lives on `Executor` rather than `DatabaseExecutor`
([`pool.dart:88`](../../packages/dust_dart/lib/src/db/pool.dart)), so reaching
it needs `db as Executor` — a cast that always succeeds, because `Pool`,
`Connection`, and `Transaction` all implement `Executor`. A fence that stops
nobody.

## The query surface

### A typed row query

```rust
let orders = sqlx::query_as!(
    Order,
    "SELECT id, account_id, item, quantity, placed_at FROM orders \
     WHERE account_id = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
    account_id, limit, offset,
)
.fetch_all(&pool)
.await?;
```

```dart
final orders = await queryAs<Order>(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1 ORDER BY id DESC LIMIT $2 OFFSET $3
''', [accountId, limit, offset]).fetchAll(pool);
```

Row type named once, SQL inline, arguments beside it. `orders` is
`Result<List<Order>, SqlxError>` where Rust has `Result<Vec<Order>, sqlx::Error>`
after `?`.

### An anonymous row

SQLx's `query!` returns a generated per-row struct. Dart records are that
without the ceremony:

```rust
let rows = sqlx::query!("SELECT id, item FROM orders WHERE account_id = $1", account_id)
    .fetch_all(&pool).await?;
```

```dart
final rows = await queryAs<({int id, String item})>(
  r'SELECT id, item FROM orders WHERE account_id = $1',
  [accountId],
).fetchAll(pool);
```

Dust generates the decoder for each record shape a query names, into the
library's own part file.

### Scalar and statement

```dart
final count = await queryScalar<int>(
  r'SELECT count(*) FROM orders WHERE account_id = $1', [accountId]).fetchOne(pool);

final done = await queryExecute(
  r'DELETE FROM orders WHERE id = $1 AND account_id = $2', [id, accountId]).execute(pool);
```

### Several queries in one function, in a transaction

```rust
let mut tx = pool.begin().await?;
sqlx::query!("UPDATE stock SET on_hand = on_hand - $1 WHERE item = $2 AND on_hand >= $1",
             order.quantity, order.item).execute(&mut *tx).await?;
let placed = sqlx::query_as!(Order, "INSERT ... RETURNING ...", ...)
    .fetch_one(&mut *tx).await?;
tx.commit().await?;
```

```dart
return pool.transaction((tx) async {
  final taken = await queryExecute(r'''
UPDATE stock SET on_hand = on_hand - $1 WHERE item = $2 AND on_hand >= $1
''', [order.quantity, order.item]).execute(tx);

  if (taken case Err(:final error)) return Err(error);
  if (taken.unwrap().rowsAffected == 0) {
    return Err(SqlxError.query('out of stock', operation: 'takeStock'));
  }

  return queryAs<Order>(r'''
INSERT INTO orders (account_id, item, quantity, placed_at)
VALUES ($1, $2, $3, $4)
RETURNING id, account_id, item, quantity, placed_at
''', [accountId, order.item, order.quantity, placedAt]).fetchOne(tx);
});
```

`RETURNING` rather than `ExecResult.lastInsertId`, which is
`sqlite3.lastInsertRowId` and has no Postgres equivalent.

SQLx hands back a guard and asks for an explicit commit; Dust scopes it to a
closure. Rust's `Drop` rolls back an uncommitted transaction, and Dart has no
destructors, so the closure — commit on `Ok`, roll back on `Err` or a throw — is
the only shape that is safe by construction here.

### Chaining

`Result` has `map`, `mapErr`, `andThen`, and `orElse`
([`result.dart`](../../packages/dust_dart/lib/src/fp/result.dart)), all
synchronous. Nothing chains a `Future<Result>`. Rust has `?`; the nearest Dart
equivalent is one extension, which belongs in `dust_dart/fp.dart` rather than
the DB library:

```dart
extension FutureResultChain<T, E> on Future<Result<T, E>> {
  Future<Result<R, E>> andThen<R>(FutureOr<Result<R, E>> Function(T value) next);
  Future<Result<R, E>> map<R>(FutureOr<R> Function(T value) mapper);
  Future<Result<T, F>> mapErr<F>(FutureOr<F> Function(E error) mapper);
}
```

## Row mapping is the generated part

`@Derive([FromRow()])` on the row class is the whole declaration. A query names
its row type and nothing else — no decoder argument, no registry.

### Why the obvious approaches fail

SQLx writes `query_as::<_, Order>(sql)` and resolves decoding through the bound
`Order: FromRow<'r, R>`. That is a **static-side trait**: the compiler picks
`Order::from_row` from the type alone, no value involved, nothing at runtime.

Dart has no static interface members. A type parameter `T` cannot reach a
constructor, a static, or a factory, so `T` alone can never produce a decoder.
That leaves three options:

1. **A runtime registry.** What used to ship — `RowMapperRegistry` was a
   process-wide `Map<Type, RowMapper>` filled by top-level initializers in
   generated part files. A miss was a runtime `SqlxError.decode`, and whether
   it hit depended on whether the part file had been imported anywhere in the
   isolate. **Deleted**; nothing outside `dust_dart`'s own tests used it, since
   generated DAOs have always passed their mapper directly.
2. **An explicit decoder argument** on every query — a value mechanically
   derivable from `T`, which is the work a generator exists to do.
3. **Generated extensions.** Below.

Option 2 is what ships, because it is what JSON already does.
`Serialize` can generate an instance interface — `mixin _$Order implements
Serializable` — since the value exists before `serialize()` is called.
`Deserialize` cannot, for the same reason `FromRow` cannot: it constructs.
Serde's answer there is a **const witness object**, `$OrderDeserializer
implements Deserializer<Order, Map<String, Object?>>`, and the row side now
mirrors it exactly:

```dart
abstract interface class RowDeserializer<T> {
  T deserialize(Row row);
}

final class $OrderFromRow implements RowDeserializer<Order> {
  const $OrderFromRow();
  @override
  Order deserialize(Row row) => OrderFromRow.fromRow(row);
}
```

The witness is public API, and the `With` terminals take one for a row type Dust
does not own. Nothing has to pass one for a row type it does: option 3 ships
alongside it, below. `dust db build` reports a missing mapping first, and can
only do so because row classes resolve package-wide.

### Terminals come from generated extensions

`queryAs<T>(sql, args)` returns a `QueryAs<T>`: a data holder with `sql`,
`parameters`, and the three `…With` primitives, but no bare terminals. (The name
stays `QueryAs` rather than the `Query<T>` first sketched here, because `Query`
is already the DAO method annotation.) The terminals are generated per row type,
into the row class's own part file, beside the decoder:

```dart
// order_model.g.dart
Order _$OrderFromRow(Row row) => Order(
      id: row.readInt('id'),
      accountId: row.readInt('account_id'),
      item: row.readString('item'),
      quantity: row.readInt('quantity'),
      placedAt: row.readString('placed_at'),
    );

extension $OrderQuery on QueryAs<Order> {
  Future<Order> fetchOne(DatabaseExecutor db) =>
      fetchOneWith(db, _$OrderFromRow);

  Future<Order?> fetchOptional(DatabaseExecutor db) =>
      fetchOptionalWith(db, _$OrderFromRow);

  Future<List<Order>> fetchAll(DatabaseExecutor db) =>
      fetchAllWith(db, _$OrderFromRow);
}
```

The decoder is a plain top-level function of type `RowMapper<Order>`, named the
way serde names `_$OrderSerialize`. It introduces no method name, so nothing can
collide with `Serializable.serialize` or `Deserializer.deserialize` from
[`serde.dart`](../../packages/dust_dart/lib/src/serde/serde.dart) — a row class
deriving both `FromRow` and `Deserialize` gains two unrelated top-level
functions.

Dart resolves extension members from the **static type** of the receiver, so
`queryAs<Order>(...).fetchAll(pool)` picks `OrderQuery.fetchAll` at compile
time. No lookup, no global state, no argument. As close as Dart gets to what
Rust's trait resolution does for SQLx.

`QueryAs<T>` must stay free of bare terminal methods, because an instance member
always beats an extension member. The `…With` terminals are safe there: they are
different names, and they take the mapping rather than assuming one.

Three properties follow:

- **A missing derive is a compile error.** `queryAs<Account>(...)` where
  `Account` does not derive `FromRow` means no extension exists, so `fetchAll`
  is undefined. Nothing is passed to get this.
- **A missing import is a compile error too**, rather than a silent
  non-registration.
- **`RowMapperRegistry` is deleted**, along with the generated
  `registerRowMapper` initializers.

`RowMapper<T>` survives as the typedef `T Function(Row)` on the five executor
primitives — the driver seam, which takes a function.

Scalar and statement terminals need no generation: `QueryScalar<T>` and
`QueryExecute` decode without a row class, so they ship as real methods.

### The cost

Static resolution is not generic:

```dart
Future<Result<List<T>, SqlxError>> loadAll<T>(Query<T> q, Executor executor) =>
    q.fetchAll(executor); // does not compile
```

At the call site `T` is a variable, so no extension applies. Rust says
`T: FromRow`; Dart cannot express that bound. Code generic over row types drops
to the executor primitives and passes a `RowMapper<T>`.

Paid by generic plumbing, not by ordinary query code.

## Arguments

`query_as!` does not check argument types itself. It **expands to code that
makes rustc check** — assertions against the described parameter types appear in
the expansion. Dust writes `part` files and cannot emit an assertion into the
middle of an expression, so argument types are the driver's business at
execution.

Postgres rejects a wrong bind. SQLite's type affinity often coerces instead, so
a `String` bound to an `INTEGER` column is stored rather than refused. That is a
real weakness of the SQLite target and belongs in the usage guide.

This is the same split SQLx has between its macros and its functions. Arity,
gaps, SQL validity, row coverage, column types, and nullability are all still
build-time.

## Replacing `queryRaw`

Four unrelated problems, one name. Three have static answers.

### `IN` lists need no dynamic SQL

```dart
// Postgres: a real array type, one placeholder.
await queryAs<Order>(
  r'SELECT id, account_id, item, quantity, placed_at FROM orders WHERE id = ANY($1)',
  [ids],
).fetchAll(pool);

// SQLite: json_each, one placeholder.
await queryAs<Order>(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE id IN (SELECT value FROM json_each($1))
''', [ids]).fetchAll(pool);
```

Both describable. `package:sqlite3` will not bind a nested `List`, so the SQLite
driver JSON-encodes a `List` argument on the way through rather than making
callers remember `jsonEncode`. Driver work, and it removes the most common
reason anyone reaches for raw SQL.

### Optional filters are separate queries

SQLx builds them at runtime with `QueryBuilder`, and nothing checks the result —
not even tier 1. Under this scope Dust builds nothing at runtime either, so a
handful of optional filters is a handful of queries chosen by a `switch`:

```dart
final query = switch ((item, minQuantity)) {
  (null, null) => queryAs<Order>(_all, [accountId]),
  (final i?, null) => queryAs<Order>(_byItem, [accountId, i]),
  (null, final m?) => queryAs<Order>(_byQuantity, [accountId, m]),
  (final i?, final m?) => queryAs<Order>(_byBoth, [accountId, i, m]),
};
final orders = await query.fetchAll(pool);
```

Every branch is a constant string that `describe` accepted, and the switch is
exhaustive. Past three or four filters this stops scaling, and at that point the
query genuinely is dynamic — see [administrative SQL](#what-remains-administrative-sql).

A `SearchQuery` declaration that enumerated the 2ⁿ variants at build time was
considered and left out; see [Rejected](#rejected).

### Sort order is a choice among queries

A sort column is an identifier, and no dialect binds an identifier — string
building is the only mechanism, which is why it is the classic injection site.
So it is a `switch` over an enum, exhaustive, with every branch described.

### What remains: administrative SQL

Migrations, `EXPLAIN`, one-off operations. Real, and rare.

```dart
abstract interface class UnsafeSql {
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters);
  Future<Result<List<T>, SqlxError>> fetchAs<T>(
      String sql, List<Object?> parameters, RowMapper<T> mapper);
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters);
}
```

Three deliberate differences from `raw`:

- **Named to sting.** `queryRaw` reads like a peer of `queryAs`. `unsafeSql`
  reads like what it is, and it greps.
- **Reached from the database facade, not the executor.** `AppDatabase.unsafe`,
  never `Executor.unsafe`. A handler holds an executor, so it cannot reach this
  — unlike today's cast, which always succeeds.
- **The decoder is passed explicitly.** Generated terminals exist only for
  validated queries. The asymmetry is deliberate: the checked path is the
  ergonomic one.

Dust warns at each use, suppressible per call, so it shows up as a deliberate
line in a diff.

## Naming

Pool, connection, and transaction names follow SQLx, down to the per-driver
aliases. `SqliteConnectOptions` already does — `fixtures/server_app` names it
exactly as `sqlx-sqlite` does — so this extends a precedent rather than setting
one.

| SQLx | Dust today | Proposed |
| :--- | :--- | :--- |
| `Executor` (trait) | `DatabaseExecutor` | `Executor` |
| — (`Executor` plus unchecked SQL) | `Executor` | removed |
| `Pool<DB>` | `Pool` | `Pool` |
| `PgPool`, `SqlitePool` | — | same names, in the driver package |
| `PoolOptions`, `PgPoolOptions`, `SqlitePoolOptions` | — | same names |
| `PoolConnection<DB>` | — | `PoolConnection` |
| `Connection` (trait) | `DatabaseConnection` | `Connection` |
| `PgConnection`, `SqliteConnection` | — | same names, in the driver package |
| `Transaction<'_, DB>` | `DatabaseTransaction` | `Transaction` |
| `PgConnectOptions`, `SqliteConnectOptions` | `SqliteConnectOptions` | unchanged |
| — | `SqlxDriver` typedef | removed |

The collision resolves itself. `Executor` currently means "`DatabaseExecutor`
plus unchecked `raw` SQL"
([`pool.dart:88`](../../packages/dust_dart/lib/src/db/pool.dart)). Once `raw`
moves to `unsafeSql`, that type has nothing left to add and disappears.

Examples name values the way SQLx does: `pool`, `conn`, `tx`.

## Validation scope

`@SqlxDatabase` is declared once per package. Migrations, dialect, and row
classes resolve **package-wide**, and every library in the package validates
against that schema whether or not it declares anything.

## Dialects

Query text does not change between dialects. `$1` stays `$1` in source; the
driver decides whether it wants `$1` or `?`.

```dart
@SqlxDatabase(type: SqlxDatabaseType.postgres, migrations: './migrations')
abstract class AppDatabase implements DatabaseClient {
  factory AppDatabase.connect(String url, {PgConnectOptions? options}) =
      _$AppDatabase.connect;
}
```

Postgres parses today
([`annotations.rs:164`](../../crates/dust_db_plugin/src/plugin/parse/annotations.rs))
and then hits three walls: validation raises "Driver.postgres is reserved for a
future Database release", emit produces `throw UnsupportedError`
([`emit/database.rs:46`](../../crates/dust_db_plugin/src/plugin/emit/database.rs)),
and `validate_sqlx_describe` returns early on any non-SQLite driver.

The runtime is `package:postgres` wrapped the way `package:sqlite3` is wrapped
today — a `dust_db_postgres` package implementing the five `Executor`
primitives, `Row`, transactions, and the migration runner. Wrapping, not
implementing. It needs one short evaluation against the `Executor` contract:
positional prepared statements, transaction and savepoint support, and pooling.

Three decisions are dialect-shaped:

- **Opening.** `.open(String path)` versus `.connect(String url)`. Only one
  fixture depends on `.open` today.
- **`ExecResult.lastInsertId`.** SQLite-only. `RETURNING` is the portable
  answer; `lastInsertId` becomes nullable and SQLite-specific, or leaves.
- **Connect options.** Per-driver types, as SQLx has.

### Validating against Postgres

`describe` needs a live server — there is no `sqlite::memory:` equivalent. So
`DUST_DATABASE_URL` drives local builds, and the offline query cache
([`validate/cache.rs`](../../crates/dust_db_plugin/src/plugin/validate/cache.rs),
already built) is the CI path. The cache key gains the driver, so a SQLite cache
cannot satisfy a Postgres build. Same arrangement SQLx uses for `.sqlx/`.

### The scanner already understands dollar-quoting

Postgres dollar-quoted strings — `$$body$$` and `$tag$body$tag$` — are neither
`'` nor `"`, and a `$1` inside one would be counted as a parameter. Postgres
needs no placeholder rewrite, but arity and gap validation still scan the text,
and a migration with a `CREATE FUNCTION` body is the obvious trigger. That
state went in with the comment fix rather than waiting for the driver.

## Advanced cases

### Nullability overrides

Reading `Describe::nullable(i)` and checking it against the row type is the
plan. SQLx does exactly that, and then had to add an escape hatch:

- A `LEFT JOIN` makes a `NOT NULL` column nullable in the result.
- `max(x)` and `sum(x)` return `NULL` over an empty set; `count(*)` does not.
- Postgres reports less than SQLite does, so more columns come back "maybe null".

SQLx's answer is column-alias syntax (`sqlx/src/macros/mod.rs`, *Nullability:
Output Columns*): `foo as "foo!"` forces NOT NULL, `foo as "foo?"` forces
nullable. It works because SQL allows arbitrary text in a column alias, and the
same trick is available here — the alias is the column name the decoder reads,
so the marker is stripped when generating `row.readInt('foo')`.

Without it, nullability checking will reject working queries the first time
someone writes a `LEFT JOIN`, and the checks have to ship as warnings.

### Type overrides

SQLx supports `foo as "foo: OrderId"`, with `"foo!: OrderId"` and
`"foo?: OrderId"` combining both. `@Sqlx(tryFrom:)` covers some of this at the
field level, but not per query, and not for a column only one query selects.

Separately, and Postgres-only, SQLx treats a cast on a bind parameter as an
override: `$1::int` suppresses type checking for that argument. Since arguments
are the driver's business here anyway, a cast is simply how a user tells
Postgres what they mean.

### Custom types and Postgres enums

```rust
#[derive(sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
enum OrderStatus { Pending, Shipped }
```

`@Sqlx(tryFrom:)` converts on the way in but says nothing about the
database-side type name, which Postgres needs for enum and composite types.

### Types with no Dart equivalent

Postgres `numeric`/`decimal` is arbitrary-precision. Dart has no built-in
counterpart, so either a package type is required or the column is read as
`String`. Money columns make this immediate. `interval`, `uuid`, and
`timestamptz` each need a decided mapping.

### Checked and found sound

Two hazards worth naming because they turned out not to be problems:

- **Nested transactions work.** `Sqlite3Driver` runs a root `BEGIN` when it owns
  the database and a `SAVEPOINT` otherwise
  ([`transaction.dart`](../../packages/dust_db_sqlite3/lib/src/transaction.dart)).
- **An escaped transaction handle fails loudly.** The handle is deactivated in a
  `finally`, so using a `tx` after its closure returns raises instead of
  silently writing outside the transaction. Rust prevents this with borrowing;
  Dart cannot, so deactivation is the right substitute and it is already there.

## What happens to `@SqlxDao`

It stays, unchanged, and lowers to the same `QuerySpec` it lowers to now. It is
the right shape when a named, injectable seam is wanted — a repository a test
replaces wholesale, which a top-level query expression cannot offer. It stops
being the only validated path.

One known wart: a DAO binds its executor at construction, so using one inside a
transaction means building a second. `fixtures/server_app` does exactly that, in
`inventory_handler.dart:48`:

```dart
final inventory = InventoryRepo(tx);
final orders = OrdersRepo(tx);
```

Inline queries take the executor per call and do not have this problem.

## Work, in order

1. ~~**Resolve `@SqlxDatabase` and row classes per package.**~~ Done — validation
   is on ([#499](https://github.com/y3l1n4ung/dust/issues/499)).
2. ~~**Fix the placeholder scanner.**~~ Done — comment states and dollar-quoting
   both ([#500](https://github.com/y3l1n4ung/dust/issues/500)).
3. **Read `nullable()` and `type_info()`**, shipping as warnings until the
   [type-mapping table](#open-questions) exists.
4. **Terminals return `Result`.** ~~Generate them per row type; delete
   `RowMapperRegistry`.~~ Both done; what is left is the error surface, since
   the generated terminals still throw the way the old instance methods did.
5. **`queryRaw` becomes `unsafeSql`** on the database facade.
6. **Name the enclosing function** in call-site diagnostics.
7. **Rename to SQLx's pool vocabulary.**
8. **`dust_db_postgres` over `package:postgres`**, with the offline cache as the
   CI path.

Steps 1–3 are bug fixes and land independently of the rest.

## Rejected

- **Generated query functions.** `@SqlxQuery()` on a const whose type named the
  terminal and row type, generating `$pageForAccount(executor, …)` with
  parameter types from `describe`. It is the only way to get build-time argument
  types, and it costs three carriers per query, a parameter-name heuristic
  (`describe` reports types and order, never names), SQL that leaves the
  expression, and code that will not analyze until the `--db` pass has run.

- **A bind-record type argument**, `QueryAs<Order, (int, int, int)>`. Restates
  in every declaration what `describe` already knows, and is not SQLx's shape.

- **A generated parameter-type table** consulted at bind time. A registry under
  another name.

- **SQLx over FFI as the runtime driver.** One engine validating and executing
  would remove all drift between build-time and runtime behavior, and would
  bring Postgres, MySQL, and SQLite at once. But `examples/shopping_app` uses
  the database, so Flutter mobile is in scope, and `release.yml` cross-compiles
  to six desktop and server triples with no iOS or Android among them and no
  plugin packaging anywhere in the repo. Wrapping `package:postgres` is the
  smaller path.

- **A `SearchQuery` declaration** enumerating 2ⁿ filter combinations at build
  time. Stricter than SQLx's unchecked `QueryBuilder`, and the largest new
  surface in the design for a case a `switch` covers at small n.

## Open questions

- **How a Dart field type maps to a described SQL type.** SQLite's type affinity
  is loose enough that a strict mapping will reject working code. A table of
  accepted pairs per dialect is needed before type and nullability checks can be
  errors rather than warnings.

- **Whether nullability overrides ship with the checks.** Without `foo as "foo!"`
  the first `LEFT JOIN` produces a false error, which argues they are the same
  piece of work rather than a follow-up.

- **How much generic plumbing exists in practice.** Generated terminals cannot
  be reached through a type variable. Whether that is a footnote or a recurring
  annoyance is empirical, and the fixture should answer it.

- **Whether `package:postgres` satisfies the `Executor` contract** as-is:
  positional prepared statements, transactions with savepoints, and pooling.
