### Task 20: Documentation and gate deletion

Last task of the plan. Everything through Task 19 has landed: relationships
declare named **roles** rather than a fixed start/end pair, roles carry target
types and cardinality, `Many` roles spill to a side table, and a relation may
itself be a role's player.

Every claim below was **measured against the tree**, not read off the plan.
The plan's line references for the spec file are accurate — unusually for this
plan, I checked all nine and every one is right. But the plan is **incomplete
in five ways and impossible in one**, listed below. **Where the plan and this
brief conflict, this brief governs.**

**Files:**
- Modify: `docs/graph.md`
- Modify: `graph/CONFORMANCE.md`
- Modify: `.specs/graph-semantic-schema-overlay.agent-spec.md`
- Modify: `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`

Line numbers shift as you edit. Either work bottom-up or re-locate each site by
its text; do not trust a cited number after your first edit to the same file.

---

## Plan defects

1. **`cargo test -p turso_cypher` names a crate that does not exist.** The graph
   crates are `turso_graph_ir`, `turso_graph_frontend`, `turso_graph_runtime`,
   `turso_graph_cypher`, `turso_graph_temporal`, `turso_graph_testkit`. Use
   `turso_graph_cypher`.

2. **Step 5 is impossible as written.** It says to run each documented example
   through `cargo run -q --bin tursodb -- -q`. There is no Cypher in the CLI at
   all — `rg -n "cypher" cli/` returns zero hits — and every example in
   `docs/graph.md` is **Rust API** code, not CLI input. See "Verifying the
   examples" below for what to do instead.

3. **The Quickstart is already broken and the plan never mentions it.**
   `docs/graph.md:46-54` builds a `RelationshipSourceRegistration` literal with
   `start_column` / `end_column` / `start_node_source` / `end_node_source`
   fields. That struct is now (`graph/frontend/src/catalog.rs:47-53`):

   ```rust
   pub struct RelationshipSourceRegistration {
       pub name: String,
       pub table: String,
       pub identity_column: String,
       /// Declaration order is stable and becomes role ordinal order.
       pub roles: Vec<RoleSourceRegistration>,
   }
   ```

   The documented literal **does not compile**. There is a `binary(...)`
   convenience constructor at `catalog.rs:60` whose own doc comment states the
   invariant this plan is built on: "a two-endpoint table registered as a
   two-role relation named `start`/`end`. This is a layout of the role model,
   not a separate kind." Fixing the Quickstart is part of this task.

4. **A third stale foedus reference the plan omits.** The plan repoints
   `.specs/graph-semantic-schema-overlay.agent-spec.md:170` and `:276`. The same
   stale path also appears at
   `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md:45`.
   Repoint all three.

5. **The plan's list of binary-language sites is incomplete.** It names lines
   38, 54, 94, 126, 145 for rewriting and 245 for deletion. These further sites
   in the same spec file now assert things that are **false**, and the plan does
   not list them:

   - `:321` — "binary endpoints are validated before mutation"
   - `:362` — lists "native n-ary relations, or relation-to-relation roles" among
     what the system does **not** represent. It now represents both.
   - `:452` — "Slice 2.6: validate binary endpoints"
   - `:557` — "n-ary relations ... require their named ADR decision gate"

   Re-scan for others rather than treating my list as complete: `rg -n -i
   "binary|n-ary|nary|start/end|endpoint"` over both documents and judge each
   hit. Report any site you decided to leave alone and why.

6. **Do not use `git add -A`.** Stage explicit paths.

---

## Step 1: Delete Decision Gate B

`.specs/graph-semantic-schema-overlay.agent-spec.md:245` opens "Decision gate B
— native n-ary storage (narrowed)". Determine its extent from the document
structure — do not assume a line count. Its body runs at least through the
"Defer native storage. Current IR and storage are binary:" passage at `:278`.

