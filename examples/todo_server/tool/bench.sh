#!/usr/bin/env bash
# Measures throughput across isolate counts with an external load generator.
#
#   bash tool/bench.sh [duration] [connections] [dbPath]
#
# With no path each isolate keeps its own in-memory store. With one, every
# isolate opens the same SQLite file in WAL mode.
set -uo pipefail

DURATION="${1:-10s}"
CONNECTIONS="${2:-64}"
DB="${3:-}"

# Precompiled: `dart run` pays a compile cost on every start, which showed up
# as a benchmark that appeared to hang.
SERVER_BIN=/tmp/bench_server
dart compile exe tool/bench_server.dart -o "$SERVER_BIN" >/dev/null
# The token carries an identity now: `<id>|<scopes>`. A malformed one is a
# 401, which benchmarks the auth extractor rather than the API.
AUTH='authorization: Bearer ada@dust.test|todos:read'

if [[ -n "$DB" ]]; then
  rm -f "$DB" "$DB-wal" "$DB-shm"
  echo "store: sqlite file $DB (WAL)"
else
  echo "store: in memory, one per isolate"
fi

printf '%-9s %-12s %-12s %s\n' isolates 'requests/s' 'latency p50' 'latency p99'

for ISOLATES in 1 2 4 8 12; do
  LOG=$(mktemp)
  "$SERVER_BIN" "$ISOLATES" 0 $DB >"$LOG" 2>&1 &
  SERVER=$!

  PORT=''
  for _ in $(seq 1 120); do
    PORT=$(sed -n 's/.*READY isolates=[0-9]* port=\([0-9]*\).*/\1/p' "$LOG" | head -1)
    [[ -n "$PORT" ]] && break
    kill -0 $SERVER 2>/dev/null || break
    sleep 0.5
  done

  # A server that never came up must fail the run, not quietly hand the load
  # generator whatever else is listening.
  if [[ -z "$PORT" ]]; then
    echo "FAILED to start with $ISOLATES isolates:" >&2
    tail -5 "$LOG" >&2
    kill -INT $SERVER 2>/dev/null
    exit 1
  fi

  OUT=$(wrk -t8 -c"$CONNECTIONS" -d"$DURATION" --latency \
    -H "$AUTH" "http://127.0.0.1:$PORT/api/v1/todos" 2>/dev/null)

  RPS=$(echo "$OUT" | awk '/Requests\/sec/ {print $2}')
  P50=$(echo "$OUT" | awk '/ 50%/ {print $2}')
  P99=$(echo "$OUT" | awk '/ 99%/ {print $2}')
  BAD=$(echo "$OUT" | awk '/Non-2xx/ {print $4}')

  # A run answering errors measures the failure path, not the API. That is how
  # a stale auth header turns a benchmark into a lie.
  if [[ -n "${BAD:-}" ]]; then
    echo "FAILED: $BAD non-2xx responses with $ISOLATES isolates" >&2
    kill -INT $SERVER 2>/dev/null
    exit 1
  fi

  printf '%-9s %-12s %-12s %s\n' "$ISOLATES" "${RPS:-?}" "${P50:-?}" "${P99:-?}"

  kill -INT $SERVER 2>/dev/null
  wait $SERVER 2>/dev/null
  rm -f "$LOG"
  sleep 1
done
