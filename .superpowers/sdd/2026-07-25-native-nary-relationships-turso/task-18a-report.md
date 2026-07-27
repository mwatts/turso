# Task 18a report: execution-time player validation

**Status: NEEDS_CONTEXT — stopped before implementing. No commit made; tree is
clean at `ae795a64c3434c787ab906538168319b9bcfa5d6` (the branch tip when I
started).**

## Summary of the blocker

Task 18a's only real deliverable, per the split ruling, is: (1) a
`MutationError::RolePlayerTypeViolation` check wired into `insert_relationship`
(before `insert_entity`) and into the `ir::Mutation::SetRoles` executor arm,
and (2) a test proving it — specifically the brief's second test, a
`$who`-parameterised role player of the wrong type refused before any write,
plus (per Correction G) a companion test that a parameterised player of the
*correct* type still succeeds.

I verified, with a real `cargo test` run (not by reasoning from the source
alone), that **a query parameter cannot be used as a role player in the
current tree at all** — `CREATE [x:Transcription](scribe: $who, text: t,
folio: f)` fails at *bind* time, not execution time, with:

```
Err(Unsupported { feature: "a role player that is not a bound variable", span_start: 59, span_end: 63 })
```

This makes both of 18a's required tests unwritable as specified, and makes the
`check_role_target` mechanism itself unreachable by any *currently existing*
Cypher surface — so there is nothing for it to be validated against without
also adding parameter-player support, which is out of the scope the
corrections gave me (`mutation.rs` only; `binder.rs` is reserved for 18b's
MERGE routing).

## How I verified it (not guessed)

`graph/frontend/src/binder.rs:1756` (`bind_role_player`, shared by
`bind_create_role_pattern` and `bind_set_roles`) is:

```rust
fn bind_role_player(
    &self,
    relationship_type: &str,
    role: &crate::semantic::SemanticRole,
    argument: &cypher::RoleArgument,
) -> Result<ir::BindingId, BindError> {
    let cypher::Expression::Variable(name) = &argument.player.value else {
        if matches!(argument.player.value, cypher::Expression::Literal(cypher::Literal::Null)) {
            return Err(BindError::RoleTargetTypeViolation { ... });
        }
        return Err(at_unsupported(argument.player.span, "a role player that is not a bound variable"));
    };
    ...
}
```

`cypher::Expression::Parameter(String)` is a distinct AST variant from
`Expression::Variable(String)` (`graph/cypher/src/ast.rs:332-335`), and
`role_argument = { identifier ~ ":" ~ expression }` (`cypher.pest:102`) already
accepts either as the value — so `$who` parses fine, but `bind_role_player`'s
`let ... else` only special-cases `Variable`; anything else (including
`Parameter`) falls into the generic `at_unsupported` branch, never reaching
`resolve_binding` or the target-type check at all.

To confirm this is really what happens (not just what the code implies), I
added a temporary probe test in `graph/frontend/tests/nary_relations.rs`
against `RoledCatalog` (the hand-rolled catalog in that file with real
`scribe -> Person` / `text -> Text` / `folio -> Folio` target constraints):

```rust
#[test]
fn probe_parameter_role_player_bind_behavior() {
    let error = bind_role_pattern_query(
        "MATCH (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribe: $who, text: t, folio: f)",
    );
    panic!("probe result: {error:?}");
}
```

Ran: `cargo test -p turso_graph_frontend --test nary_relations probe_parameter_role_player_bind_behavior -- --nocapture`

Actual output:

```
thread 'probe_parameter_role_player_bind_behavior' panicked at graph/frontend/tests/nary_relations.rs:467:5:
probe result: Err(Unsupported { feature: "a role player that is not a bound variable", span_start: 59, span_end: 63 })
```

This is a `BindError::Unsupported`, produced during *binding*, before any plan
runs — it is not `RolePlayerTypeViolation` and it never reaches
`insert_relationship`. I then reverted the probe (`git status --porcelain`
shows the tree clean; no trace of it remains).

## Why Correction F does not hold

Correction F says:

> `CREATE` with a role pattern parses and binds today (Task 13a), so this test
> is fully reachable.

That is true for the *role-pattern surface in general* (a `Variable` player
binds and checks fine — that's what Task 13a shipped, and the existing 19
tests exercise it). It is not true for a *parameterised* player specifically:
`bind_role_player` categorically refuses anything that is not a bound
`Variable` (or a `null` literal), so `$who` is refused before the binder even
looks at its declared type. The brief's second test and Correction G's
companion test both depend on a parameter reaching `insert_relationship` as a
role player — neither can be written against the current tree.

