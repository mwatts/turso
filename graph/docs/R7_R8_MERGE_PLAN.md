# Plan: R7 MERGE Many multiset + R8 concurrent MERGE

| Field | Value |
| --- | --- |
| Status | Plan only (not implemented) |
| Depends on | PR-F (R16) landed on branch |
| Review | `graph/GRAPH_FRONTEND_DEEP_REVIEW.md` R7, R8, PR-G |
| Open questions | §8.1 (multiset vs subset), §8.2 (empty MERGE key) |

## Goals

### R7 — MERGE Many-role match is exact player multiset (or documented subset)

**Today (problem).** Relationship MERGE that touches `Many` roles tends to
match when each named player **EXISTS** in the spill set, not when the spill
multiset **equals** the pattern’s players (count + identity). A relation with
players `{A,B,C}` can therefore match `MERGE ()-[:T {…}]->()` patterns that
only name `{A,B}`, creating or matching the wrong edge.

**Product choice (must decide first).**

| Option | Semantics | Pros | Cons |
| --- | --- | --- | --- |
| **A — Exact multiset (review default)** | Match only if for every Many role, multiset of spill `node_id` equals pattern players | Aligns with “this relationship is exactly these endpoints”; fewer surprise MERGEs | May diverge from AGE/subset corpus rows |
| **B — Subset EXISTS (document)** | Keep EXISTS; document in `docs/graph.md` + IR comments; pin corpus | Smaller code change | Leaves silent under-match as intentional |

**Recommendation:** implement **A** unless a corpus inventory shows AGE/TCK
require subset. If product picks B, still land the doc + pin test (R7 “or
documented subset”).

### R8 — Concurrent MERGE does not double-insert without documenting the race

**Today (problem).** Under two connections each doing `BEGIN IMMEDIATE` + MERGE
of the same pattern, both can fail to see the other’s uncommitted row and both
INSERT → **duplicate relationships**. No physical UNIQUE on MERGE keys; no
`ON CONFLICT` path.

**Product choice.**

| Option | Work | Integrity |
| --- | --- | --- |
| **1 — Fix** | Physical unique key(s) where semantic unique/MERGE key allows + `INSERT … ON CONFLICT` or retry | Strong |
| **2 — Document** | State in `docs/graph.md` that concurrent MERGE of the same pattern can create duplicates until unique indexes land; pin a test that currently *documents* the race or expects a conflict error once fixed | Honest experimental boundary |

**Recommendation:** land **2** immediately if unique indexes are blocked on
partial-index research (review open Q5); schedule **1** when a MERGE key is
expressible as a table UNIQUE (typed single-source first).

## Implementation plan (PR-G)

### PR-G1 — Decision record + tests that fail (R7)

1. Write the chosen semantics in:
   - `docs/graph.md` (short user-facing paragraph)
   - IR / binder comment on relationship MERGE match
   - this file’s “Decision” section (fill in after product pick)
2. Add failing tests under `graph/frontend/tests/` (or lib tests):
   - **Exact multiset (if A):** create relationship with Many players
     `{A,B,C}`; `MERGE` pattern with `{A,B}` must **not** match that row
     (creates a second relationship or no-match create path as Cypher
     requires).
   - **Subset (if B):** same setup must match; assert single row + doc quote.
   - **One-cardinality roles** unchanged: start/end column equality still
     drives match.

### PR-G2 — R7 match implementation (if A)

1. Locate relationship MERGE match SQL in `mutation.rs`
   (`insert_relationship` / merge predicates / spill EXISTS).
2. Replace “each player EXISTS” with:
   - same count: `COUNT(*)` of spill rows for `(relation_id, role)` equals
     pattern length, and
   - every pattern player appears (EXISTS or join), and
   - no extra players (count equality + all present ⇔ multiset if identities
     unique per role; if duplicate players allowed in pattern, use multiset
     compare carefully).
3. Prefer parameterized SQL through `MutationIo` / StatementCache (R16).
4. Keep One-cardinality roles on endpoint columns.

### PR-G3 — R8 documentation + optional unique path

**Minimum (document race):**

1. `docs/graph.md`: concurrent writers may create duplicate MERGEd
   relationships of the same pattern; use a single writer or application-level
   lock until unique indexes exist.
2. Test: two connections, `BEGIN IMMEDIATE`, same MERGE pattern — either:
   - **document mode:** assert two rows exist (current behavior pin), or
   - after fix: assert one row / conflict error.

**Preferred fix when feasible:**

1. For single-source MERGE keys that are already semantic-unique properties,
   install a physical UNIQUE index at constraint registration time (may be
   out of scope if multi-type shared tables need partial indexes — open Q5).
2. MERGE create path: `INSERT … ON CONFLICT DO NOTHING RETURNING` / retry
   match after conflict.
3. Test: concurrent MERGE → one relationship row.

### PR-G4 — Tracker + PERFORMANCE_BACKLOG

1. Mark R7/R8 in `GRAPH_FRONTEND_DEEP_REVIEW.md` §7.0 with commit SHAs.
2. Note any new prepare shapes from multiset SQL in PERFORMANCE_BACKLOG if
   measurable.

## Non-goals for PR-G

- Full mutation-as-`PreparedSource` (open Q4)
- Semantic unique physical indexes for multi-type shared tables (open Q5)
- REMOVE label (R9), OPTIONAL nullability (R10), etc.
- Changing One-role MERGE match (already column equality)

## Verification

```sh
cargo test -p turso_graph_frontend --lib
cargo test -p turso_graph_frontend --test <new_merge_tests>
cargo fmt --check
cargo clippy -p turso_graph_frontend --all-targets -- --deny=warnings
```

Optional after semantic change: `mise run corpus` and compare REPORT per-suite
counts (MERGE-related failures may move under exact multiset).

## Risk notes

- **Corpus:** exact multiset may flip AGE/TCK MERGE rows; inventory before
  flipping default if release pressure is high.
- **Empty MERGE key** (`1 = 1 LIMIT 1`): still open Q2; do not “fix” by
  matching a random row under multi-row sources without a product decision.
- **Integrity:** documenting the concurrent race without a test that names
  the behavior is worse than a pin test that fails when behavior changes
  accidentally.

## Decision (product, 2026-08-10)

| Item | Choice | Date |
| --- | --- | --- |
| R7 multiset vs subset | **A — exact multiset** | 2026-08-10 |
| R8 fix vs document | **1 — fix** (unique enforcement, not document-only) | 2026-08-10 |
| Empty MERGE key | **Fail closed** when match key is empty | 2026-08-10 |
| Shared tables (Q5/R8) | **In scope for R8** — merge-key uniqueness must work when multiple types share a table (type is part of the key; Many multiset is part of the key) | 2026-08-10 |

**Implementation approach for R8 + shared tables:** a catalog merge-key table
keyed by a stable hash of (graph, relationship source, One-role columns,
sorted Many-role player multisets, relationship type names). Concurrent MERGE
of the same key hits UNIQUE on that table and re-matches instead of inserting
a second relationship. This avoids incorrect `UNIQUE(src,dst)` on multi-type
shared tables and covers Many roles that endpoint indexes cannot express.
