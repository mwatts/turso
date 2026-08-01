# AGE error-expectation triage

The testkit expectation-parsing fix (`graph/testkit/src/age.rs`) restored 234 AGE
expectations that a regex bug had been silently swallowing. 130 of those rows
started passing; 104 started failing. Every one of the 104 fails the same way:

```
expectation: "error"   outcome: "failed"
message: "query succeeded but AGE expects an error"
```

So none of them are new breakage — they are rows we were never being graded on.
This file records the per-family verdict: does the engine have a real gap, or is
AGE's error a donor-specific behavior that belongs on the restriction list?

The line drawn below: **openCypher/TCK agrees the query is an error → fix.**
**Only AGE/Postgres says so → restriction.**

Totals: **57 to fix, 47 to reclassify as restrictions.**

The restriction half has landed: corpus 8956 → 9004 of 10242. That is 48 AGE
rows, not 47 — `age.cypher.match.query-173` is a second
`MATCH cannot follow OPTIONAL MATCH` row that was already failing before the
expectation fix, so it never appeared in the 104. No row regressed.

The fix half has landed too: corpus 9004 → 9069 of 10242, no row regressed. The
gain is larger than the 57 because siblings that were already failing came along
with each rule, and because `substring` turned out to be 1-indexed (see below).

Two rows moved between the columns while implementing:

- The int4-range family splits by sign, not by the message. AGE reports
  `substring('abcdef', -2147483649, 0)` with the same "out of INT range" text as
  the positive overflow, but a negative offset is an error openCypher agrees
  with. `is_age_restriction_error` now takes the query text so only the
  non-negative overflows are restrictions.
- `range(0, 10, null)` returns `[0..10]` in AGE: a null step is a missing step.
  Only a null start or end is an error, which is what the donor's own message
  says ("neither start or end can be NULL").

## Fix (57)

### Wrong-typed arguments to scalar functions (44)

The engine does not type-check function arguments; it inherits SQLite's
coercion, so these return a confidently wrong answer instead of a `TypeError`:

| query | engine today | AGE / openCypher |
|---|---|---|
| `abs("1")`, `sin("0")`, … (15 numeric fns) | `1.0`, `0.0` | TypeError |
| `atan2("0", 1)`, `atan2(0, "1")` | number | TypeError |
| `toUpper(true)`, `toLower`, `trim`, `lTrim`, `rTrim` | `"1"` | TypeError |
| `replace("Hello", "e", 1)` | `"H1llo"` | TypeError |
| `toInteger(true)` | `1` | TypeError |
| `size(1234567890)`, `size({...})` | `10` (digit count!) | TypeError |
| `head/last(1234567890)`, `head/last({...})` | `null` | TypeError |
| `tail(123)` | list-ish | TypeError |
| `reverse(true)`, `reverse(3.14)`, `reverse({})`, `reverse(v)` | `"1"` | TypeError |
| `keys([1,2,3])` | `[0,1,2]` (indices!) | TypeError |
| `length(true)` | `1` | TypeError |
| `toBooleanList(123)`, `toFloatList(555)` | `[1]` | TypeError |

`size(1234567890) = 10` and `keys([1,2,3]) = [0,1,2]` are the worst of these:
plausible-looking values that no correct program wants.

Implementable now: `cypher_raise('TypeError', <detail>)` already exists
(`graph/temporal/src/lib.rs:1452`) and is already used by list indexing
(`graph/frontend/src/lowering.rs:2357`). The stale note in
`DESIGN_DECISIONS.md` — "SELECT cannot raise" — predates it.

### Negative offsets and lengths (8)

`substring(s, -1)`, `substring(s, 0, -1)`, `left(s, -1)`, `right(s, -1)` and the
`-2147483648` variants. The engine returns `""`. openCypher requires an error
for a negative length, so this matches AGE for the right reason.

### Null in `range()` (1)

`range(0, null, -3)`. A range needs a start and an end to walk between.

