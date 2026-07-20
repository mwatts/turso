# Graph frontend: remaining failure long tail

Triage of every currently-failing record in the conformance corpus and the
CypherBench benchmark, as of the data below. No corpus or benchmark run was
executed to produce this report; both are read directly from their existing
on-disk results.

## Summary

**Conformance corpus** (`graph/test-results/history.jsonl`, run
`20260720T190819.946079Z-5f1aa4051d8d-corpus-deep`): 10,082 records, 8,203
passed, **1,879 failing**.

Failing records by root-cause shape:

| Bucket | Count | Sources |
| --- | ---: | --- |
| Unsupported feature (parser/binder rejects a construct by name) | 534 | tck 158, age 332, sparrowdb 30, grafeo 11, cqlite 3 |
| Result mismatch (executes, wrong rows) | 501 | tck 481, grafeo 20 |
| Bare parse error (grammar cannot derive the query at all) | 370 | tck 65, age 207, grafeo 51, sparrowdb 34, cqlite 13 |
| Missing/misrouted function | 290 | tck 64, age 167, sparrowdb 42, grafeo 15, cqlite 2 |
| Fixture/dataset setup failure | 57 | tck 21, grafeo 36 |
| Missing error detection (spec requires a runtime error; execution succeeded) | 40 | tck 40 |
| Other (integer/relationship range edge cases, misc) | 25 | mixed |
| Side-effect mismatch (mutation ran, wrong +nodes/+rels/+props count) | 23 | tck |
| IR invariant failure (internal assertion, not a "not supported" rejection) | 14 | tck 3, age 11 |
| Duplicate variable rejected in mutation path | 11 | age |
| Unknown variable rejected in mutation path | 8 | tck 5, age 3 |
| Unsupported unicode escape (surrogate pairs) | 6 | age |
| **Total** | **1,879** | |

**CypherBench** (`/tmp/cypherbench-final-detail.jsonl`): 2,348 rows, 2,278
matched, 58 mismatched, 11 errored, 1 timeout — **70 non-matched**, which
decompose into 6 mutually exclusive clusters: bool-as-int (29),
UNWIND-of-NULL extra row (25), errored temporal field-chain access (11),
ORDER BY/LIMIT 1 tie-break (3), timeout (1), float precision (1).

Two of these clusters are the same defect wearing two hats: the benchmark's
11 "errored" rows and part of the corpus's "property access requires a node
or relationship" failures (41 rows total: tck 13, grafeo 2, age 26) share one
root cause — the binder rejects `n.date.year`-style chained property access
on temporal values because the IR's `ValueType` enum has no `Date`/`DateTime`
variant, even though the runtime function that would answer the access
(`temporal_get`) already exists in `graph/temporal`.

A prior scope decision to set aside four classes as unfixable/out-of-scope
has been reversed: AGE `jsonb` operator syntax and pgvector casts, postgres
`EXPLAIN` forms, `reduce()`, and spec-mandated runtime `TypeError`/
`EntityNotFound` errors are all now carried in the impact-ordered queue below
as feature work, each tagged with a verified core-dependency flag. Two of
those four were originally over-rated in complexity and are corrected here
with source citations: pgvector needs no core work (turso core already ships
a full vector type/function surface), and postgres `EXPLAIN` needs no core
work either (turso core already implements `EXPLAIN QUERY PLAN` in full).

## Impact-ordered fix queue

Ordered by estimated rows unlocked (conformance + benchmark + quantified
currently-excluded donor rows), complexity as a tiebreaker at equal impact.

