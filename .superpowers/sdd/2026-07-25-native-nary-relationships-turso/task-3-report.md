# Task 3 Report: Fail loudly on a pre-role catalog

## Commit

`008b8caf86b51ead188e13c3eaeec088e6ab2a10` on `feature/graph-nary`
(parent `b9029ec0960d86a167aa7e9661fd145ec7784f37`).

`graph/catalog: reject pre-role catalogs at open` — 1 file changed
(`graph/frontend/src/catalog.rs`), 47 insertions.

## Files changed

- `graph/frontend/src/catalog.rs`
  - Added `CatalogError::IncompatibleGraphLayout { detail: String }` (after
    `InvalidCatalogValue`), with the exact error text from the brief.
  - Added a detection check in `load_registered_graph`, immediately after
    `ensure_catalog_exists(connection)?`, that queries
    `sqlite_schema` for `RELATIONSHIP_ROLES_TABLE` and returns
    `IncompatibleGraphLayout { detail: "{RELATIONSHIP_ROLES_TABLE} is absent" }`
    if it is missing.
  - Added test
    `a_catalog_predating_roles_fails_at_open_and_names_the_fresh_start_policy`
    in `mod tests`, placed after `loading_a_graph_detects_removed_source_tables`.

## Finding: `ensure_catalog_exists` placement

Read `ensure_catalog_exists` (catalog.rs, pre-edit line 642) before trusting
the brief's placement instruction. It does **not** create anything:

```rust
fn ensure_catalog_exists(connection: &Arc<Connection>) -> Result<(), CatalogError> {
    let rows = query_rows(connection, &format!(
        "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = {}",
        sql_string(GRAPHS_TABLE)
    ))?;
    if rows.is_empty() {
        return Err(CatalogError::GraphNotFound("catalog is not initialized".to_owned()));
    }
    Ok(())
}
```

It only checks whether `GRAPHS_TABLE` exists and returns `GraphNotFound`
if not; table creation happens exclusively in `create_catalog`, which is
called only from `register_graph_in_transaction` (i.e., during
registration, never during load). So the risk the brief flagged — "if
`ensure_catalog_exists` CREATEs the catalog tables when missing, it will
recreate the roles table before the check runs" — does not apply to this
codebase: `ensure_catalog_exists` never CREATEs anything.

Consequence for the two scenarios that mattered:

- **Brand-new empty database** (no graph ever registered): `GRAPHS_TABLE`
  doesn't exist, so `ensure_catalog_exists` returns `GraphNotFound` before
  my check ever runs. Fresh graph creation is unaffected — `register_graph`
  never calls `load_registered_graph` until after `create_catalog` has
  already created every catalog table including the roles table.
- **Pre-role catalog** (`GRAPHS_TABLE` exists, a graph is registered, but
  `RELATIONSHIP_ROLES_TABLE` is absent because it predates Task 2):
  `ensure_catalog_exists` passes (GRAPHS_TABLE is present), then my new
  check correctly fires `IncompatibleGraphLayout`.

Conclusion: placed the check exactly where the brief said — "immediately
after `ensure_catalog_exists`" — because the assumed hazard was verified
not to exist here. No restructuring needed.

## Test helpers verification

`connection()`, `create_sources()`, `register_graph()`,
`registration("social")`, `execute_internal`, `query_rows`, `sql_string` all
exist unchanged in the current `mod tests` and were usable as-is —
`registration(...)` still returns a `GraphRegistration` with a `binary(...)`
relationship source, same shape the brief assumed. No helper adaptation was
required.

One real adaptation was needed, discovered via TDD (not assumed): the
brief's test body called `execute_internal(&connection, "DROP TABLE
{RELATIONSHIP_ROLES_TABLE}")` directly, with no open transaction. That
panicked:

```
internal error: entered unreachable code: invalid transaction state for
SetCookie: TransactionState::None, should be write
```

