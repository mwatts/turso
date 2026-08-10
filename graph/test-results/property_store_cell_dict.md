# Cell store: integer prop_id dictionary + typed values

## Why not `prop_key TEXT` on every cell?

With open property sets and high cardinality entities, repeating UTF-8 property
names on every cell row:

- inflates storage and index leaf size
- slows `(prop_key, value)` seeks (longer keys)
- duplicates strings the catalog already knows

**Dictionary:** map each property **name** once to a stable **integer id**.

```sql
CREATE TABLE graph_prop_dict(
  prop_id INTEGER PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE,
  value_type TEXT NOT NULL   -- 'text' | 'integer' | 'float' | 'boolean' | 'any'
);

CREATE TABLE graph_node_props(
  node_id INTEGER NOT NULL,
  prop_id INTEGER NOT NULL,  -- FK-ish to graph_prop_dict.prop_id
  value,                     -- SQLite dynamic type; type *meaning* from dict
  PRIMARY KEY(node_id, prop_id)
);
CREATE INDEX graph_node_props_by_kv ON graph_node_props(prop_id, value);
```

In this codebase the integer often is (or aligns with) IR `PropertyId` once the
binder has resolved `n.name` → id.

## What is the type of `value`?

**Storage:** one SQL column `value` with SQLite dynamic affinity (INTEGER / REAL
/ TEXT / BLOB / NULL per *row*). That keeps one table for all properties.

**Meaning:** per-`prop_id` type comes from **`graph_prop_dict.value_type`**, not
from guessing at runtime on every row. Because filters always include
`prop_id = ?`, each index seek is type-homogeneous for that property.

| Cypher / ops | Dict type | SQL sketch |
|---|---|---|
| `=`, `<>`, `<`, `>`, `<=`, `>=` | integer / float / text | `value` compared with normal affinity |
| `IS NULL` / `IS NOT NULL` | any | `value IS NULL` (missing cell = null property) |
| `CONTAINS` / `STARTS WITH` / `ENDS WITH` | **text only** | e.g. `instr(value, ?)` / `LIKE`; **reject** if dict type is integer |
| `=~` regex (if supported) | text only | same gate |
| list/map property | later | separate encoding or typed columns |

**Do you need the dictionary for lowering?**  
**Yes** for open Cell stores:

1. Name → `prop_id` at bind (`n.foo` → integer for SQL).  
2. Type → legal operators (`CONTAINS` only if text).  
3. Optional casts (`CAST(value AS TEXT)` only when declared text).  
4. Avoid mixed-type chaos under one `(prop_id, value)` seek.

Spike API: `PropertyPhysical::Cell { prop_id, value_type, … }` and
`supports_text_predicate()`. Binder wiring to *enforce* CONTAINS rejection is
still a follow-up; the physical layer now carries the type.

## Exact implementation in the spike

```text
n.age  (PropertyId = 2, value_type = Integer)
  → PropertyPhysical::Cell {
       props_table: "node_props",
       prop_id: 2,
       prop_id_column: "prop_id",
       value_column: "value",
       value_type: Integer,
       …
     }

Read:  (SELECT value FROM node_props WHERE node_id = n.id AND prop_id = 2)
Filter open:  WHERE prop_id = 2 AND value = 36   -- uses INDEX(prop_id, value)
SET:   DELETE … prop_id = 2; INSERT … (node_id, 2, 36)
```

No property-name string on the hot path after bind.

## Optional dual-write bag

For fast `properties(n)` only:

```sql
ALTER TABLE graph_nodes ADD COLUMN props JSONB;  -- dual-write on SET/CREATE
```

Filters still go through **cells + dict**, not bag extract, under open query.

## In-memory API

`PropertyDictionary` (`graph/frontend/src/property_physical.rs`):

- `register(name, value_type)` → allocates next `prop_id`
- `register_with_id` → load durable rows
- case-insensitive name match; type conflicts rejected
- `cell_physical(...)` → `PropertyPhysical::Cell { prop_id, value_type, … }`

Binder already rejects `CONTAINS` / `STARTS WITH` / `ENDS WITH` when
`ResolvedProperty.value_type` is non-text (`text_compatible`). Dictionary types
must be fed into `catalog.property()` so that gate fires (CellDict tests cover
`n.age CONTAINS` fail and `n.name CONTAINS` pass).

## Tests

`property_physical_modes`:

- `cell_dict_*` — durable-shaped tables, integer prop_id, index, CREATE/MERGE  
- `binder_rejects_contains_on_integer_cell_property` / `binder_allows_contains_on_text_*`  
- `open_multi_property_and_filter_via_cell_dict` — AND of two open props  
- `property_dictionary_register_*` — in-memory dictionary