| # | Cluster | Conformance | Benchmark | Excluded-donor potential | Core dep. | Complexity | Isolation |
| --- | --- | ---: | ---: | ---: | :---: | :---: | --- |
| 1 | Unify the read/mutation dual-binder architecture | 109 direct, ~250+ as downstream second error | – | – | No | L | Cross-cutting (`binder.rs` entire mutation path) |
| 2 | Entity-introspection functions restricted to bound pattern variables | ~98 | – | – | No | M | Contained (`binder.rs`) |
| 3 | Temporal `ORDER BY` not instant-normalized | 113 | – | – | No | M | Contained (`lowering.rs`) |
| 4 | AGE jsonb operator syntax → map onto core JSON functions | 0 measured | – | 250 | Partial (containment ops) | S–M (M–L for `@>`/`<@`) | Contained (grammar+binder+lowering) |
| 5 | Generic missing/misnamed function surface (long tail of ~15 names) | ~70 | – | – | Mixed | S–M each | Contained, independent per function |
| 6 | Procedures / `CALL` outside the hardcoded registry | 67 | – | – | No | M | Contained (`binder.rs::bind_call`) |
| 7 | `reduce()` — grammar gap now, recursion later | 76 | – | – | Yes (full); No (bounded-unroll subset) | S (grammar); L (full lowering) | Contained (grammar); cross-cutting (needs core recursive CTEs) |
| 8 | pgvector casts/operators → map onto existing core vector functions | 0 measured | – | 71 | No | S–M | Contained (grammar+binder+lowering) |
| 9 | Read-path binder semantic long tail (arithmetic/boolean operand typing, indexing, VLE edge cases, ~15 distinct gaps) | ~101 | – | – | No | S–M each | Contained, mostly independent |
| 10 | Nested list-comprehension outer-scope variable capture | 33 | – | – | No | M | Contained (`lowering.rs`) |
| 11 | OPTIONAL MATCH / UNWIND null propagation | 44 | 25 | – | No | M | Contained (`lowering.rs`, two plan-kind arms) |
| 12 | Bitemporal interval calculus (`btic*`) function family | 28 | – | – | No | L | Isolated new module, niche donor extension |
| 13 | Entity literal rendered as raw id after CREATE/MERGE | 24 | – | – | No | S–M | Contained (`mutation.rs` + projection rendering) |
| 14 | Boolean value erasure (renders as `0`/`1`) | ~7 | 29 | – | No | S | Contained (value-rendering boundary) |
| 15 | Runtime-error mechanism for spec-mandated TypeError/EntityNotFound | 40 | – | – | No | M | Contained (new static-extension function, per `graph/temporal` pattern) |
| 16 | Side-effect accounting mismatch in mutation execution | 23 | – | – | No | M | Contained (`mutation.rs`) |
| 17 | AGE catalog-admin functions (`vertex_stats`, `graph_stats`, …) | 50 | – | – | No | S | Low value — AGE-internal, not real Cypher |
| 18 | sparrowdb vector/FTS surface (`vector_similarity`, `full_text_search`, …) | 17 | – | – | Depends on FTS primitive | M | Contained, adjacent to #8 |
| 19 | postgres `EXPLAIN` forms → reuse core `EXPLAIN QUERY PLAN` | 0 measured | – | 60 | No | S–M | Contained (testkit + frontend SQL-prefixing) |
| 20 | `graph IR invariant failed: duplicate binding name` | 14 | – | – | No | M (needs root-cause, not just rejection) | Contained (IR construction) |
| 21 | Legal variable reuse (MATCH→CREATE) rejected as duplicate/unknown | 19 | – | – | No | S–M | Mostly resolved by #1 |
| 22 | Unsupported UTF-16 surrogate-pair unicode escapes | 6 | – | – | No | S | Fully isolated (lexer) |
| 23 | NaN comparison returns NULL instead of definite bool | 7 | – | – | No | S | Contained, low frequency |
| 24 | Misc integer/relationship-range edge cases | 25 | – | – | No | S each | Isolated, low priority |

