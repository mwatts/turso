# Graph type-system follow-ups (deferred, non-blocking)

Source: final whole-branch review of `feature/graph-type-system`
(`a31ce0ef7..f55c23199`, plus the subsequent fix-forward round at
`28d9300f8..61d691027`), triaging the Minor findings accumulated across all
13 tasks plus the final review's own new Minor findings. None of these
block merge — recorded here so they aren't lost once the branch lands.

## 1. Redundant schema lookup in `require_custom_types_enabled_for_source`

`graph/frontend/src/catalog.rs:594-620` (Task 4). The STRICT
custom-types-disabled gate calls `connection.current_schema()` and
`schema.get_table(table_name)` independently of the schema/table lookups
already performed by nearby registration code in the same call path
(`register_graph_in_transaction`, `require_columns`). Not a correctness
issue — just a second `Arc<RwLock<Schema>>` read where one might be
reusable. Low priority; only worth doing alongside other work that already
touches this function.

## 2. Duplicated array-branch logic between `Builtin|Domain` and `Custom` arms

`graph/frontend/src/schema_catalog.rs:193-212` (Task 6 out-of-band fix). The
`ColumnTypeKind::Builtin | ColumnTypeKind::Domain` arm and the
`ColumnTypeKind::Custom` arm both compute the same
`if is_array { primitive_value_type(...) } else { sqlite_type_value_type(column.affinity().to_type()) }`
scalar resolution; `Custom` then wraps the result in `ir::ValueType::Custom`.
Could be factored into a shared helper returning the scalar, with `Custom`
wrapping it — deferred at review time to avoid touching working, tested
code without a concrete reason.

## 3. No duplicate-field-name rejection in `bind_map_property`

`graph/frontend/src/binder.rs:1201` (Task 8). A map literal like
`{name: 'a', name: 'b'}` bound against a STRUCT/UNION target silently keeps
only the last entry per name rather than raising a bind error. All map
literal bind errors already share `BindError::Unsupported`'s static-string
convention, so this would follow that pattern
(`"duplicate map literal field name"` or similar) if implemented.

## 4. `PropertyId::new(43)` magic number in a binder test

`graph/frontend/src/binder.rs:1743,1775` (Task 10). The nested-property-read
regression test hardcodes `PropertyId::new(43)` rather than deriving it from
the same catalog construction the test itself performs. Purely a test
readability nit — the value is correct, just unexplained.

## 5. No test coverage for nested composite (STRUCT-in-STRUCT/UNION) field types

`graph/frontend/src/schema_catalog.rs` (Task 9 out-of-band fix, the
`resolve_named_type` bare-primitive fallback). The fix threads core's
per-field/variant `Affinity` through as a registry-miss fallback for
top-level STRUCT/UNION fields, but there's no regression test for a field
that is itself a STRUCT or UNION (two levels of composite nesting). Related
to, but distinct from, Task 10/11's separate nested-*property-read* depth
cap (which *is* tested) — this gap is specifically about a composite type's
*field* being declared as another composite type.

## Also flagged, not itemized above

The final review noted CREATE cannot bind a Cypher map literal whose own
field is itself a nested map literal at depth ≥2 (`bind_map_property`
doesn't recurse into itself for nested Struct/Union targets) — this is a
real, user-visible limitation (not just an internal code-quality nit) and
should be called out in the PR description for discoverability, separately
from this follow-up list.