### Off by one in `substring()` (found while fixing the above)

Not one of the 104 — the rows that would have caught it were graded as errors.
`substring` lowered straight onto SQL's `substr()`, which counts from 1, while
Cypher counts from 0. Both authorities agree: the AGE donor has
`substring("0123456789", 1, 3)` → `"123"`, and TCK String1 [1] has
`substring('0123456789', 1)` → `'123456789'`.

### Non-boolean predicate (1)

`CASE WHEN 1 THEN 'fail' END`. openCypher predicates are boolean-typed;
`1` is not a truthy value in Cypher.

### Pattern-variable rules (3)

| query | engine today | rule |
|---|---|---|
| `CREATE p=() WITH p MATCH (p) RETURN p` | rows | a path variable cannot be re-bound as a node |
| `MERGE (x:r)-[y:E]->(x)-[z:E]->(y)` | rows | an edge variable cannot be used as a node |
| `MATCH ()-[p *]-()-[p *]-() RETURN p` | `[]` | a relationship variable cannot repeat in one pattern |

openCypher rejects all three. Returning empty for the third is the dangerous
one — it looks like "no match" rather than "your pattern is illegal".

## Reclassify as restrictions (47)

### AGE `::` typecast semantics (27)

`::` is Postgres cast syntax that AGE inherits; it has no openCypher
counterpart, and the strictness being tested is Postgres input-syntax
strictness, not Cypher semantics.

- Postgres literal parsing (13): `'0.0'::int`, `'1.5'::int`, `''::int`,
  `'false_'::int`, `1.23::bool`, `''::bool`, `'false_'::bool`,
  `'NaN'::float::int`, `'infinity'::float::int`, `'2:71'::float`, `'infi'::float`,
  `''::pg_float8`, `('2:71'::numeric)::numeric`
- agtype-internal constructors (14): `::vertex` ×4, `::edge` ×5, `::path` ×3,
  `::pg_float8` ×2 — these validate AGE's on-disk agtype shape (`id`,
  `label`, `start_id`, `end_id`), which is not our storage model.

### Outer-SQL column-definition artifacts (7)

`cannot cast agtype {boolean,integer,float,numeric} to json`,
`cannot cast agtype path to type int`, `cypher(...) in expressions is not
supported`. The Cypher runs fine; the error comes from the wrapping
`SELECT … AS (result json)`. `is_age_restriction_error` already lists the
siblings `cannot cast agtype object` and `cannot cast agtype array` — this is
a one-line widening of a rule we already made.

### Postgres INT32 range on string functions (8, revised to 4)

`substring/left/right` with `±2147483649` → "out of INT range". AGE's limit is
Postgres `int4`. Cypher integers are 64-bit, so the bound does not exist for us.
The four negative rows moved to the fix column: they overflow the int4 check on
the way past it, but a negative offset or length is an error either way.

### Null offsets and lengths (3)

`substring(s, null)`, `left(s, null)`, `right(s, null)`. We return `null`;
AGE raises. Null propagation is the openCypher-correct answer, so AGE is the
outlier here — unlike the negative-length rows above.

### AGE grammar restrictions (2)

- `ON MATCH SET specified more than once` — exact sibling of the already-listed
  `ON CREATE SET specified more than once`.
- `MATCH cannot follow OPTIONAL MATCH` — legal openCypher; AGE's parser
  forbids it.

## How the work went

The restriction rows are message-pattern additions to
`is_age_restriction_error`. They stop expecting an error and start expecting
rows, so they pass as soon as the query runs — no engine change.

The fixes turned out to be one theme with a long tail, and none of them needed
`cypher_raise` after all: every wrong-typed argument in the corpus has a type the
binder already knows, so the check lands at bind time. `builtin_argument_conflict`
in `graph/frontend/src/binder.rs` is the shared "argument must be a
list/string/number" helper, rather than 44 bespoke checks.