Not ranked above: `mixing UNION and UNION ALL` (6) and `UNION branches with
different result columns` (7) were checked against the Cypher spec and are
correct rejections, not bugs — see the note at the end of Cluster details.
`fixture-setup-failure` (57) is not a separate defect; it is bare-parse-error
(#7-adjacent grammar gaps) hitting donor setup statements instead of the
query under test, and disappears as grammar work lands.

## Cluster details

**1. Read/mutation dual-binder architecture.** `graph/frontend/src/binder.rs`
implements two structurally separate binders: `bind_query`/`bind_read_clauses`
(lines ~258–341) for read-only queries, and `bind_mutation`/
`bind_mutation_query` (lines 197, 430) for anything containing `CREATE`,
`MERGE`, `SET`, `REMOVE`, `DELETE`, or `FOREACH`. The harness tries the read
path first; when a query mixes read and mutating clauses, the read binder
correctly declines with `"mutation clauses in read queries is not supported
in the initial graph slice"` (this exact phrase appears in 109 messages), and
the harness retries via the mutation binder, which reports its own,
independent failure. Representative: `age.cypher.create.query-40` — `Parse
error: mutation clauses in read queries is not supported in the initial
graph slice at byte 49..73; mutation execution failed: Cypher mutation
binding failed: duplicate variable \`X\`...`. Breaking down what the mutation
binder actually fails on for those 109 rows: `no such table: q` /
`no such column: bN` (30 rows — the mutation binder's SQL lowering emits a
reference to a table/column alias that was never created, a genuine lowering
bug, not a "not implemented" stub), `labels or properties on an
already-bound CREATE node` (15), `duplicate variable` (11), `mutation query
contains no mutation clauses` (9), `unknown variable` (8), `relationship
creation without exactly one type` (7), and a dozen smaller one-off
limitations, all under the same `"...is not supported in the initial graph
slice"` banner, which is this feature's own name for its current phase.
Feature frequency: high — this is the largest single directly-attributable
bucket, and its two-attempt architecture is also why almost every
"unsupported" and "missing-function" message in this report is doubled up
(`query execution failed: ...; mutation execution failed: ...`). Fix
complexity: L — this is a binder redesign (a single binder that recognizes
read and mutation clauses in the same clause sequence), not a patch.
Isolation: cross-cutting across all of `binder.rs`'s mutation path and the
lowering it feeds. Similarity: shares surface with #21 (duplicate/unknown
variable, mostly a symptom of this same split) and #16 (side-effect
accounting, which lives downstream of the same mutation binder).

**2. Entity-introspection functions restricted to bound pattern variables.**
`binder.rs` lines ~3221 and ~3283 special-case `labels`/`type`/`label`/
`properties`/`nodes`/`relationships`/`startNode`/`endNode`/`length` only when
their argument is a literal bound pattern variable or a `Path`-typed value.
Representative: `type;` fails 23 times across tck/grafeo/age/sparrowdb
whenever the argument is anything else (a list element, a `WITH`-projected
alias, a nested expression). Rolling up `type`(23) + `label`(11) +
`properties`(9) + `startNode`(6) + `start_id`(6) + `end_id`(6) + `endNode`(5)
+ `nodes`(5) + `relationships`(5) + `labels`(4) + `id`/`size`/`head`/`last`(4)
+ `all_shortest_paths`/`shortestPath`/`shortest_path`(14) totals ~98 rows.
Feature frequency: high — these are core Cypher entity-inspection builtins,
not donor-specific. Fix complexity: M — needs the argument-matching logic in
`binder.rs` to accept any expression that *type-checks* as Node/Relationship/
Path, not just literal pattern variables. Isolation: contained to
`binder.rs`. Similarity: overlaps #6 (procedures) in spirit — both are
"binder recognizes a fixed vocabulary by exact shape" limitations.