## Why this is not a small fix within 18a's given scope

Making `$who` (or any parameter) a legal role player is not just a
`bind_role_player` tweak. `ir::RoleBinding` is:

```rust
// graph/ir/src/role.rs:51
pub struct RoleBinding {
    pub role: RoleId,
    pub value: BindingId,
}
```

`value` is strictly a `BindingId` — a reference into the read plan's bound
scope. There is no way to represent "resolve this role's player from a named
query parameter at execution time" without one of:

1. A new IR shape for `RoleBinding.value` (e.g. an enum of `Binding(BindingId)
   | Parameter(String)`), touching `graph/ir/src/role.rs`,
   `graph/frontend/src/binder.rs` (both `bind_create_role_pattern` and
   `bind_set_roles`), `graph/frontend/src/lowering.rs`, and
   `graph/frontend/src/mutation.rs` (every place that currently does
   `values.get(&binding.value)`); or
2. A synthetic scope `Binding` for the parameter registered during bind
   (skipping the target-type check for it specifically, deferring to
   execution), plus new plumbing so `values: HashMap<BindingId, Value>` —
   which today is populated *exclusively* by decoding the read plan's row
   output (`decode_mutation_rows`, `mutation.rs:126-168`) — also gets seeded
   from the query's `Parameters` map before mutations run.

Either path is a binder + IR (+ possibly lowering) change. The corrections
are explicit that `binder.rs` is reserved for 18b (MERGE routing only) and
that 18a's files are `mutation.rs` (plus the test). I did not make this change
without checking first, per your standing instruction to flag a correction
that cannot work rather than silently expanding scope around it.

## A further consequence worth flagging

Even setting aside the parameter question: every *currently reachable* way to
fill a role already runs through the same bind-time target-type check —
arrow-form `CREATE`/`MERGE` roles (`binder.rs:1629-1673`, the `start`/`end`
loop) and the role-pattern `CREATE`/`SET` surfaces (`bind_create_role_pattern`
and `bind_set_roles`, both via `bind_role_player`). I could not find a
currently-reachable path where a role player's real type could differ from
what bind time already checked (unlabeled `MATCH (n)` patterns are already
refused at bind time whenever a role has non-empty targets; schemaless roles
have empty targets and skip the check entirely on both sides). So without
parameter-player support, `check_role_target` at execution time would be
correct but **dead code on every currently reachable path** — there would be
no test, including Correction G's "correct-type parameterised player still
succeeds" companion, that can exercise it honestly. This reinforces that the
gap is specifically "parameters can't be role players yet," not "execution
lacks a check bind time already has a hook for."

## What I did not do

- No commits. `git log` still shows `ae795a64c` as branch tip.
- No `check_role_target` function or `MutationError::RolePlayerTypeViolation`
  variant added — adding the error/check machinery without a test that can
  fail without it, and pass with it, would violate "every change needs a test
  that fails without the change."
- No fixture changes (I did confirm, per Correction G's instruction to check,
  that the fixtures I looked at either have empty role targets — schemaless,
  `ternary_session`/`witnessed_session` — or would need a new
  semantic-schema-registered fixture with real `SemanticRoleRegistration`
  targets to exercise a genuine type violation at all; none of the existing
  `graph/frontend/tests/fixture.rs` sessions register one).
- Gates (`cargo fmt`, `clippy`, `mise run corpus`, `mise run
  cypherbench-sample`) not run, since there is no code change yet to gate.

## What I need from you

1. Do you want me to add the minimal binder+IR plumbing to make a query
   parameter a legal (deferred-type-check) role player, so the brief's two
   tests become writable as specified? If so, I'd scope that as its own
   sub-step (touching `graph/ir/src/role.rs`, `binder.rs`, `lowering.rs`,
   `mutation.rs`) before layering `check_role_target` on top — happy to draft
   that plan.
2. Or would you rather redefine 18a's test surface to not require a
   parameterised player (e.g. test `check_role_target` some other way, or
   accept it as forward-compatible dead code for now, landing it as
   preparation for a later task that adds parameter players)?
3. Or is there a reachable path I'm missing that would let me write the two
   tests without touching the binder? I looked for one (see "further
   consequence" above) and didn't find it.

I did not guess past this point because two of your prior corrections on this
plan were wrong and you asked to be told rather than worked around; this is
the same situation — Correction F's reachability claim does not hold against
the tree as it stands, verified by running the actual test, not by inference.
