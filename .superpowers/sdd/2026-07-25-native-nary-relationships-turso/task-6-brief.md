### Task 6: Lower expands through roles

The contract half's first move: lowering stops reading `direction` and reads the
role pair instead. The emitted SQL must not change for a two-role relation.

**Files:**
- Modify: `graph/frontend/src/lowering.rs:1398-1560` (`lower_fixed_expand`)
- Test: `graph/frontend/tests/dialect_alignment.rs`

**Interfaces:**
- Consumes: `RelationshipTableLayout::role` (Task 4), `FixedExpand::role_pair` (Task 5).
- Produces: no new public API. `lower_fixed_expand` is renamed `lower_role_expand` in Task 7, not here.

- [ ] **Step 1: Write the failing test**

In `graph/frontend/tests/dialect_alignment.rs`:

```rust
#[test]
fn role_lowering_emits_byte_identical_sql_for_a_two_role_relation() {
    // Binary is a layout of the role model. If role lowering produces even a
    // different alias or predicate order, every donor query's plan shifts and
    // the corpus number stops meaning what it meant.
    for query in [
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.name",
        "MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.name",
        "MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b.name",
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) WHERE b.age > 30 RETURN b",
    ] {
        assert_eq!(
            lower_to_sql(query),
            expected_binary_sql(query),
            "role lowering changed the SQL for {query}"
        );
    }
}
```

`expected_binary_sql` is a golden map recorded in the same file. Populate it by
running `lower_to_sql` for each query **before** Step 3 and pasting the output:

```bash
cargo test -p turso_graph_frontend --test dialect_alignment -- --nocapture print_binary_sql_goldens
```

Add that printer as a `#[test] #[ignore]` helper in the same file.

That golden is a regression fence, not the red-green driver: it passes before
the change by construction. The behavioural test that must fail first is a hop
a direction-based lowering cannot express at all. Add it in the same file:

```rust
#[test]
fn a_ternary_hop_lowers_through_the_named_role_pair() {
    // Direction-based lowering has only start and end to name, so a
    // scribe -> folio hop is inexpressible: it would silently lower as
    // start -> end and return the text instead of the folio.
    let sql = lower_ternary_to_sql("MATCH [x:Transcription](scribe: s, folio: f) RETURN f.id");
    assert!(sql.contains("scribe"), "the from role must name its own column: {sql}");
    assert!(sql.contains("folio"), "the to role must name its own column: {sql}");
    assert!(!sql.contains("txt"), "the unnamed text role must not be joined: {sql}");
}
```

`lower_ternary_to_sql` lowers a hand-built `ir::RoleExpand` over the three-role
fixture from Task 4, so it does not depend on the surface syntax landing in
Task 12.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test dialect_alignment`
Expected: `a_ternary_hop_lowers_through_the_named_role_pair` FAILS — lowering
still reads `direction` and emits the `start`/`end` columns regardless of the
role pair. `role_lowering_emits_byte_identical_sql_for_a_two_role_relation`
passes, as the fence it is.

- [ ] **Step 3: Replace the direction match with a role match**

In `lower_fixed_expand`, replace the six-arm `(bound_reference, direction)` match
(`lowering.rs:1441-1505`) with:

```rust
    let from_column = layout
        .role(expand.from_role)
        .ok_or(LoweringError::UnknownRole {
            relation: layout.table.clone(),
            role: expand.from_role,
        })?
        .column
        .clone();
    let to_column = layout
        .role(expand.to_role)
        .ok_or(LoweringError::UnknownRole {
            relation: layout.table.clone(),
            role: expand.to_role,
        })?
        .column
        .clone();

    let join_predicate = match (bound_reference, expand.symmetric) {
        (Some(target), false) => format!(
            "{relationship_alias}.{from} = {source_alias}.{identity} \
             AND {relationship_alias}.{to} = {target}",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
        (None, false) => format!(
            "{relationship_alias}.{from} = {source_alias}.{identity}",
            from = quote_identifier(&from_column),
            identity = quote_identifier(&node_identity),
        ),
        // Symmetric: the same relation row matches with the pair in either
        // order. This is the shape today's Direction::Both lowers to, and it
        // is only reachable when both roles target the same node source.
        (Some(target), true) => format!(
            "(({relationship_alias}.{from} = {source_alias}.{identity} \
               AND {relationship_alias}.{to} = {target}) \
              OR ({relationship_alias}.{to} = {source_alias}.{identity} \
               AND {relationship_alias}.{from} = {target}))",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
        (None, true) => format!(
            "({relationship_alias}.{from} = {source_alias}.{identity} \
              OR {relationship_alias}.{to} = {source_alias}.{identity})",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
    };
```

and replace the target-column selection that followed the old match with the
symmetric-aware expression:

```rust
    let target_expression = if expand.symmetric {
        format!(
            "CASE WHEN {relationship_alias}.{from} = {source_alias}.{identity} \
             THEN {relationship_alias}.{to} ELSE {relationship_alias}.{from} END",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        )
    } else {
        format!(
            "{relationship_alias}.{to}",
            to = quote_identifier(&to_column)
        )
    };
```

Add the error variant to `LoweringError`:

```rust
    #[error("relation {relation} has no role {role:?}")]
    UnknownRole {
        relation: String,
        role: ir::RoleId,
    },
```

Index selection needs no separate change: the frontend lowers to SQL, so naming
the role columns in the join is exactly what makes the storage planner key off
the role pair. The per-role and per-pair indexes installed in Task 2 are what it
selects from, and the `bound_target` cycle fold becomes "composite over
(from_role, to_role)" for free because that is the index the pair installer
created.

- [ ] **Step 4: Run to verify the SQL is unchanged**

Run: `cargo test -p turso_graph_frontend --test dialect_alignment`
Expected: PASS. If the golden differs, the difference is the bug — fix the
lowering to match the golden rather than re-recording it.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/lowering: join expands through role columns

Lowering reads the expand's role pair instead of its direction. The six
direction arms collapse to four cases over (bound target, symmetric),
which is the same shape with the endpoint columns named by role.

A golden test pins the emitted SQL for the two-role case so the migration
cannot shift a donor query's plan.

Tests: dialect_alignment goldens; corpus at 8,926; cypherbench at baseline."
```

---