**3. Temporal `ORDER BY` not instant-normalized.** `graph/frontend/src/
lowering.rs::lower_ordering` (line 835) builds the SQL `ORDER BY` clause
directly from each key's lowered expression without normalizing
temporal-typed keys to a comparable absolute instant first. Representative:
`tck.clauses.with-orderby.withorderby1.scenario-15` — expected
`[["12:35:15+05:00"], ["12:30:14.645876123+01:01"], ...]`, observed
`[["10:35-08:00"], ["12:30:14.645876123+01:01"], ...]` — the sort order
depends on the wall-clock/offset text rather than the UTC instant each value
denotes. This single defect accounts for 113 of the 501 result-mismatch
rows, making it the single largest result-mismatch sub-cluster. Feature
frequency: high (any query that sorts `time`/`datetime` values). Fix
complexity: M — `lower_ordering` needs to route temporal-typed order keys
through an instant/epoch-comparable projection, most naturally by calling
into `graph/temporal`'s existing extension functions (e.g. a new
`temporal_instant`-style accessor) instead of comparing the rendered text.
Isolation: contained to `lowering.rs`. Similarity: none of the other
temporal clusters (#which covers `duration.between` formatting) share this
root cause — this is a comparator problem, not a rendering problem.

**4. AGE jsonb operator syntax.** `graph/testkit/src/age.rs` (`AGE_SPECIFIC_
FILES`, lines ~68–77) excludes `jsonb_operators.sql` from the corpus
entirely; replaying the harness's own extraction regex against that file
counts 159 genuine `cypher()`-wrapped invocations of postgres jsonb operators
(`?`, `?&`, `?|`, `->`, `->>`, `#>`, `#>>`, `||`, `@>`, `<@`) applied to
Cypher values — e.g. `MATCH (n) return n ? 'list'`
(`graph/testdata/donors/age/sql/jsonb_operators.sql:659`). Core dependency:
**partial**. `core/function.rs` already has a complete JSON1-style surface
(`json_extract`/`jsonb_extract`, `json_patch`/`jsonb_patch`, etc.), so `->`,
`->>`, `#>`, `#>>`, and existence-check `?` can lower onto those directly —
no core dependency. Deep/recursive containment (`@>`, `<@`) has no home:
core only has single-level `array_contains`/`array_contains_all`, no
recursive JSON-containment primitive, and building one in the frontend runs
into the same recursive-CTE wall as `reduce()` (#7). Fix complexity: S–M
for the direct-mapping operators, M–L for containment. Isolation: contained
to grammar (new operator tokens) + binder (operator→function dispatch) +
lowering. Similarity: shares the "postgres syntax layered onto Cypher
grammar" shape with #8 (pgvector) and #19 (EXPLAIN), and the
recursion/core-primitive gap with #7 (reduce).

**5. Generic missing/misnamed function surface.** A long tail of ~15
distinct function names that either have no core SQL implementation at all
or exist in core under a different name that `rewrite_builtin_call`
(`binder.rs` line 4068) doesn't know about. The clean, cheap win inside this
bucket: `percentileCont`/`percentileDisc` (15 rows combined) already exist in
core as `percentile_cont`/`percentile_disc` — this is a two-line addition to
the rewrite table, S complexity, isolated. The rest (`collect` outside a
top-level aggregate position, 20; `split`, 16; `cot`, `stDev`, `stDevP`,
`localtime`, `localdatetime`, `isEmpty`, `toBoolean`/`toFloat`/`toInteger`/
`toIntegerList`/`toString`/`toUpper`/`toLower`, `left`/`right`/`tail`, `now`,
`elementId`, `isNotNull`, `degree`, `timestamp` — each 1–4 rows) need either
a new core scalar function or a rewrite-table entry, evaluated per function.
Feature frequency: medium (individually low-count, collectively large).
Isolation: independent per function, no shared blocker. Similarity: `collect`
outside a top-level aggregate position is really the same defect as #2's
argument-matching restriction, just for aggregates instead of
entity-introspection functions — the binder's `bind_aggregate_call`
(`binder.rs` ~2508) only recognizes `count`/`sum`/`avg`/`min`/`max`/
`collect` as the *direct* top-level projection expression, not nested inside
another call.

**6. Procedures / `CALL` outside the hardcoded registry.** `binder.rs::
bind_call` (line 362) matches only `db.labels`/`db.relationshipTypes` by
literal string comparison (lines 367–373); every other procedure name
(`db.schema`, AGE/sparrowdb catalog procedures, user-defined procedures)
fails with the same `"procedures outside the built-in registry"` message —
67 rows (tck 24, age 22, sparrowdb 20, grafeo 1). Feature frequency: medium —
procedures are a named Cypher construct but a narrow slice of real queries
use them. Fix complexity: M — needs an extensible procedure registry rather
than a string match. Isolation: contained to `binder.rs::bind_call`.
Similarity: same "closed vocabulary" shape as #2.

**7. `reduce()`.** Verified two-stage blocker, not one. First: the grammar
has no reducer-expression production at all — `graph/cypher/src/cypher.pest`
line 133 (`postfix_expression = { primary_expression ~ postfix_suffix* }`)
and line 134 (`postfix_suffix = { property_suffix | index_suffix |
cast_suffix | call_suffix }`) parse `reduce(...)` as an ordinary function
call; the `acc = init, x IN list |` reducer syntax has no `|`-continuation
rule anywhere in the grammar. This is why every `reduce()` query in the
corpus fails identically with `expected AND, OR, xor_op, comparison_op,
predicate_suffix, additive_op, multiplicative_op, power_op, or
postfix_suffix` at the exact byte offset of the `|` character (confirmed
against `graph/testdata/donors/grafeo/tests/spec/lpg/cypher/expressions.
gtest:256`, query `MATCH (n:N) RETURN reduce(acc = 0, x IN [1,2,3,4] | acc +
x) AS r`, byte 50 = the `|`). This grammar gap alone accounts for 76 of the
370 bare-parse-error rows and is entirely independent of the second blocker.
Second, once parsed: `reduce()` is a sequential fold over a list, and the
natural lowering strategy (recursive accumulation) needs `WITH RECURSIVE`,
which turso core rejects outright — confirmed empirically (`WITH RECURSIVE
cnt(x) AS (...) SELECT * FROM cnt;` against `./target/debug/tursodb -q`
returns `Parse error: Recursive CTEs are not yet supported`) and in source
(`core/translate/planner.rs`, two `bail_parse_error!("Recursive CTEs are not
yet supported")` sites, ~line 750 and ~line 1382). Core dependency: **yes**
for the general case. A bounded-unroll fallback (statically unrolling the
fold when the list argument is a literal-length list expression, not an
arbitrary runtime-sized column) needs no core work and would pass a real
subset — most TCK/donor `reduce()` examples use short literal lists. Fix
complexity: S for the grammar gap (frontend-only, unblocks parsing
immediately regardless of the rest); L for full lowering (blocked on core);
S–M for the bounded-unroll subset. Isolation: grammar fix is contained;
general lowering is cross-cutting on a core capability that doesn't exist
yet. Similarity: shares the deep-recursion core gap with #4's `@>`/`<@`
containment operators.

**8. pgvector casts/operators.** Verified core dependency: **no**. `core/
function.rs` lines 347–359 register a complete vector surface: `vector`,
`vector32`, `vector32_sparse`, `vector64`, `vector8`, `vector1bit`,
`vector_extract`, `vector_distance_cos`, `vector_distance_l2`,
`vector_distance_jaccard`, `vector_distance_dot`, `vector_concat`,
`vector_slice`. `jsonb_operators.sql`'s sibling `pgvector.sql` (excluded the
same way, 71 genuine `cypher()`-wrapped invocations by the same regex
replay) uses `cosine_distance(...)` calls and `[1.22,2.22,3.33]::vector`/
`::halfvec`/`::sparsevec` casts (`graph/testdata/donors/age/sql/
pgvector.sql:26-47`) — this is a pure mapping exercise: grammar support for
the `::vector`-family cast syntax and the `<->`/`<=>`/`<#>` distance
operators, a binder rule mapping cast target and operator to the matching
core vector constructor and `vector_distance_*` function, and lowering to
emit that call. Fix complexity: S–M. Isolation: contained to
grammar+binder+lowering, no core work. Open question, not a blocker:
fidelity mapping between pgvector's three vector subtypes and core's five
(`vector32`/`64`/`8`/`1bit`/`sparse`) needs an explicit table, since they
don't line up 1:1 on precision.

**9. Read-path binder semantic long tail.** About 15 distinct, mostly
independent gaps in `bind_read_clauses`, none individually large: arithmetic
on non-numeric operands (17), optional variable-length relationships (13),
reusing a non-node/non-relationship variable in a pattern (16 combined),
indexing this operand/key combination (12), boolean operators on
non-boolean operands (12), star arguments outside aggregating projections
(10), variable-length path values (8), `CALL` subqueries after other
clauses (5), pattern expressions in projections (3), multiple `OPTIONAL
MATCH` paths (2), unaliased `WITH` expressions (2), `IN` against a non-list
operand (2), slicing a non-list operand (2) — ~101 rows aggregate. Feature
frequency: individually low, collectively a real long tail. Fix complexity:
S–M each. Isolation: each is its own clause/operator-type check, largely
independent of the others and of #1.

**10. Nested list-comprehension outer-scope variable capture.** 33 rows,
`"outer list-scope variables inside nested list scopes"` — one list
comprehension nested inside another (`[x IN list1 | [y IN list2 WHERE
y > x | y]]`) fails to resolve the outer comprehension's bound variable
inside the inner one. `lowering.rs` lowers comprehensions to
`json_each`-based subqueries (lines ~1459–1492: `SELECT
json_group_array({element}) FROM json_each(({list})) AS {alias} WHERE
({filter})`); nesting two of these needs the inner subquery to see the outer
alias, which the current lowering doesn't wire through. Fix complexity: M.
Isolation: contained to `lowering.rs`'s comprehension lowering.

**11. OPTIONAL MATCH / UNWIND null propagation.** Two related null-handling
bugs in the same file. `Match7.feature`'s "optionally matching" scenarios
(44 conformance rows) expect that when an optional pattern fails to match,
*every* variable bound within that pattern becomes null for the row — not
just the specific unmatched part. Representative:
`tck.clauses.match.match7.scenario-8` — expected `[["<null>"]]`, observed
`[["(:C)"]]`; `scenario-16` — expected `[["<null>"]]`, observed
`[["{\"nodes\":[2,null],\"relationships\":[null]}"]]` — a partial binding
leaks through instead of the whole pattern collapsing to null. Separately,
CypherBench's 25-row "UNWIND-of-NULL extra row" cluster (e.g. qid
`f62f09d8...`: observed rows have a spurious leading `[""]` row that
expected does not) matches the Cypher spec rule that `UNWIND NULL` must
produce zero rows, not one — the `Unwind` plan-kind lowering (`lowering.rs`
~line 528, a plain inner join against `json_each(list)`) has no NULL guard.
Feature frequency: medium-high, real cross-dataset overlap (this is the
overlap the coordinator specifically asked to have called out). Fix
complexity: M for each. Isolation: contained to `lowering.rs`, two separate
plan-kind arms (`LeftApply`/optional-match and `Unwind`).