Root cause, verified by reading `core/connection.rs` and
`core/translate/schema.rs`:
- `execute_internal` calls `prepare_internal`, which marks the statement as
  a nested/internal statement (`StatementOrigin::InternalHelper`).
- `validate_drop_table` in `core/translate/schema.rs` only allows DROP TABLE
  on a reserved/system-prefixed table (`is_system_table`) when
  `connection.is_nested_stmt()` is true — otherwise it fails "Cannot drop
  system table". So the DROP had to go through `execute_internal`, not
  plain `connection.execute`.
- But a nested statement cannot itself open a write transaction (same
  constraint documented on `register_graph`'s savepoint-vs-BEGIN logic), so
  issuing it in autocommit mode with no open transaction hits the
  unreachable branch above.

Fix: wrapped the `execute_internal` DROP call in an explicit
`connection.execute("BEGIN IMMEDIATE")` / `connection.execute("COMMIT")`,
mirroring the pattern `register_graph` itself uses for its own internal
DDL. This is a test-only change; no production code needed adjustment for
this issue.

## Test-driven flow

1. **Red (step 2 form — compile failure):**
   ```
   cargo test -p turso_graph_frontend --lib catalog::a_catalog_predating_roles
   ```
   Result: compile error `no variant named 'IncompatibleGraphLayout' found
   for enum 'catalog::CatalogError'` — confirmed the stated red state.

2. **Green (after adding the variant, the check, and fixing the
   transaction wrapping in the test):**
   ```
   cargo test -p turso_graph_frontend --lib catalog::
   ```
   Result: `cargo test: 26 passed, 116 filtered out (1 suite, 0.06s)` — all
   catalog tests, including the new one, pass.

## Gates

- `cargo fmt` (checked with `cargo fmt --check -p turso_graph_frontend`):
  no diff, no changes needed.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  `cargo clippy: 0 errors, 10 warnings`, exit code 0. Gate passes (0 errors
  under `--deny=warnings`; the reported warnings are pre-existing/tool-level
  and not treated as blocking by this invocation).
- `cargo test -p turso_graph_frontend`: `cargo test: 265 passed (11
  suites, 0.60s)`. All green.
- `mise run corpus` (release build, by design):
  - New run recorded: `run_id
    20260726T012938.152246Z-b9029ec0960d-corpus-deep`, `total=10242,
    passed=8927, failed=1315`.
  - Baseline (matching the numbers quoted in the task — 8926 passed / 1316
    failed / 10242 total — found under `run_id
    20260726T005932.453206Z-0678787100af-corpus-deep`, commit `067878710`,
    the immediate parent-side ancestor commit; note: the task described this
    baseline as tagged `82175eb5e...`, but no run in `runs.jsonl` carries
    that commit hash — the row matching the exact stated numbers carries
    `0678787100af` instead. Used the numeric match since the numbers are
    authoritative per the brief.)
  - Diffed the full per-test failure sets between the two runs via
    `graph/test-results/history.jsonl` (both runs' 10242 rows are present):
    exactly one test differs — `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2`
    flipped from `failed` (baseline) to `passed` (mine). Zero tests flipped
    the other way (no new failures).
  - My passed count (8927) is >= baseline (8926), and the single delta is a
    flip toward passing, not away from it — consistent with the ±1
    run-to-run noise the task described, not a regression. Gate passes.
  - `graph/test-results/runs.jsonl` and `graph/test-results/REPORT.md` are
    left uncommitted, per instructions (`history.jsonl` is gitignored, no
    action needed there).

## Concerns / notes

- The task's stated baseline commit hash (`82175eb5e...`) does not appear
  in `runs.jsonl`; I used the row whose numbers exactly match what the task
  described (8926/1316/10242, commit `0678787100af`) as the real baseline
  for comparison. Flagging this discrepancy rather than silently assuming
  which row was intended.
- Never built or ran anything with `--release` except `mise run corpus`,
  per the repo's stated exception.
