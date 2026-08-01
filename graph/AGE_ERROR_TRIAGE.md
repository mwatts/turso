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

`range(0, null, -3)`. `range` takes integers; null is not one.

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

### Postgres INT32 range on string functions (8)

`substring/left/right` with `±2147483649` → "out of INT range". AGE's limit is
Postgres `int4`. Cypher integers are 64-bit, so the bound does not exist for us.

### Null offsets and lengths (3)

`substring(s, null)`, `left(s, null)`, `right(s, null)`. We return `null`;
AGE raises. Null propagation is the openCypher-correct answer, so AGE is the
outlier here — unlike the negative-length rows above.

### AGE grammar restrictions (2)

- `ON MATCH SET specified more than once` — exact sibling of the already-listed
  `ON CREATE SET specified more than once`.
- `MATCH cannot follow OPTIONAL MATCH` — legal openCypher; AGE's parser
  forbids it.

## Doing the work

The 47 restriction rows are message-pattern additions to
`is_age_restriction_error` and flip to `Unsupported` with no engine change.

The 57 fixes are one theme with a long tail: argument type-checking at lowering
time, raising through `cypher_raise`. Sensible order is (1) the 3 pattern-variable
rules (bind-time, no runtime plumbing), (2) negative/null argument-domain checks
on `substring`/`left`/`right`/`range`, (3) the 44 function type checks, which
want a shared "argument must be a list/string/number" helper rather than 44
bespoke checks.