**12. Bitemporal interval calculus (`btic*`).** 28 rows across `btic` (10)
and 18 distinct `btic_*` accessor/predicate functions (`btic_lo`, `btic_hi`,
`btic_duration`, `btic_overlaps`, `btic_before`, `btic_intersection`, etc.,
1 row each) — all from the tck `Btic1.feature` suite. This is a donor-suite
extension, not standard openCypher (no such feature exists in the upstream
TCK), so it needs a new value type plus ~19 supporting functions built from
scratch — a large, low/medium-frequency, self-contained feature area rather
than a bug fix. Fix complexity: L. Isolation: an isolated new module, though
worth deferring relative to items that touch real, widely-used Cypher
surface.

**13. Entity literal rendered as raw id after CREATE/MERGE.** 24 rows —
`tck.clauses.create.create3.scenario-6`: expected `[["(:X)"]]`, observed
`[["1"]]`. `graph/frontend/src/mutation.rs` lowers `CREATE` to `INSERT INTO
...  RETURNING <id-column>` (lines ~1299–1305); when the query immediately
`RETURN`s that freshly-created variable, the projection returns the raw
returned id instead of re-rendering it as a node/relationship literal the
way a MATCH-bound variable would be. Fix complexity: S–M — wire the
existing entity-literal rendering path (used for MATCH results) to
mutation-produced bindings too. Isolation: contained, spans `mutation.rs`
and the projection-rendering helper it needs to call.