Replace the whole gate with a one-line note recording that it was resolved by
native n-ary relationships, pointing at
`docs/superpowers/specs/2026-07-25-native-nary-relationships-design.md`.

The gate is resolved **by deletion, not by rewording**: there is no binary code
path left to narrow. Delete the Global Constraint at `:101` that forbids native
n-ary (it claims n-ary "need[s] no native support: they compose ... by
catalog-level reification"), and rewrite the binary-endpoint language at `:38`,
`:54`, `:94`, `:126`, `:145` in role terms — plus the sites in defect 5.

Do the same for `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`
lines `:21` and `:1680`.

## Step 2: Repoint the archived foedus spec reference

Change `foedus/docs/superpowers/specs/2026-07-23-turso-ontology-store-design.md`
to `foedus/docs/superpowers/specs/2026-07-25-turso-ontology-evolution-design.md`
at all **three** sites (defect 4).

## Step 3: Refresh the conformance number

`graph/CONFORMANCE.md:44` reads `- **8,919 passed**;`. Update it from **your
own** `mise run corpus` in Step 6, and record the `run_id` alongside, matching
the format already in the file.

**Do not record a bare total.** The `tck-deep` suite flakes ±2 across identical
commits, so the total moves without any code changing — which is exactly how
the plan came to assert a fixed "8,926" that is not a real number. Record the
per-suite breakdown beside whatever total you write, and note the tck flake
band, so the next person can tell a real regression from a flake.

## Step 4: Document the role model

Add a Roles section to `docs/graph.md`. Its existing headings are Quickstart,
Open modes and Core seams, Optional semantic schema, The session API,
Transactions, Composing frontends, Reference — place it where it reads
naturally, and fix the Quickstart per defect 3.

Cover:
- a relation declares named roles; each role has target types, optionality, and
  cardinality
- **binary is the two-role layout named `start`/`end`, not a separate kind** —
  this is the plan's central invariant and the document should say it plainly
- the standalone pattern `[x:T {props}](role: player, …)`
- the arrow forms as sugar over it
- the role-edge read sugar and its ambiguity rule
- `SET [x](role: player)` **replacing** rather than appending for many-valued
  roles
- the requirement to name a role pair for variable-length traversal past arity 2
- that a relation may be a role's player, and that a role's target list decides
  whether it may

Write from what the code does, not from the plan's prose. `graph/frontend/tests/nary_relations.rs`
is the behavioral record for most of the above and is the best source for exact
syntax and exact refusal wording.

## Step 5: Verifying the examples

The requirement behind the plan's impossible CLI step is real: **every example
must actually run.** Since the examples are Rust API code, verify by compiling
and running them, not by eyeballing.

Pick whichever fits the document: mirror each example as a test in
`graph/frontend/tests/`, or check each against an existing passing test that
performs the same calls. State in your report, for each example you add or
change, the specific test or run that proves it works. "It matches the pattern
used elsewhere" is not proof — a compiled, executed example is.

If an example cannot be made to run, fix the document, not the output.

## Step 6: Gate and commit

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher
mise run corpus
mise run cypherbench-sample
```

Run the clippy command **exactly as written**. Five implementers on this plan
have substituted a narrower `-p <package>` form, hit two pre-existing `core/`
unused-import warnings, and reported the gate as broken. The literal workspace
form exits 0. If you believe it fails, paste the literal command and its exit
code.

`mise run corpus` is known to exit 1 with "task failed" even when every suite is
at baseline — read the per-suite numbers, do not trust the exit code.

The corpus gate is **per suite, never a total**: `age-deep` 3042, `cqlite-deep`
113, `grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly** at baseline;
`tck-deep` within **3329-3332**. This task is documentation, so any non-`tck`
suite moving off baseline means you changed behavior somewhere — stop and report
BLOCKED with the suite and the delta.

`git add` with **explicit paths**, `git commit -S`, and commit **code and docs
only** — nothing under `graph/test-results/`, which the controller commits
separately.