**14. Boolean value erasure.** CypherBench: 29 rows, e.g. qid
`19b30258-...`: expected `[["false"]]`, observed `[["0"]]`. Conformance:
a smaller echo (`other:[["0", "1"]]`, 4 rows; `other:[["0"]]`, 3 rows,
partial overlap with #23). `graph/CYPHER_CORPUS_GAPS.md` (a prior,
now-superseded corpus snapshot) independently documented the same defect as
"Boolean type erasure" (12 failures at that time): "Boolean values emerge
from relational/list lowering as integers 0 and 1." Feature frequency:
high relative to its fix cost. Fix complexity: S — this is a single
value-rendering boundary that isn't tagging SQL integer 0/1 as Cypher
boolean before display. Isolation: contained to the value-rendering
boundary. Similarity: NOT the same root cause as #23 (NaN comparisons) —
that one returns SQL NULL, this one returns a real 0/1 integer; they only
look similar in truncated message previews.

**15. Runtime-error mechanism for TypeError/EntityNotFound.** 40 tck-only
rows, all `"expected an error but execution succeeded"` — e.g.
`tck.clauses.return.return2.scenario-15` ("Fail when returning properties of
deleted nodes") and `scenario-17` (deleted relationships), which the TCK
expects to raise `EntityNotFound: DeletedEntityAccess` at runtime. Plain SQL
execution semantics have no way to raise a typed Cypher runtime error — the
underlying row is simply gone or the value is simply NULL, and SQL either
silently returns that or executes without complaint. Core dependency: no.
Proposed mechanism, following the exact pattern `graph/temporal/src/lib.rs`
already uses (`install_temporal_extension`, registering scalar functions via
`Connection::register_static_extension` + `ExtensionApi::
register_scalar_function`): register an error-raising scalar function (or
family — one per spec error kind) that the binder/lowering can invoke at the
point spec mandates a check, which raises a distinguishable SQL error the
frontend then reports as the correctly-typed Cypher runtime error. This
directly unlocks the 40 measured rows and, once wired to real invariant
checks, additional TypeError-family scenarios that currently fail via other
buckets (bare-parse-error or missing-function) before ever reaching
execution. Fix complexity: M — designing which checks need this per
construct is the real work; the mechanism itself mirrors an existing
pattern. Isolation: contained, new static-extension functions plus binder/
lowering call sites.

**16. Side-effect accounting mismatch.** 23 rows, tck only, e.g.
`tck.clauses.merge.merge1.scenario-14`: `side effect +nodes expected 1,
observed 0`; `tck.clauses.set.set1.scenario-1`: `side effect +properties
expected 1, observed 0`. The mutation executes and (per other passing
assertions) affects the graph, but the harness's side-effect counters
under-report what changed. Fix complexity: M. Isolation: contained to
`mutation.rs`'s side-effect bookkeeping. Similarity: downstream of #1's
mutation path, same file family.

**17. AGE catalog-admin functions.** 50 rows (`vertex_stats` 21,
`delete_global_graphs` 14, `is_valid_label_name` 8, `graph_stats` 7) — these
are AGE's own graph-catalog administration surface, not Cypher language
features. Feature frequency: low real value — flagged for exclusion from
active scoring rather than fix priority, though mechanically S complexity
if pursued (thin wrappers over catalog metadata already available to the
frontend).

**18. sparrowdb vector/FTS surface.** 17 rows: `vector_similarity` (2),
`vector_distance` (1), `vector_dot` (1) — same shape as #8, map onto core's
existing `vector_distance_*` functions; `full_text_search` (8) and
`hybrid_search` (5) have no equivalent core primitive today (core has no
FTS/BM25 surface in `core/function.rs`), so this half has a real,
uninvestigated core dependency. Fix complexity: M.

**19. postgres `EXPLAIN` forms.** Currently contributes 0 measured rows —
not because these donor rows pass, but because `graph/testkit/src/age.rs`
strips any `cypher()` invocation whose query text starts with `explain`
(case-insensitive, lines ~117–124) before test generation. Replaying the
harness's extraction regex counts 60 such invocations across the
non-excluded AGE files. Core dependency: **no** — `core/connection.rs`,
`core/vdbe/builder.rs`, `core/vdbe/mod.rs`, and `core/statement.rs` already
implement `EXPLAIN`/`EXPLAIN QUERY PLAN` in full. What "pass" means for
these donor rows: execution-success only, not plan-text matching — the
donor expectations are postgres planner text, which is meaningless against
turso's own plan format. `graph/testkit/src/model.rs`'s `Expectation` enum
(`Rows`/`Error`/`Unsupported`, lines 21–25) has no "executes without error,
ignore output" variant today, so a fourth variant is a real prerequisite.
Fix complexity: S–M — add the `Expectation` variant, prefix lowered SQL
with `EXPLAIN`/`EXPLAIN QUERY PLAN` in the frontend, return core's plan
output, then remove the current per-invocation skip in `age.rs`. Isolation:
contained to testkit + a small frontend addition, no core work.

**20. `graph IR invariant failed: duplicate binding name`.** 14 rows (tck 3,
age 11), e.g. `tck.clauses.match.match4.scenario-8`: `graph IR invariant
failed: duplicate binding name: rs`; `age.cypher.vle.query-59` through
`-65`: `duplicate binding name: p`, all from variable-length-relationship
queries. Unlike the "X is not supported" family, this is an internal
assertion failing — the IR believes two distinct bindings share a name,
which is either a real name-generation collision in variable-length-path
construction or an overly strict invariant. This needs root-cause
investigation before a fix, not just a "reject cleanly" patch, since an
assertion failure on a code path that should be reachable is a correctness
concern, not a missing-feature one. Fix complexity: M. Isolation: contained
to IR construction for VLE patterns.

**21. Legal variable reuse (MATCH→CREATE) rejected as duplicate/unknown.**
11 rows primary-categorized as `duplicate-variable` (age), 8 as
`unknown-variable` (tck 5, age 3), with a larger population (26/43
respectively) appearing as the second-half error when the read binder
rejects the query first for containing mutation clauses (see #1). Cypher
legally allows `MATCH (n) CREATE (n)-[:X]->(m)` to reuse `n` from the
preceding `MATCH`; the mutation binder currently treats that reuse as either
a duplicate declaration or, in the reverse direction, an unresolved
reference. Fix complexity: S–M. Isolation: mostly resolves as a byproduct
of #1's binder unification, since both are symptoms of the mutation binder
not sharing scope with the preceding read clauses.

**22. Unsupported UTF-16 surrogate-pair unicode escapes.** 6 rows, all
`age.scan.query-*`: `invalid unicode escape \`\uD835\`` /
`\`\uDEF0\``. Cypher string literals allow UTF-16 surrogate pairs
(`𝛰` encoding a single supplementary-plane codepoint); the lexer
currently rejects each half individually instead of combining a high/low
surrogate pair into one codepoint. Fix complexity: S. Isolation: fully
contained to the string-literal lexer, no interaction with any other
cluster.

**23. NaN comparison returns NULL instead of definite bool.** 7 rows:
`tck.expressions.comparison.comparison1.scenario-8` (4 examples) and
`comparison2.scenario-5` (3 examples). Per spec, `RETURN 0.0/0.0 = 1 AS
isEqual, 0.0/0.0 <> 1 AS isNotEqual` must yield `isEqual = false, isNotEqual
= true` (NaN compares false to everything via `=`, `<`, `>`, etc., and true
via `<>`, even against itself) — a Cypher-specific rule, not standard SQL
NULL propagation. Observed: both come back NULL. Root cause: SQLite has no
representable NaN float — `0.0/0.0` evaluates to SQL NULL, not a
distinguishable NaN value, so NULL then propagates through the comparison
the ordinary SQL way instead of the comparison operator special-casing NaN
per Cypher's rules. Fix complexity: S, but the exact lowering site for
comparison operators was not pinned down further (would need either
representing NaN as a real `f64::NAN` that survives division, or special-
casing NaN detection at the comparison-operator lowering). Feature
frequency: low (NaN literals are rare in real Cypher). Isolation: contained,
low priority given the count. Not the same defect as #14 (see that entry).

**24. Misc integer/relationship-range edge cases.** 25 rows: relationship
range outside the supported u32 range, invalid relationship range (e.g.
`2..1`), integer literal outside the supported i64 range, a handful of
unknown-parameter donor artifacts. Each is a narrow bounds-check gap. Fix
complexity: S each. Isolation: fully independent, low priority.

**Assessed as spec-correct, not defects:** `mixing UNION and UNION ALL in
one query` (6 rows) and `UNION branches with different result columns`
(7 rows) were checked against the Cypher spec's UNION rules (`binder.rs`
lines 262–271 mixed-UNION rejection, line 4791 `assemble_union_branches`
with the column-count check at line 4811) and match real spec requirements.
These are donor queries that are themselves invalid Cypher, not gaps in the
frontend — they are intentionally excluded from the backlog above.

## Feature gaps requiring larger investment

These four are the reclassified items from the prior "excluded/unfixable"
list. They are ranked in the queue above (positions 4, 7, 8, 15, 19 — jsonb
splits across two queue-relevant facets); this section is their deep-dive
companion, not a separate exclusion list.

- **AGE jsonb operators** (queue #4) — partial core dependency, real work
  only for deep containment (`@>`/`<@`); everything else maps directly onto
  existing core JSON functions.
- **`reduce()`** (queue #7) — the grammar gap is a same-day S fix
  independent of everything else; full sequential-fold lowering is
  genuinely blocked on core recursive CTEs (confirmed by direct empirical
  test against `tursodb` and by source in `core/translate/planner.rs`), with
  a bounded-unroll fallback available today for literal-length lists.
- **pgvector** (queue #8) — corrected down from an assumed core dependency:
  turso core already ships the full vector type/function surface
  (`vector32`/`64`/`8`/`1bit`/`sparse`, all four `vector_distance_*`
  functions, `vector_concat`/`vector_slice`/`vector_extract`). This is
  grammar+binder+lowering mapping work only.
- **postgres `EXPLAIN`** (queue #19) — corrected down from an assumed need
  for a new plan-rendering path: turso core already implements
  `EXPLAIN QUERY PLAN` in full. This needs a new testkit `Expectation`
  variant plus SQL-prefixing in the frontend, not a core change.
- **Runtime TypeError/EntityNotFound** (queue #15) — no core dependency;
  the proposed mechanism (an error-raising scalar function registered as a
  static extension, following `graph/temporal`'s existing registration
  pattern) is new frontend infrastructure, not new core capability.
