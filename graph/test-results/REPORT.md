# Graph test history

Generated from `graph/test-results/history.jsonl`. Results are grouped by stable test identity; performance comparisons are meaningful only for matching environment and workload dimensions.

## Latest complete corpus run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Records: 10392
- Passed: 7462
- Failed: 2930

### Failure-reason histogram

| Failure family | Count |
|---|---:|
| `execution`: other | 764 |
| `parser`: other grammar | 550 |
| `execution`: mutation projection unsupported | 473 |
| `execution`: runtime scalar function missing | 279 |
| `parser`: expression/operator continuation grammar | 270 |
| `execution`: mutation operation unsupported | 149 |
| `parser`: graph-pattern grammar | 110 |
| `parser`: unsupported starting clause | 85 |
| `parser`: projection/expression item grammar | 53 |
| `parser`: map-literal grammar | 41 |
| `execution`: expected-error mismatch | 38 |
| `setup-execution`: other | 34 |
| `parameter-binding`: other | 20 |
| `side-effect-comparison`: other | 20 |
| `execution`: parameter binding/declaration | 19 |
| `fixture-execution`: other | 19 |
| `dataset-execution`: other | 4 |
| `setup-execution`: runtime scalar function missing | 2 |

## Latest `age-deep` run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Commit: `aeb0c662831c736bacd67f988d1a6a878f60a196` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 3677
- Passed: 2204
- Unsupported: 0
- Failed or changed: 1473

### Outcome changes from `20260719T210524.711499Z-51b9a1bd28bf-corpus-deep`

- `age.cypher.match.query-7`: Passed
- `age.cypher.match.query-35`: Passed
- `age.cypher.match.query-41`: Passed
- `age.cypher.match.query-124`: Passed
- `age.expr.query-327`: Passed

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| age_global_graph | `failed` | 30 |
| age_global_graph | `passed` | 27 |
| age_load | `failed` | 6 |
| age_load | `passed` | 7 |
| age_reduce | `failed` | 72 |
| age_reduce | `passed` | 3 |
| age_shortest_path | `failed` | 17 |
| age_shortest_path | `passed` | 177 |
| agtype | `failed` | 3 |
| agtype | `passed` | 15 |
| agtype_jsonb_cast | `passed` | 3 |
| analyze | `failed` | 1 |
| analyze | `passed` | 1 |
| catalog | `passed` | 7 |
| cypher | `passed` | 20 |
| cypher_call | `failed` | 40 |
| cypher_call | `passed` | 2 |
| cypher_create | `failed` | 36 |
| cypher_create | `passed` | 57 |
| cypher_delete | `failed` | 4 |
| cypher_delete | `passed` | 111 |
| cypher_match | `failed` | 175 |
| cypher_match | `passed` | 238 |
| cypher_merge | `failed` | 99 |
| cypher_merge | `passed` | 174 |
| cypher_remove | `failed` | 4 |
| cypher_remove | `passed` | 38 |
| cypher_set | `failed` | 35 |
| cypher_set | `passed` | 82 |
| cypher_subquery | `failed` | 15 |
| cypher_subquery | `passed` | 38 |
| cypher_union | `failed` | 14 |
| cypher_union | `passed` | 5 |
| cypher_unwind | `failed` | 3 |
| cypher_unwind | `passed` | 14 |
| cypher_vle | `failed` | 60 |
| cypher_vle | `passed` | 52 |
| cypher_with | `failed` | 24 |
| cypher_with | `passed` | 17 |
| direct_field_access | `failed` | 6 |
| direct_field_access | `passed` | 35 |
| expr | `failed` | 443 |
| expr | `passed` | 646 |
| fuzzystrmatch | `failed` | 10 |
| fuzzystrmatch | `passed` | 1 |
| generated_columns | `passed` | 10 |
| graph_generation | `passed` | 2 |
| index | `failed` | 13 |
| index | `passed` | 52 |
| issue_369 | `passed` | 4 |
| jsonb_operators | `failed` | 159 |
| list_comprehension | `failed` | 26 |
| list_comprehension | `passed` | 100 |
| map_projection | `failed` | 17 |
| map_projection | `passed` | 1 |
| name_validation | `failed` | 6 |
| name_validation | `passed` | 4 |
| pattern_expression | `failed` | 4 |
| pattern_expression | `passed` | 28 |
| pg_trgm | `failed` | 6 |
| pgvector | `failed` | 71 |
| predicate_functions | `failed` | 17 |
| predicate_functions | `passed` | 45 |
| reserved_keyword_alias | `failed` | 12 |
| reserved_keyword_alias | `passed` | 19 |
| scan | `failed` | 38 |
| scan | `passed` | 19 |
| security | `failed` | 7 |
| security | `passed` | 126 |
| subgraph | `passed` | 24 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 658 |
| `execution` | `passed` | 2204 |
| `parser` | `failed` | 815 |

### Failures (1473)

- `age.age.global.graph.query-4`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-5`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-6`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-7`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-8`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.age.global.graph.query-9`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-10`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-11`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-12`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-13`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-14`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-15`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-16`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-17`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-18`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-19`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..33
- `age.age.global.graph.query-20`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-21`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-22`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..41
- `age.age.global.graph.query-25`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 65..93; mutation execution failed: Cypher mutation binding failed: modifiers on a mutation RETURN clause is not supported in the initial graph slice at byte 93..116
- `age.age.global.graph.query-26`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-27`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.global.graph.query-28`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-29`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-30`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-35`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-36`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 18..91; mutation execution failed: Cypher mutation binding failed: modifiers on a mutation RETURN clause is not supported in the initial graph slice at byte 292..324
- `age.age.global.graph.query-39`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-40`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.age.global.graph.query-51`: query execution failed: Parse error: property access requires a node or relationship at byte 132..133; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 89..140
- `age.age.load.query-8`: expected not_expression at byte 27..27
- `age.age.load.query-9`: query execution failed: Parse error: no such function: graph_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..39
- `age.age.load.query-10`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..38
- `age.age.load.query-11`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..38
- `age.age.load.query-12`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 24..44
- `age.age.load.query-13`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 24..44
- `age.age.reduce.query-1`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-2`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 58..58
- `age.age.reduce.query-3`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 42..42
- `age.age.reduce.query-4`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 38..38
- `age.age.reduce.query-5`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 31..31
- `age.age.reduce.query-6`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-7`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 43..43
- `age.age.reduce.query-8`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 43..43
- `age.age.reduce.query-9`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 29..29
- `age.age.reduce.query-10`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 31..31
- `age.age.reduce.query-11`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 31..31
- `age.age.reduce.query-12`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 32..32
- `age.age.reduce.query-13`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-14`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-15`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-16`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-17`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 33..33
- `age.age.reduce.query-18`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-19`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-20`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.age.reduce.query-21`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-22`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 41..41
- `age.age.reduce.query-23`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 51..51
- `age.age.reduce.query-24`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 45..45
- `age.age.reduce.query-25`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 38..38
- `age.age.reduce.query-26`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 45..45
- `age.age.reduce.query-27`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-28`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 45..45
- `age.age.reduce.query-29`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 49..49
- `age.age.reduce.query-30`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 51..51
- `age.age.reduce.query-31`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 40..40
- `age.age.reduce.query-32`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.age.reduce.query-33`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.age.reduce.query-34`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 40..40
- `age.age.reduce.query-35`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 51..51
- `age.age.reduce.query-36`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 58..58
- `age.age.reduce.query-37`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-38`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.reduce.query-39`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 56..56
- `age.age.reduce.query-40`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 47..47
- `age.age.reduce.query-41`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 72..72
- `age.age.reduce.query-42`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 57..57
- `age.age.reduce.query-43`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 58..58
- `age.age.reduce.query-44`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 54..54
- `age.age.reduce.query-45`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 76..76
- `age.age.reduce.query-47`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 88..88
- `age.age.reduce.query-49`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 46..46
- `age.age.reduce.query-50`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 67..67
- `age.age.reduce.query-51`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-52`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 57..57
- `age.age.reduce.query-53`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 62..62
- `age.age.reduce.query-54`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 75..75
- `age.age.reduce.query-55`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-56`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 63..63
- `age.age.reduce.query-57`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 68..68
- `age.age.reduce.query-58`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 53..53
- `age.age.reduce.query-59`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 55..55
- `age.age.reduce.query-60`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 63..63
- `age.age.reduce.query-61`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 65..65
- `age.age.reduce.query-62`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-63`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-64`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 55..55
- `age.age.reduce.query-65`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 55..55
- `age.age.reduce.query-66`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-67`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 52..52
- `age.age.reduce.query-68`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 33..33
- `age.age.reduce.query-69`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 33..33
- `age.age.reduce.query-70`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 29..29
- `age.age.reduce.query-71`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 20..20
- `age.age.reduce.query-72`: query execution failed: Parse error: unknown variable `s` at byte 14..16; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..33
- `age.age.reduce.query-73`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 35..35
- `age.age.reduce.query-74`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.age.shortest.path.query-2`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-66`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-67`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..71
- `age.age.shortest.path.query-68`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..76
- `age.age.shortest.path.query-69`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..80
- `age.age.shortest.path.query-73`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-134`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-135`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..80
- `age.age.shortest.path.query-136`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..85
- `age.age.shortest.path.query-137`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..91
- `age.age.shortest.path.query-138`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..96
- `age.age.shortest.path.query-139`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..96
- `age.age.shortest.path.query-140`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..87
- `age.age.shortest.path.query-142`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-168`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-180`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.age.shortest.path.query-190`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..47
- `age.agtype.query-1`: expected not_expression at byte 114..114
- `age.agtype.query-9`: query execution failed: Parse error: invalid resolved function or parameter name: ag_catalog.agtype_build_map; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..73
- `age.agtype.query-11`: expected primary_expression at byte 38..38
- `age.analyze.query-2`: expected clause at byte 0..0
- `age.cypher.call.query-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.cypher.call.query-4`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.cypher.call.query-5`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.cypher.call.query-6`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..26
- `age.cypher.call.query-7`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..35
- `age.cypher.call.query-8`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..26; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.cypher.call.query-9`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-10`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-11`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.cypher.call.query-12`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-13`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-14`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-15`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-16`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 15..19; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 10..24
- `age.cypher.call.query-17`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 15..19; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 10..35
- `age.cypher.call.query-18`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-19`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-20`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-21`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-22`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-23`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-24`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-25`: expected identifier at byte 58..58
- `age.cypher.call.query-26`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-27`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-28`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-29`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-30`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-31`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-32`: expected EOI, UNION, or clause at byte 50..50
- `age.cypher.call.query-33`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-34`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-35`: expected EOI, UNION, or clause at byte 63..63
- `age.cypher.call.query-36`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..24; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..44
- `age.cypher.call.query-37`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..11; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.cypher.call.query-38`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.cypher.call.query-39`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..11; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.cypher.call.query-40`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..11; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.cypher.call.query-41`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..38
- `age.cypher.call.query-42`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.cypher.create.query-6`: expected not_expression at byte 15..15
- `age.cypher.create.query-7`: expected not_expression at byte 15..15
- `age.cypher.create.query-8`: expected not_expression at byte 16..16
- `age.cypher.create.query-9`: expected not_expression at byte 16..16
- `age.cypher.create.query-10`: expected not_expression at byte 16..16
- `age.cypher.create.query-11`: expected not_expression at byte 16..16
- `age.cypher.create.query-12`: expected node_pattern at byte 18..18
- `age.cypher.create.query-13`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..19; mutation execution failed: Cypher mutation binding failed: undirected relationship creation is not supported in the initial graph slice at byte 11..17
- `age.cypher.create.query-14`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..20; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 11..16
- `age.cypher.create.query-29`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..26; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.cypher.create.query-30`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..35; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.cypher.create.query-32`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..45; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.cypher.create.query-36`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..39; mutation execution failed: Cypher mutation binding failed: unknown parameter `$var_name` at byte 27..36
- `age.cypher.create.query-38`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 42..74; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 49..61
- `age.cypher.create.query-39`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 42..75; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 49..62
- `age.cypher.create.query-40`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 49..73; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 61..62
- `age.cypher.create.query-41`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..14; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.cypher.create.query-62`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 20..33
- `age.cypher.create.query-63`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..29; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 23..29
- `age.cypher.create.query-64`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..12; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.create.query-66`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 14..15
- `age.cypher.create.query-67`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..24; mutation execution failed: mutation references binding 1 before it has a value
- `age.cypher.create.query-69`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..22; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.create.query-70`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..31; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 22..23
- `age.cypher.create.query-71`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..32; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 23..29
- `age.cypher.create.query-72`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..25; mutation execution failed: Cypher mutation binding failed: duplicate variable `e` at byte 37..38
- `age.cypher.create.query-73`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 24..47; mutation execution failed: Cypher mutation binding failed: duplicate variable `e` at byte 36..37
- `age.cypher.create.query-74`: expected identifier at byte 10..10
- `age.cypher.create.query-75`: expected identifier at byte 10..10
- `age.cypher.create.query-76`: expected identifier at byte 10..10
- `age.cypher.create.query-79`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..21; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.create.query-84`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..25; mutation execution failed: Cypher mutation binding failed: duplicate variable `e` at byte 37..38
- `age.cypher.create.query-85`: expected not_expression at byte 34..34
- `age.cypher.create.query-86`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..39; mutation execution failed: Cypher mutation binding failed: duplicate variable `e` at byte 26..27
- `age.cypher.create.query-87`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..55; mutation execution failed: Cypher mutation binding failed: duplicate variable `e` at byte 29..30
- `age.cypher.create.query-89`: expected not_expression at byte 35..35
- `age.cypher.delete.query-72`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..19; mutation execution failed: Cypher mutation binding failed: unknown variable `m` at byte 17..18
- `age.cypher.delete.query-74`: expected identifier at byte 17..17
- `age.cypher.delete.query-77`: expected identifier at byte 17..17
- `age.cypher.delete.query-105`: expected identifier at byte 27..27
- `age.cypher.match.query-57`: expected not_expression at byte 21..21
- `age.cypher.match.query-58`: expected not_expression at byte 30..30
- `age.cypher.match.query-60`: expected not_expression at byte 23..23
- `age.cypher.match.query-61`: expected not_expression at byte 22..22
- `age.cypher.match.query-62`: expected not_expression at byte 22..22
- `age.cypher.match.query-63`: expected not_expression at byte 31..31
- `age.cypher.match.query-64`: expected not_expression at byte 31..31
- `age.cypher.match.query-65`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-74`: query execution failed: Parse error: duplicate variable `a` at byte 20..21; mutation execution failed: Cypher mutation binding failed: duplicate variable `a` at byte 20..21
- `age.cypher.match.query-75`: query execution failed: Parse error: duplicate variable `r0` at byte 28..30; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 28..30
- `age.cypher.match.query-76`: query execution failed: Parse error: duplicate variable `r0` at byte 31..33; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 31..33
- `age.cypher.match.query-77`: query execution failed: Parse error: duplicate variable `r0` at byte 31..33; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 31..33
- `age.cypher.match.query-80`: query execution failed: Parse error: duplicate variable `r0` at byte 40..42; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 40..42
- `age.cypher.match.query-89`: query execution failed: Parse error: duplicate variable `r0` at byte 40..42; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 40..42
- `age.cypher.match.query-90`: query execution failed: Parse error: duplicate variable `r0` at byte 31..33; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 31..33
- `age.cypher.match.query-92`: query execution failed: Parse error: duplicate variable `r1` at byte 47..49; mutation execution failed: Cypher mutation binding failed: duplicate variable `r1` at byte 47..49
- `age.cypher.match.query-97`: query execution failed: Parse error: duplicate variable `r0` at byte 12..14; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 12..14
- `age.cypher.match.query-98`: query execution failed: Parse error: duplicate variable `r0` at byte 28..30; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 28..30
- `age.cypher.match.query-99`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 31..35; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 31..35
- `age.cypher.match.query-100`: query execution failed: Parse error: duplicate variable `r0` at byte 31..33; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 31..33
- `age.cypher.match.query-101`: query execution failed: Parse error: duplicate variable `r0` at byte 28..30; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 28..30
- `age.cypher.match.query-102`: query execution failed: Parse error: duplicate variable `r0` at byte 31..33; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 31..33
- `age.cypher.match.query-103`: query execution failed: Parse error: duplicate variable `r0` at byte 22..24; mutation execution failed: Cypher mutation binding failed: duplicate variable `r0` at byte 22..24
- `age.cypher.match.query-104`: query execution failed: Parse error: duplicate variable `r1` at byte 40..42; mutation execution failed: Cypher mutation binding failed: duplicate variable `r1` at byte 40..42
- `age.cypher.match.query-105`: query execution failed: Parse error: duplicate variable `r1` at byte 38..40; mutation execution failed: Cypher mutation binding failed: duplicate variable `r1` at byte 38..40
- `age.cypher.match.query-106`: query execution failed: Parse error: duplicate variable `r1` at byte 30..32; mutation execution failed: Cypher mutation binding failed: duplicate variable `r1` at byte 30..32
- `age.cypher.match.query-107`: query execution failed: Parse error: duplicate variable `r1` at byte 38..40; mutation execution failed: Cypher mutation binding failed: duplicate variable `r1` at byte 38..40
- `age.cypher.match.query-120`: expected not_expression at byte 37..37
- `age.cypher.match.query-121`: expected not_expression at byte 37..37
- `age.cypher.match.query-131`: query execution failed: Parse error: duplicate variable `e` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..55
- `age.cypher.match.query-132`: query execution failed: Parse error: generated relational SQL did not parse: near "json_array": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 69..83
- `age.cypher.match.query-134`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..39
- `age.cypher.match.query-136`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 29..37
- `age.cypher.match.query-137`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..41
- `age.cypher.match.query-138`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..57
- `age.cypher.match.query-139`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..61
- `age.cypher.match.query-140`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..44
- `age.cypher.match.query-141`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..46
- `age.cypher.match.query-142`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 26..34
- `age.cypher.match.query-143`: expected not_expression at byte 32..32
- `age.cypher.match.query-144`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..61
- `age.cypher.match.query-145`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 40..60
- `age.cypher.match.query-146`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 43..63
- `age.cypher.match.query-147`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..65
- `age.cypher.match.query-148`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..64
- `age.cypher.match.query-149`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 47..67
- `age.cypher.match.query-150`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 48..68
- `age.cypher.match.query-151`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 52..72
- `age.cypher.match.query-152`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..61
- `age.cypher.match.query-156`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..56
- `age.cypher.match.query-157`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..56
- `age.cypher.match.query-158`: query execution failed: Parse error: no such function: isEmpty; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 40..60
- `age.cypher.match.query-159`: query execution failed: Parse error: no such function: isEmpty; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..55
- `age.cypher.match.query-170`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 57..118
- `age.cypher.match.query-171`: query execution failed: Parse error: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..215; mutation execution failed: Cypher mutation binding failed: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..95
- `age.cypher.match.query-172`: query execution failed: Parse error: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..234; mutation execution failed: Cypher mutation binding failed: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 68..114
- `age.cypher.match.query-176`: query execution failed: Parse error: no such column: b4; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 93..104
- `age.cypher.match.query-177`: query execution failed: Parse error: no such column: b4; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 94..105
- `age.cypher.match.query-178`: query execution failed: Parse error: duplicate variable `r` at byte 59..60; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 59..60
- `age.cypher.match.query-179`: query execution failed: Parse error: duplicate variable `r` at byte 59..60; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 59..60
- `age.cypher.match.query-180`: query execution failed: Parse error: duplicate variable `r` at byte 59..60; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 59..60
- `age.cypher.match.query-181`: query execution failed: Parse error: duplicate variable `r` at byte 59..60; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 59..60
- `age.cypher.match.query-182`: query execution failed: Parse error: duplicate variable `r` at byte 59..60; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 59..60
- `age.cypher.match.query-183`: expected identifier at byte 24..24
- `age.cypher.match.query-184`: expected not_expression at byte 17..17
- `age.cypher.match.query-194`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..39
- `age.cypher.match.query-195`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 31..45; mutation execution failed: graph mutation database operation failed: near "(": syntax error
- `age.cypher.match.query-196`: expected primary_expression at byte 43..43
- `age.cypher.match.query-197`: expected primary_expression at byte 43..43
- `age.cypher.match.query-198`: expected primary_expression at byte 43..43
- `age.cypher.match.query-202`: expected not_expression at byte 23..23
- `age.cypher.match.query-203`: expected not_expression at byte 44..44
- `age.cypher.match.query-204`: expected not_expression at byte 34..34
- `age.cypher.match.query-205`: expected not_expression at byte 44..44
- `age.cypher.match.query-206`: expected not_expression at byte 44..44
- `age.cypher.match.query-207`: expected not_expression at byte 16..16
- `age.cypher.match.query-208`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..57
- `age.cypher.match.query-209`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..61
- `age.cypher.match.query-211`: expected not_expression at byte 43..43
- `age.cypher.match.query-212`: expected not_expression at byte 36..36
- `age.cypher.match.query-213`: expected not_expression at byte 53..53
- `age.cypher.match.query-214`: expected not_expression at byte 53..53
- `age.cypher.match.query-215`: expected not_expression at byte 26..26
- `age.cypher.match.query-216`: expected not_expression at byte 26..26
- `age.cypher.match.query-217`: expected not_expression at byte 17..17
- `age.cypher.match.query-220`: expected not_expression at byte 33..33
- `age.cypher.match.query-221`: expected not_expression at byte 33..33
- `age.cypher.match.query-222`: expected not_expression at byte 18..18
- `age.cypher.match.query-223`: expected not_expression at byte 18..18
- `age.cypher.match.query-224`: expected not_expression at byte 18..18
- `age.cypher.match.query-225`: expected not_expression at byte 18..18
- `age.cypher.match.query-226`: query execution failed: Parse error: duplicate variable `b` at byte 21..22; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 21..22
- `age.cypher.match.query-227`: query execution failed: Parse error: duplicate variable `b` at byte 21..22; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 21..22
- `age.cypher.match.query-228`: query execution failed: Parse error: duplicate variable `b` at byte 27..28; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 27..28
- `age.cypher.match.query-229`: query execution failed: Parse error: duplicate variable `b` at byte 27..28; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 27..28
- `age.cypher.match.query-230`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10
- `age.cypher.match.query-231`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10
- `age.cypher.match.query-232`: query execution failed: Parse error: duplicate variable `p` at byte 12..13; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 12..13
- `age.cypher.match.query-233`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19
- `age.cypher.match.query-234`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19
- `age.cypher.match.query-235`: query execution failed: Parse error: duplicate variable `p` at byte 21..22; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 21..22
- `age.cypher.match.query-236`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.match.query-237`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.match.query-238`: query execution failed: Parse error: duplicate variable `p` at byte 19..20; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 19..20
- `age.cypher.match.query-239`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..11; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 18..29
- `age.cypher.match.query-240`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..12; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 19..29
- `age.cypher.match.query-241`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..24; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 31..42
- `age.cypher.match.query-242`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..12; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 19..36
- `age.cypher.match.query-244`: expected not_expression at byte 24..24
- `age.cypher.match.query-245`: expected not_expression at byte 24..24
- `age.cypher.match.query-246`: expected not_expression at byte 29..29
- `age.cypher.match.query-247`: expected not_expression at byte 38..38
- `age.cypher.match.query-251`: expected not_expression at byte 14..14
- `age.cypher.match.query-259`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 17..36
- `age.cypher.match.query-262`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..378; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 58..200
- `age.cypher.match.query-272`: expected clause at byte 0..0
- `age.cypher.match.query-282`: expected clause at byte 0..0
- `age.cypher.match.query-335`: query execution failed: Parse error: unknown variable `e` at byte 80..81; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 67..81
- `age.cypher.match.query-346`: query execution failed: Parse error: duplicate variable `e1` at byte 77..79; mutation execution failed: Cypher mutation binding failed: duplicate variable `e1` at byte 77..79
- `age.cypher.match.query-347`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 92..96; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 92..96
- `age.cypher.match.query-349`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 89..98
- `age.cypher.match.query-350`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 62..70
- `age.cypher.match.query-353`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..99; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 52..83
- `age.cypher.match.query-354`: expected map_literal at byte 18..18
- `age.cypher.match.query-355`: expected map_literal at byte 18..18
- `age.cypher.match.query-356`: expected map_literal at byte 18..18
- `age.cypher.match.query-357`: expected map_literal at byte 18..18
- `age.cypher.match.query-358`: expected map_literal at byte 18..18
- `age.cypher.match.query-359`: expected map_literal at byte 18..18
- `age.cypher.match.query-360`: expected map_literal at byte 18..18
- `age.cypher.match.query-361`: expected range_literal or map_literal at byte 19..19
- `age.cypher.match.query-362`: expected map_literal at byte 18..18
- `age.cypher.match.query-363`: expected map_literal at byte 18..18
- `age.cypher.match.query-364`: expected map_literal at byte 18..18
- `age.cypher.match.query-365`: expected map_literal at byte 18..18
- `age.cypher.match.query-366`: expected range_literal or map_literal at byte 30..30
- `age.cypher.match.query-367`: expected range_literal or map_literal at byte 30..30
- `age.cypher.match.query-369`: expected clause at byte 0..0
- `age.cypher.match.query-370`: expected clause at byte 0..0
- `age.cypher.match.query-371`: expected map_literal at byte 18..18
- `age.cypher.match.query-372`: expected map_literal at byte 18..18
- `age.cypher.match.query-373`: expected map_literal at byte 18..18
- `age.cypher.match.query-374`: expected map_literal at byte 18..18
- `age.cypher.match.query-375`: expected map_literal at byte 18..18
- `age.cypher.match.query-376`: expected map_literal at byte 18..18
- `age.cypher.match.query-377`: expected map_literal at byte 18..18
- `age.cypher.match.query-378`: expected range_literal or map_literal at byte 19..19
- `age.cypher.match.query-379`: expected map_literal at byte 18..18
- `age.cypher.match.query-380`: expected map_literal at byte 18..18
- `age.cypher.match.query-381`: expected map_literal at byte 18..18
- `age.cypher.match.query-382`: expected map_literal at byte 18..18
- `age.cypher.match.query-383`: expected range_literal or map_literal at byte 30..30
- `age.cypher.match.query-384`: expected range_literal or map_literal at byte 30..30
- `age.cypher.match.query-386`: expected clause at byte 0..0
- `age.cypher.match.query-387`: expected clause at byte 0..0
- `age.cypher.match.query-388`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..44; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 61..88
- `age.cypher.match.query-389`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 50..73
- `age.cypher.match.query-390`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 50..73
- `age.cypher.match.query-391`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 44..58
- `age.cypher.match.query-393`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 32..51; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 68..91
- `age.cypher.match.query-396`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-397`: expected relationship_types, range_literal, or map_literal at byte 12..12
- `age.cypher.match.query-398`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-399`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-400`: expected relationship_types, range_literal, or map_literal at byte 12..12
- `age.cypher.match.query-401`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-402`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..116; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 130..151
- `age.cypher.match.query-403`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..37; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 48..67
- `age.cypher.match.query-404`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..36; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 47..66
- `age.cypher.match.query-406`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..38; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 49..80
- `age.cypher.match.query-409`: query execution failed: Parse error: unknown variable `name` at byte 178..182; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 120..182
- `age.cypher.match.query-410`: query execution failed: Parse error: unknown variable `name` at byte 173..177; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 115..185
- `age.cypher.match.query-411`: query execution failed: Parse error: unknown variable `name` at byte 175..179; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 117..187
- `age.cypher.match.query-412`: query execution failed: Parse error: unknown variable `name` at byte 158..162; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 100..162
- `age.cypher.match.query-413`: query execution failed: Parse error: unknown variable `name` at byte 144..148; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 86..148
- `age.cypher.merge.query-1`: expected not_expression at byte 13..13
- `age.cypher.merge.query-4`: expected not_expression at byte 12..12
- `age.cypher.merge.query-5`: expected not_expression at byte 11..11
- `age.cypher.merge.query-12`: expected not_expression at byte 12..12
- `age.cypher.merge.query-16`: expected not_expression at byte 12..12
- `age.cypher.merge.query-36`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..20; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-48`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..26; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-58`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..24; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-61`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-64`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..11; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-67`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..11; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-84`: expected not_expression at byte 25..25
- `age.cypher.merge.query-85`: expected not_expression at byte 25..25
- `age.cypher.merge.query-86`: expected not_expression at byte 25..25
- `age.cypher.merge.query-90`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..16; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-93`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-96`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..18; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-99`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..18; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-102`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..21; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.cypher.merge.query-103`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..18; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-105`: expected EOI, UNION, clause, or relationship_pattern at byte 9..9
- `age.cypher.merge.query-106`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 16..32; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 24..30
- `age.cypher.merge.query-108`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 39..48; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.merge.query-118`: expected not_expression at byte 57..57
- `age.cypher.merge.query-122`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..45; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 27..32
- `age.cypher.merge.query-123`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..45; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 27..32
- `age.cypher.merge.query-124`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..47; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-125`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..47; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-126`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..47; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-129`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-130`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..21; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-133`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..21; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-134`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-137`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-138`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-140`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..21; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-141`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..22; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-143`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-144`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..25; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 20..25
- `age.cypher.merge.query-145`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..21; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 16..21
- `age.cypher.merge.query-146`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-147`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..31; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 22..28
- `age.cypher.merge.query-148`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..31; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 22..28
- `age.cypher.merge.query-149`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-150`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 24..30
- `age.cypher.merge.query-151`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..35; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 26..27
- `age.cypher.merge.query-152`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 24..30
- `age.cypher.merge.query-160`: expected not_expression at byte 23..23
- `age.cypher.merge.query-161`: expected not_expression at byte 21..21
- `age.cypher.merge.query-164`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-165`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..20; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.merge.query-167`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 22..32; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.merge.query-168`: expected not_expression at byte 34..34
- `age.cypher.merge.query-169`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..38; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-170`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-171`: expected not_expression at byte 35..35
- `age.cypher.merge.query-172`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 22..32; mutation execution failed: Cypher mutation binding failed: mutation query contains no mutation clauses
- `age.cypher.merge.query-173`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..24; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-174`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-175`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..16; mutation execution failed: MERGE requires at least one property to identify the entity
- `age.cypher.merge.query-182`: expected identifier at byte 16..16
- `age.cypher.merge.query-183`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 66..145; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 91..95
- `age.cypher.merge.query-184`: expected identifier at byte 16..16
- `age.cypher.merge.query-185`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 156..235; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 181..185
- `age.cypher.merge.query-186`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 156..235; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 181..185
- `age.cypher.merge.query-187`: expected identifier at byte 16..16
- `age.cypher.merge.query-188`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 156..235; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 181..185
- `age.cypher.merge.query-190`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-191`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-195`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-197`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-199`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-202`: expected identifier or not_expression at byte 8..8
- `age.cypher.merge.query-206`: expected property_target at byte 192..192
- `age.cypher.merge.query-209`: expected property_target at byte 304..304
- `age.cypher.merge.query-212`: expected property_target at byte 232..232
- `age.cypher.merge.query-216`: expected property_target at byte 232..232
- `age.cypher.merge.query-223`: expected property_target at byte 116..116
- `age.cypher.merge.query-230`: query execution failed: Parse error: unknown variable `label` at byte 70..75; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 14..75
- `age.cypher.merge.query-235`: query execution failed: Parse error: unknown variable `label` at byte 70..75; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 14..75
- `age.cypher.merge.query-242`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 28..75; mutation execution failed: Cypher mutation binding failed: modifiers on a mutation RETURN clause is not supported in the initial graph slice at byte 119..168
- `age.cypher.merge.query-246`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 28..75; mutation execution failed: Cypher mutation binding failed: modifiers on a mutation RETURN clause is not supported in the initial graph slice at byte 148..197
- `age.cypher.merge.query-254`: expected EOI, UNION, clause, or relationship_pattern at byte 37..37
- `age.cypher.merge.query-255`: expected EOI, UNION, clause, or relationship_pattern at byte 37..37
- `age.cypher.merge.query-256`: expected EOI, UNION, clause, or relationship_pattern at byte 35..35
- `age.cypher.merge.query-257`: expected EOI, UNION, clause, or relationship_pattern at byte 35..35
- `age.cypher.merge.query-258`: expected EOI, UNION, clause, or relationship_pattern at byte 88..88
- `age.cypher.merge.query-259`: expected EOI, UNION, clause, or relationship_pattern at byte 88..88
- `age.cypher.merge.query-260`: expected EOI, UNION, clause, or relationship_pattern at byte 36..36
- `age.cypher.merge.query-261`: expected EOI, UNION, clause, or relationship_pattern at byte 35..35
- `age.cypher.merge.query-262`: expected EOI, UNION, clause, or relationship_pattern at byte 35..35
- `age.cypher.merge.query-263`: expected EOI, UNION, clause, or relationship_pattern at byte 35..35
- `age.cypher.merge.query-264`: expected EOI, UNION, clause, or relationship_pattern at byte 37..37
- `age.cypher.merge.query-265`: expected EOI, UNION, clause, or relationship_pattern at byte 37..37
- `age.cypher.merge.query-268`: expected EOI, UNION, clause, or relationship_pattern at byte 77..77
- `age.cypher.merge.query-269`: expected EOI, UNION, clause, or relationship_pattern at byte 75..75
- `age.cypher.merge.query-270`: expected EOI, UNION, clause, or relationship_pattern at byte 60..60
- `age.cypher.merge.query-271`: expected EOI, UNION, clause, or relationship_pattern at byte 77..77
- `age.cypher.merge.query-272`: expected EOI, UNION, clause, or relationship_pattern at byte 77..77
- `age.cypher.remove.query-12`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 31..42; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.remove.query-40`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..10; mutation execution failed: Cypher mutation binding failed: unknown variable `n` at byte 7..8
- `age.cypher.remove.query-41`: expected EOI, UNION, or clause at byte 21..21
- `age.cypher.remove.query-42`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..28; mutation execution failed: Cypher mutation binding failed: unknown variable `wrong_var` at byte 17..26
- `age.cypher.set.query-17`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 19..32; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.set.query-28`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 89..110; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 93..94
- `age.cypher.set.query-29`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 97..118; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 101..102
- `age.cypher.set.query-30`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 37..52; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.set.query-34`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..30; mutation execution failed: Cypher mutation binding failed: unknown parameter `$var_name` at byte 20..30
- `age.cypher.set.query-38`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..14; mutation execution failed: Cypher mutation binding failed: unknown variable `n` at byte 4..5
- `age.cypher.set.query-39`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..29; mutation execution failed: Cypher mutation binding failed: unknown variable `wrong_var` at byte 14..23
- `age.cypher.set.query-40`: expected property_target at byte 14..14
- `age.cypher.set.query-62`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 35..55; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.set.query-66`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..111; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 20..111
- `age.cypher.set.query-68`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 35..76; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 45..76
- `age.cypher.set.query-70`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..23; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 20..23
- `age.cypher.set.query-78`: expected not_expression at byte 84..84
- `age.cypher.set.query-79`: expected property_target at byte 56..56
- `age.cypher.set.query-80`: expected property_target at byte 56..56
- `age.cypher.set.query-81`: expected property_target at byte 33..33
- `age.cypher.set.query-82`: expected property_target at byte 33..33
- `age.cypher.set.query-83`: expected property_target at byte 34..34
- `age.cypher.set.query-84`: expected property_target at byte 34..34
- `age.cypher.set.query-85`: expected property_target at byte 35..35
- `age.cypher.set.query-86`: expected property_target at byte 32..32
- `age.cypher.set.query-87`: expected property_target at byte 28..28
- `age.cypher.set.query-88`: expected property_target at byte 37..37
- `age.cypher.set.query-89`: expected property_target at byte 27..27
- `age.cypher.set.query-90`: expected property_target at byte 28..28
- `age.cypher.set.query-93`: expected property_target at byte 149..149
- `age.cypher.set.query-94`: expected identifier at byte 17..17
- `age.cypher.set.query-95`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 79..145; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 157..160
- `age.cypher.set.query-96`: expected identifier at byte 17..17
- `age.cypher.set.query-97`: expected property_target at byte 70..70
- `age.cypher.set.query-98`: expected identifier at byte 17..17
- `age.cypher.set.query-107`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 47..68; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.set.query-111`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..39; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 51..65
- `age.cypher.set.query-112`: query execution failed: Parse error: property access requires a node or relationship at byte 34..38; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..38
- `age.cypher.set.query-117`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 47..109; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.cypher.subquery.query-1`: expected not_expression at byte 23..23
- `age.cypher.subquery.query-9`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 159..159
- `age.cypher.subquery.query-10`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 159..159
- `age.cypher.subquery.query-11`: expected RETURN, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 137..137
- `age.cypher.subquery.query-12`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 124..124
- `age.cypher.subquery.query-13`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 157..157
- `age.cypher.subquery.query-16`: query execution failed: Parse error: unknown variable `b` at byte 142..144; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 142..144
- `age.cypher.subquery.query-17`: query execution failed: Parse error: unknown variable `b` at byte 170..171; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 170..171
- `age.cypher.subquery.query-19`: query execution failed: Parse error: unknown variable `b` at byte 160..178; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 160..178
- `age.cypher.subquery.query-42`: expected not_expression at byte 153..153
- `age.cypher.subquery.query-43`: expected not_expression at byte 81..81
- `age.cypher.subquery.query-44`: expected not_expression at byte 106..106
- `age.cypher.subquery.query-45`: expected primary_expression at byte 173..173
- `age.cypher.subquery.query-46`: expected primary_expression at byte 174..174
- `age.cypher.subquery.query-49`: query execution failed: Parse error: property access requires a node or relationship at byte 107..115; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 107..115
- `age.cypher.union.query-4`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 19..33; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 19..33
- `age.cypher.union.query-5`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 19..36; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 19..36
- `age.cypher.union.query-6`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 15..29; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 15..29
- `age.cypher.union.query-7`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 12..29; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 12..29
- `age.cypher.union.query-8`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 12..33; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 12..33
- `age.cypher.union.query-11`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..71; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 19..48
- `age.cypher.union.query-12`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..71; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 19..44
- `age.cypher.union.query-13`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 12..34; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 12..34
- `age.cypher.union.query-14`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..51; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 12..34
- `age.cypher.union.query-15`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..51; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 12..30
- `age.cypher.union.query-16`: query execution failed: Parse error: unknown variable `n` at byte 46..47; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 19..47
- `age.cypher.union.query-17`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..65; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 16..38
- `age.cypher.union.query-18`: query execution failed: Parse error: mixing UNION and UNION ALL in one query is not supported in the initial graph slice at byte 0..63; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 16..40
- `age.cypher.union.query-19`: query execution failed: Parse error: UNION branches with different result columns is not supported in the initial graph slice at byte 18..38; mutation execution failed: Cypher mutation binding failed: UNION in mutation queries is not supported in the initial graph slice at byte 18..38
- `age.cypher.unwind.query-7`: query execution failed: Parse error: property access requires a node or relationship at byte 65..69; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 58..74
- `age.cypher.unwind.query-9`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 70..91
- `age.cypher.unwind.query-13`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 58..82; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 62..66
- `age.cypher.vle.query-1`: expected not_expression at byte 73..73
- `age.cypher.vle.query-19`: expected not_expression at byte 31..31
- `age.cypher.vle.query-20`: expected not_expression at byte 26..26
- `age.cypher.vle.query-21`: expected not_expression at byte 24..24
- `age.cypher.vle.query-22`: expected not_expression at byte 25..25
- `age.cypher.vle.query-23`: expected not_expression at byte 24..24
- `age.cypher.vle.query-27`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..53
- `age.cypher.vle.query-28`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 53..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..58
- `age.cypher.vle.query-34`: query execution failed: Parse error: no such column: b3; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..55
- `age.cypher.vle.query-35`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 54..56; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..59
- `age.cypher.vle.query-37`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..52
- `age.cypher.vle.query-38`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..52
- `age.cypher.vle.query-39`: query execution failed: Parse error: no such column: b4; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..47
- `age.cypher.vle.query-40`: query execution failed: Parse error: no such column: b4; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..49
- `age.cypher.vle.query-41`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..47
- `age.cypher.vle.query-42`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..49
- `age.cypher.vle.query-43`: query execution failed: Parse error: no such column: b4; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 38..51
- `age.cypher.vle.query-44`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 43..52
- `age.cypher.vle.query-45`: query execution failed: Parse error: no such column: b3; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..51
- `age.cypher.vle.query-46`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..38
- `age.cypher.vle.query-47`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 59..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..64
- `age.cypher.vle.query-48`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..89
- `age.cypher.vle.query-50`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 24..83
- `age.cypher.vle.query-52`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 48..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..53
- `age.cypher.vle.query-53`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 48..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..53
- `age.cypher.vle.query-59`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-60`: query execution failed: Parse error: duplicate variable `p` at byte 19..20; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 19..20
- `age.cypher.vle.query-61`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-62`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 15..18; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 15..18
- `age.cypher.vle.query-63`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-64`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-65`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-66`: query execution failed: Parse error: duplicate variable `p` at byte 28..29; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 28..29
- `age.cypher.vle.query-67`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-68`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-69`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 25..26; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 25..26
- `age.cypher.vle.query-70`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-71`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: p; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: p
- `age.cypher.vle.query-75`: expected not_expression at byte 46..46
- `age.cypher.vle.query-76`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 20..43
- `age.cypher.vle.query-77`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 24..47
- `age.cypher.vle.query-78`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-79`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..58
- `age.cypher.vle.query-80`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..63
- `age.cypher.vle.query-81`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-82`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..58
- `age.cypher.vle.query-83`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..63
- `age.cypher.vle.query-84`: query execution failed: Parse error: property access requires a node or relationship at byte 29..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..69
- `age.cypher.vle.query-85`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-86`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..95
- `age.cypher.vle.query-87`: query execution failed: Parse error: property access requires a node or relationship at byte 25..29; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..77
- `age.cypher.vle.query-88`: query execution failed: Parse error: property access requires a node or relationship at byte 25..29; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..81
- `age.cypher.vle.query-89`: query execution failed: Parse error: property access requires a node or relationship at byte 25..29; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..87
- `age.cypher.vle.query-91`: query execution failed: Parse error: property access requires a node or relationship at byte 24..28; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 24..28
- `age.cypher.vle.query-93`: query execution failed: Parse error: property access requires a node or relationship at byte 24..28; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 24..28
- `age.cypher.vle.query-102`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 59..85; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 59..85
- `age.cypher.vle.query-103`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 59..85; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 59..85
- `age.cypher.vle.query-104`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 59..85; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 59..85
- `age.cypher.vle.query-107`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 52..82; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 52..82
- `age.cypher.vle.query-110`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 52..82; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 52..82
- `age.cypher.with.query-3`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 70..120
- `age.cypher.with.query-4`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 44..44
- `age.cypher.with.query-8`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 97..124
- `age.cypher.with.query-9`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 91..165
- `age.cypher.with.query-13`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 161..199
- `age.cypher.with.query-16`: query execution failed: Parse error: unaliased WITH expressions is not supported in the initial graph slice at byte 5..15; mutation execution failed: Cypher mutation binding failed: unaliased WITH expressions is not supported in the initial graph slice at byte 5..15
- `age.cypher.with.query-17`: query execution failed: Parse error: unaliased WITH expressions is not supported in the initial graph slice at byte 27..37; mutation execution failed: Cypher mutation binding failed: unaliased WITH expressions is not supported in the initial graph slice at byte 27..37
- `age.cypher.with.query-18`: query execution failed: Parse error: unknown variable `b` at byte 44..45; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..45
- `age.cypher.with.query-19`: query execution failed: Parse error: unknown variable `end_node` at byte 177..185; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 155..190
- `age.cypher.with.query-21`: query execution failed: Parse error: unknown variable `d` at byte 156..157; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 147..157
- `age.cypher.with.query-23`: query execution failed: Parse error: unknown variable `v` at byte 74..75; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 67..87
- `age.cypher.with.query-25`: expected clause at byte 0..0
- `age.cypher.with.query-26`: expected clause at byte 0..0
- `age.cypher.with.query-28`: expected clause at byte 0..0
- `age.cypher.with.query-29`: expected clause at byte 0..0
- `age.cypher.with.query-30`: expected clause at byte 0..0
- `age.cypher.with.query-32`: expected clause at byte 0..0
- `age.cypher.with.query-34`: expected clause at byte 0..0
- `age.cypher.with.query-35`: expected clause at byte 0..0
- `age.cypher.with.query-36`: expected clause at byte 0..0
- `age.cypher.with.query-37`: expected clause at byte 0..0
- `age.cypher.with.query-39`: expected clause at byte 0..0
- `age.cypher.with.query-40`: expected clause at byte 0..0
- `age.cypher.with.query-41`: expected clause at byte 0..0
- `age.direct.field.access.query-30`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 73..98
- `age.direct.field.access.query-31`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 73..96
- `age.direct.field.access.query-33`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..51
- `age.direct.field.access.query-34`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 37..57
- `age.direct.field.access.query-35`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 73..93
- `age.direct.field.access.query-36`: query execution failed: Parse error: property access requires a node or relationship at byte 83..96; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..222
- `age.expr.query-7`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-8`: query execution failed: Parse error: property access requires a node or relationship at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..20
- `age.expr.query-9`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..22
- `age.expr.query-10`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-11`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-12`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-13`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-44`: query execution failed: Parse error: IN membership against a non-list operand is not supported in the initial graph slice at byte 15..20; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..20
- `age.expr.query-45`: query execution failed: Parse error: IN membership against a non-list operand is not supported in the initial graph slice at byte 16..21; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..21
- `age.expr.query-62`: query execution failed: Parse error: slicing a non-list operand is not supported in the initial graph slice at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-63`: query execution failed: Parse error: slicing a non-list operand is not supported in the initial graph slice at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-88`: expected identifier, node_labels, not_expression, or map_literal at byte 8..8
- `age.expr.query-92`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-93`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-95`: expected identifier, node_labels, not_expression, or map_literal at byte 8..8
- `age.expr.query-98`: expected not_expression at byte 23..23
- `age.expr.query-99`: expected not_expression at byte 23..23
- `age.expr.query-100`: expected not_expression at byte 23..23
- `age.expr.query-101`: expected not_expression at byte 23..23
- `age.expr.query-102`: expected not_expression at byte 23..23
- `age.expr.query-154`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 13..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-155`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 16..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-156`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 16..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-157`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 12..16; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-158`: query execution failed: Parse error: NOT on a non-boolean operand is not supported in the initial graph slice at byte 11..12; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-159`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 16..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-160`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 13..14; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-161`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 18..19; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..34
- `age.expr.query-162`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 15..19; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-163`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 16..24; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-164`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 17..27; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-165`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 16..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-166`: expected not_expression or dotdot at byte 184..184
- `age.expr.query-167`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 90..97; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..97
- `age.expr.query-168`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 90..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..99
- `age.expr.query-169`: query execution failed: Parse error: property access requires a node or relationship at byte 86..87; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..97
- `age.expr.query-170`: query execution failed: Parse error: property access requires a node or relationship at byte 86..87; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..97
- `age.expr.query-171`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-172`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-173`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-174`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-175`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-176`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-177`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-178`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-179`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-183`: expected primary_expression at byte 31..31
- `age.expr.query-184`: expected primary_expression at byte 31..31
- `age.expr.query-185`: expected primary_expression at byte 31..31
- `age.expr.query-186`: expected primary_expression at byte 31..31
- `age.expr.query-197`: integer literal is outside the supported i64 range at byte 7..27
- `age.expr.query-198`: integer literal is outside the supported i64 range at byte 7..27
- `age.expr.query-215`: query execution failed: Parse error: property access requires a node or relationship at byte 8..66; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..76
- `age.expr.query-216`: query execution failed: Parse error: property access requires a node or relationship at byte 8..78; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..83
- `age.expr.query-217`: query execution failed: Parse error: property access requires a node or relationship at byte 8..78; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..88
- `age.expr.query-222`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-238`: query execution failed: Parse error: property access requires a node or relationship at byte 8..71; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..85
- `age.expr.query-239`: query execution failed: Parse error: property access requires a node or relationship at byte 8..71; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..74
- `age.expr.query-240`: query execution failed: Parse error: property access requires a node or relationship at byte 8..71; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..83
- `age.expr.query-251`: query execution failed: Parse error: property access requires a node or relationship at byte 8..64; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..76
- `age.expr.query-252`: query execution failed: Parse error: property access requires a node or relationship at byte 8..78; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..83
- `age.expr.query-253`: query execution failed: Parse error: property access requires a node or relationship at byte 8..78; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..90
- `age.expr.query-268`: query execution failed: Parse error: property access requires a node or relationship at byte 8..64; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..80
- `age.expr.query-269`: query execution failed: Parse error: property access requires a node or relationship at byte 8..82; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..87
- `age.expr.query-270`: query execution failed: Parse error: property access requires a node or relationship at byte 8..82; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..94
- `age.expr.query-273`: query execution failed: Parse error: property access requires a node or relationship at byte 8..71; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..85
- `age.expr.query-283`: expected not_expression at byte 20..20
- `age.expr.query-284`: expected not_expression at byte 30..30
- `age.expr.query-285`: expected not_expression at byte 13..13
- `age.expr.query-286`: expected not_expression at byte 20..20
- `age.expr.query-287`: expected not_expression at byte 28..28
- `age.expr.query-288`: expected not_expression at byte 28..28
- `age.expr.query-289`: expected not_expression at byte 28..28
- `age.expr.query-290`: expected not_expression at byte 13..13
- `age.expr.query-291`: expected not_expression at byte 13..13
- `age.expr.query-292`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 13..19; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-293`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 13..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-294`: expected not_expression at byte 21..21
- `age.expr.query-295`: expected not_expression at byte 20..20
- `age.expr.query-296`: expected not_expression at byte 11..11
- `age.expr.query-297`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 11..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-298`: expected not_expression at byte 21..21
- `age.expr.query-299`: expected not_expression at byte 20..20
- `age.expr.query-300`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 11..15; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-301`: expected not_expression at byte 23..23
- `age.expr.query-302`: expected not_expression at byte 31..31
- `age.expr.query-303`: expected not_expression at byte 23..23
- `age.expr.query-304`: expected not_expression at byte 23..23
- `age.expr.query-305`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 11..15; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-306`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 13..17; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-309`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..27
- `age.expr.query-310`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..30
- `age.expr.query-316`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..27
- `age.expr.query-333`: query execution failed: Parse error: no such function: id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-334`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..34
- `age.expr.query-335`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..21
- `age.expr.query-336`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..28
- `age.expr.query-337`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-338`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..32
- `age.expr.query-339`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-340`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..26
- `age.expr.query-341`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-342`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..55
- `age.expr.query-343`: query execution failed: Parse error: no such function: startNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..22
- `age.expr.query-344`: query execution failed: Parse error: no such function: startNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..29
- `age.expr.query-345`: query execution failed: Parse error: no such function: startNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-346`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..51
- `age.expr.query-347`: query execution failed: Parse error: no such function: endNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..20
- `age.expr.query-348`: query execution failed: Parse error: no such function: endNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..27
- `age.expr.query-349`: query execution failed: Parse error: no such function: endNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-350`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..30
- `age.expr.query-351`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-352`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..24
- `age.expr.query-353`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-355`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..32
- `age.expr.query-356`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 57..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..64
- `age.expr.query-357`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-358`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..33
- `age.expr.query-359`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-360`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..27
- `age.expr.query-361`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..54
- `age.expr.query-363`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-368`: query execution failed: Parse error: no such function: size; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-369`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..79
- `age.expr.query-377`: query execution failed: Parse error: no such function: head; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-378`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..55
- `age.expr.query-386`: query execution failed: Parse error: no such function: last; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-387`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..55
- `age.expr.query-389`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..30
- `age.expr.query-390`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..36
- `age.expr.query-391`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..23
- `age.expr.query-392`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..23
- `age.expr.query-393`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-396`: expected not_expression at byte 22..22
- `age.expr.query-400`: query execution failed: Parse error: coalesce function with less than 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..21
- `age.expr.query-401`: query execution failed: Parse error: coalesce function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-404`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-405`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-406`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-410`: query execution failed: Parse error: no such function: toBoolean; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-412`: expected identifier or not_expression at byte 22..22
- `age.expr.query-413`: expected identifier or not_expression at byte 22..22
- `age.expr.query-417`: expected identifier or not_expression at byte 22..22
- `age.expr.query-418`: expected identifier or not_expression at byte 23..23
- `age.expr.query-419`: query execution failed: Parse error: unknown variable `fail` at byte 21..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..26
- `age.expr.query-420`: expected DISTINCT, not_expression, or star_argument at byte 21..21
- `age.expr.query-424`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-425`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-426`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-427`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-430`: query execution failed: Parse error: no such function: toFloat; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-440`: query execution failed: Parse error: unknown variable `failed` at byte 20..26; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-441`: expected DISTINCT, not_expression, or star_argument at byte 19..19
- `age.expr.query-445`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-446`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-447`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-448`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-451`: query execution failed: Parse error: no such function: toInteger; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-457`: expected identifier or not_expression at byte 22..22
- `age.expr.query-459`: query execution failed: Parse error: no such function: toIntegerList; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.expr.query-460`: expected not_expression or dotdot at byte 24..24
- `age.expr.query-461`: expected not_expression at byte 30..30
- `age.expr.query-462`: expected not_expression at byte 30..30
- `age.expr.query-465`: query execution failed: Parse error: length function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-467`: query execution failed: Parse error: no such function: toString; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-474`: query execution failed: Parse error: unknown variable `b` at byte 27..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-475`: query execution failed: Parse error: unknown variable `test` at byte 21..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-476`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-478`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-486`: expected not_expression at byte 35..35
- `age.expr.query-489`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..53
- `age.expr.query-492`: query execution failed: Internal error: expected 1 argument(s), got 0; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-498`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-500`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-504`: query execution failed: Parse error: no such function: toUpper; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-506`: query execution failed: Parse error: no such function: toLower; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-507`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-508`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-509`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-513`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-514`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-515`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-522`: query execution failed: Parse error: ltrim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-523`: query execution failed: Parse error: rtrim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-524`: query execution failed: Parse error: trim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-525`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-526`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-527`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-530`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-531`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-532`: query execution failed: Parse error: no such function: left; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-536`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-537`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-538`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-541`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-542`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-543`: query execution failed: Parse error: no such function: right; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-547`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-548`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-549`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-550`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-560`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-561`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-562`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-563`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.expr.query-564`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-565`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-566`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-567`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-568`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-569`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-570`: query execution failed: Parse error: no such function: split; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-571`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-572`: expected not_expression at byte 19..19
- `age.expr.query-573`: expected not_expression at byte 24..24
- `age.expr.query-574`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-575`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-576`: query execution failed: Parse error: no such function: split; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-577`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-578`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-579`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-580`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-581`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-582`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-583`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-584`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-586`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-587`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-588`: query execution failed: Parse error: wrong number of arguments to function replace(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-589`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-590`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-591`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-592`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-596`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-600`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-601`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-602`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-603`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-604`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-605`: query execution failed: Parse error: sin function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-606`: query execution failed: Parse error: cos function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-607`: query execution failed: Parse error: tan function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-608`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-623`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-624`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-625`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-626`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-627`: expected not_expression at byte 16..16
- `age.expr.query-628`: query execution failed: Parse error: asin function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-629`: query execution failed: Parse error: acos function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-630`: query execution failed: Parse error: atan function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-631`: query execution failed: Parse error: atan2 function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-632`: query execution failed: Parse error: atan2 function called with not exactly 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-639`: query execution failed: Parse error: pi function with arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-640`: query execution failed: Parse error: pi function with arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-651`: query execution failed: Parse error: radians function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-652`: query execution failed: Parse error: degrees function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-653`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-654`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.expr.query-688`: query execution failed: Parse error: abs function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-689`: query execution failed: Parse error: ceil function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-690`: query execution failed: Parse error: floor function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-691`: query execution failed: Parse error: round function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-692`: query execution failed: Parse error: sign function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-693`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-694`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-695`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-696`: expected DISTINCT, not_expression, or star_argument at byte 13..13
- `age.expr.query-697`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-709`: query execution failed: Parse error: log function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-710`: query execution failed: Parse error: log10 function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-711`: query execution failed: Parse error: no such function: e; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.expr.query-712`: query execution failed: Parse error: no such function: e; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-716`: query execution failed: Parse error: exp function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-717`: expected DISTINCT, not_expression, or star_argument at byte 11..11
- `age.expr.query-723`: query execution failed: Parse error: sqrt function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-724`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-725`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..37
- `age.expr.query-726`: expected DISTINCT or not_expression at byte 23..23
- `age.expr.query-727`: query execution failed: Parse error: invalid resolved function or parameter name: ag_catalog.age_sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..30
- `age.expr.query-728`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..39
- `age.expr.query-729`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-730`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..37
- `age.expr.query-731`: expected DISTINCT or not_expression at byte 33..33
- `age.expr.query-732`: expected projection_items at byte 15..15
- `age.expr.query-733`: query execution failed: Parse error: invalid resolved function or parameter name: contains.age_sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-734`: expected not_expression at byte 25..25
- `age.expr.query-735`: expected not_expression at byte 25..25
- `age.expr.query-736`: expected not_expression at byte 25..25
- `age.expr.query-737`: expected not_expression at byte 25..25
- `age.expr.query-738`: expected not_expression at byte 25..25
- `age.expr.query-739`: expected not_expression at byte 25..25
- `age.expr.query-740`: expected not_expression at byte 25..25
- `age.expr.query-742`: query execution failed: Parse error: misuse of aggregate function sum(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..88
- `age.expr.query-743`: expected not_expression at byte 25..25
- `age.expr.query-744`: expected not_expression at byte 25..25
- `age.expr.query-746`: query execution failed: Parse error: misuse of aggregate function sum(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..88
- `age.expr.query-750`: query execution failed: Parse error: aggregate calls without exactly one argument is not supported in the initial graph slice at byte 7..12; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-751`: query execution failed: Parse error: aggregate calls without exactly one argument is not supported in the initial graph slice at byte 7..12; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-752`: query execution failed: Parse error: aggregate calls without exactly one argument is not supported in the initial graph slice at byte 7..14; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-759`: query execution failed: Parse error: wrong number of arguments to function min(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-760`: query execution failed: Parse error: wrong number of arguments to function max(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-761`: query execution failed: Parse error: no such function: stDev; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..44
- `age.expr.query-762`: query execution failed: Parse error: no such function: stDev; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-763`: query execution failed: Parse error: no such function: stDevP; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-764`: query execution failed: Parse error: no such function: stDev; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-765`: query execution failed: Parse error: no such function: stDevP; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-766`: expected not_expression at byte 39..39
- `age.expr.query-767`: expected not_expression at byte 39..39
- `age.expr.query-768`: expected not_expression at byte 39..39
- `age.expr.query-769`: expected not_expression at byte 28..28
- `age.expr.query-770`: expected not_expression at byte 28..28
- `age.expr.query-771`: expected DISTINCT, not_expression, or star_argument at byte 22..22
- `age.expr.query-772`: expected DISTINCT, not_expression, or star_argument at byte 22..22
- `age.expr.query-778`: expected primary_expression at byte 24..24
- `age.expr.query-779`: query execution failed: Parse error: aggregate calls without exactly one argument is not supported in the initial graph slice at byte 7..16; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-780`: expected not_expression at byte 25..25
- `age.expr.query-793`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 40..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..71
- `age.expr.query-794`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 54..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..73
- `age.expr.query-798`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 30..39; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..61
- `age.expr.query-799`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 25..34; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..56
- `age.expr.query-807`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..28; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 12..26
- `age.expr.query-816`: query execution failed: Parse error: unknown variable `name` at byte 76..80; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..80
- `age.expr.query-817`: query execution failed: Parse error: unknown variable `name` at byte 76..81; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..84
- `age.expr.query-818`: query execution failed: Parse error: unknown variable `name` at byte 76..81; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..85
- `age.expr.query-819`: query execution failed: Parse error: unknown variable `age` at byte 76..80; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..100
- `age.expr.query-820`: query execution failed: Parse error: unknown variable `age` at byte 76..80; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..100
- `age.expr.query-824`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 19..21
- `age.expr.query-830`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 38..39; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..45
- `age.expr.query-846`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 47..60; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..82
- `age.expr.query-847`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 57..70; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..92
- `age.expr.query-852`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..120
- `age.expr.query-853`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..113
- `age.expr.query-854`: query execution failed: Parse error: generated relational SQL did not parse: near "(": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..123
- `age.expr.query-855`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 47..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..83
- `age.expr.query-863`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..33; mutation execution failed: Cypher mutation binding failed: RETURN * after mutation clauses is not supported in the initial graph slice at byte 33..41
- `age.expr.query-865`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 13..22; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..22
- `age.expr.query-866`: expected DISTINCT or projection_items at byte 7..7
- `age.expr.query-867`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 60..69; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..72
- `age.expr.query-868`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..72
- `age.expr.query-873`: expected not_expression at byte 73..73
- `age.expr.query-874`: query execution failed: Parse error: no such function: vertex_stats; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..32
- `age.expr.query-875`: query execution failed: Parse error: property access requires a node or relationship at byte 16..31; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 16..31
- `age.expr.query-876`: query execution failed: Parse error: property access requires a node or relationship at byte 16..31; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 16..31
- `age.expr.query-877`: query execution failed: Parse error: property access requires a node or relationship at byte 16..31; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 16..31
- `age.expr.query-878`: expected not_expression at byte 82..82
- `age.expr.query-879`: expected not_expression at byte 57..57
- `age.expr.query-881`: expected not_expression at byte 115..115
- `age.expr.query-889`: expected DISTINCT, not_expression, or star_argument at byte 12..12
- `age.expr.query-891`: expected not_expression at byte 16..16
- `age.expr.query-892`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..84; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.expr.query-896`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-897`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.expr.query-898`: expected DISTINCT, not_expression, or star_argument at byte 23..23
- `age.expr.query-899`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..25
- `age.expr.query-900`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..32
- `age.expr.query-904`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..26
- `age.expr.query-905`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.expr.query-906`: expected DISTINCT, not_expression, or star_argument at byte 31..31
- `age.expr.query-907`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..33
- `age.expr.query-908`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..40
- `age.expr.query-920`: query execution failed: Extension error: Invalid Argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-922`: query execution failed: Parse error: range over non-integer arguments is not supported in the initial graph slice at byte 7..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-924`: expected identifier or not_expression at byte 13..13
- `age.expr.query-928`: query execution failed: Parse error: unknown variable `abc` at byte 12..15; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-929`: query execution failed: Parse error: no such function: tail; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-930`: expected not_expression at byte 24..24
- `age.expr.query-931`: expected not_expression at byte 24..24
- `age.expr.query-932`: expected not_expression at byte 22..22
- `age.expr.query-933`: expected not_expression at byte 22..22
- `age.expr.query-935`: query execution failed: Parse error: no such function: labels; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-936`: expected DISTINCT, not_expression, or star_argument at byte 14..14
- `age.expr.query-949`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 24..1021; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.expr.query-950`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 24..1030; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1
- `age.expr.query-951`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.pg_typeof; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..53
- `age.expr.query-956`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 60..76
- `age.expr.query-957`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 60..76
- `age.expr.query-958`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 60..76
- `age.expr.query-959`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 74..90
- `age.expr.query-960`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..52
- `age.expr.query-961`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 71..87
- `age.expr.query-962`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 103..119
- `age.expr.query-963`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..103
- `age.expr.query-964`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..52
- `age.expr.query-965`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 71..87
- `age.expr.query-966`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 87..103
- `age.expr.query-967`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 95..119
- `age.expr.query-968`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..75
- `age.expr.query-969`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 67..83
- `age.expr.query-970`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 65..81
- `age.expr.query-971`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 72..88
- `age.expr.query-972`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..55
- `age.expr.query-973`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 80..105
- `age.expr.query-974`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 67..83
- `age.expr.query-975`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 74..90
- `age.expr.query-976`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 77..93
- `age.expr.query-977`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..104
- `age.expr.query-994`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 7..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..61
- `age.expr.query-995`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 7..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..45
- `age.expr.query-996`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..44
- `age.expr.query-997`: query execution failed: Parse error: no such function: delete_global_graphs; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.expr.query-998`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 7..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..44
- `age.expr.query-999`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..43
- `age.expr.query-1000`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..46
- `age.expr.query-1004`: expected identifier at byte 39..39
- `age.expr.query-1016`: expected MATCH or path_pattern at byte 35..35
- `age.expr.query-1017`: expected MATCH or path_pattern at byte 29..29
- `age.expr.query-1018`: expected MATCH or path_pattern at byte 35..35
- `age.expr.query-1019`: expected MATCH or path_pattern at byte 26..26
- `age.expr.query-1020`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..93; mutation execution failed: Cypher mutation binding failed: unknown variable `x` at byte 65..67
- `age.expr.query-1023`: expected clause at byte 0..0
- `age.expr.query-1024`: expected clause at byte 0..0
- `age.expr.query-1026`: expected clause at byte 0..0
- `age.expr.query-1027`: expected clause at byte 0..0
- `age.expr.query-1029`: expected clause at byte 0..0
- `age.expr.query-1030`: expected clause at byte 0..0
- `age.expr.query-1031`: expected clause at byte 0..0
- `age.expr.query-1032`: expected clause at byte 0..0
- `age.expr.query-1033`: expected clause at byte 0..0
- `age.expr.query-1034`: expected clause at byte 0..0
- `age.expr.query-1035`: expected clause at byte 0..0
- `age.expr.query-1036`: expected clause at byte 0..0
- `age.expr.query-1037`: expected clause at byte 0..0
- `age.expr.query-1038`: expected clause at byte 0..0
- `age.expr.query-1039`: expected clause at byte 0..0
- `age.expr.query-1040`: expected clause at byte 0..0
- `age.expr.query-1041`: expected clause at byte 0..0
- `age.expr.query-1042`: expected clause at byte 0..0
- `age.expr.query-1043`: expected clause at byte 0..0
- `age.expr.query-1045`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..61
- `age.expr.query-1046`: expected clause at byte 0..0
- `age.expr.query-1047`: expected clause at byte 0..0
- `age.expr.query-1048`: expected clause at byte 0..0
- `age.expr.query-1049`: expected clause at byte 0..0
- `age.expr.query-1050`: expected clause at byte 0..0
- `age.expr.query-1051`: expected clause at byte 0..0
- `age.expr.query-1052`: expected clause at byte 0..0
- `age.expr.query-1053`: expected clause at byte 0..0
- `age.expr.query-1054`: expected clause at byte 0..0
- `age.expr.query-1055`: expected clause at byte 0..0
- `age.expr.query-1056`: expected clause at byte 0..0
- `age.expr.query-1087`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..33; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 21..30
- `age.expr.query-1088`: expected property_target at byte 21..21
- `age.fuzzystrmatch.query-1`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.fuzzystrmatch.query-2`: expected DISTINCT, not_expression, or star_argument at byte 15..15
- `age.fuzzystrmatch.query-3`: expected not_expression at byte 23..23
- `age.fuzzystrmatch.query-5`: expected not_expression at byte 37..37
- `age.fuzzystrmatch.query-6`: expected not_expression at byte 36..36
- `age.fuzzystrmatch.query-7`: query execution failed: Parse error: no such function: metaphone; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..37
- `age.fuzzystrmatch.query-8`: query execution failed: Parse error: no such function: dmetaphone; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..35
- `age.fuzzystrmatch.query-9`: expected not_expression at byte 47..47
- `age.fuzzystrmatch.query-10`: expected not_expression at byte 47..47
- `age.fuzzystrmatch.query-11`: expected not_expression at byte 47..47
- `age.index.query-32`: expected not_expression at byte 26..26
- `age.index.query-34`: expected clause at byte 0..0
- `age.index.query-36`: expected clause at byte 0..0
- `age.index.query-37`: expected clause at byte 0..0
- `age.index.query-38`: expected clause at byte 0..0
- `age.index.query-40`: expected not_expression at byte 31..31
- `age.index.query-42`: expected clause at byte 0..0
- `age.index.query-45`: expected clause at byte 0..0
- `age.index.query-49`: expected clause at byte 0..0
- `age.index.query-54`: expected clause at byte 0..0
- `age.index.query-57`: expected clause at byte 0..0
- `age.index.query-61`: expected clause at byte 0..0
- `age.index.query-64`: expected clause at byte 0..0
- `age.jsonb.operators.query-1`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..68; mutation execution failed: Cypher mutation binding failed: map literal outside a struct or union property is not supported in the initial graph slice at byte 36..66
- `age.jsonb.operators.query-2`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-3`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-4`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-5`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-6`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-7`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-8`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-9`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-10`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-11`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-12`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-13`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-14`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-15`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-16`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-17`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-18`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-19`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-20`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-21`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-22`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-23`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-24`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-25`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 25..25
- `age.jsonb.operators.query-26`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-27`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-28`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-29`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-30`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-31`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-32`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-33`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 25..25
- `age.jsonb.operators.query-34`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-35`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-36`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-37`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-38`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-39`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-40`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-41`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-42`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-43`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-44`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-45`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-46`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-47`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-48`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-49`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-50`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.jsonb.operators.query-51`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-52`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-53`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-54`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-55`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-56`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-57`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-58`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-59`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-60`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-61`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-62`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-63`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-64`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-65`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-66`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-67`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-68`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-69`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-70`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-71`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-72`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-73`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-74`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-75`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-76`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-77`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-78`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-79`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-80`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-81`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-82`: expected primary_expression at byte 17..17
- `age.jsonb.operators.query-83`: expected primary_expression at byte 17..17
- `age.jsonb.operators.query-84`: expected primary_expression at byte 17..17
- `age.jsonb.operators.query-85`: expected primary_expression at byte 126..126
- `age.jsonb.operators.query-86`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-87`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-88`: expected primary_expression at byte 17..17
- `age.jsonb.operators.query-89`: expected primary_expression at byte 19..19
- `age.jsonb.operators.query-90`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-91`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-92`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-93`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-94`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-95`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 102..102
- `age.jsonb.operators.query-96`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 102..102
- `age.jsonb.operators.query-97`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-98`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-99`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-100`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-101`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-102`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.jsonb.operators.query-103`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.jsonb.operators.query-104`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 32..32
- `age.jsonb.operators.query-105`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 32..32
- `age.jsonb.operators.query-106`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-107`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-108`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-109`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-110`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-111`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-112`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-113`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-114`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-115`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-116`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-117`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-118`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-119`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-120`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 19..19
- `age.jsonb.operators.query-121`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 95..95
- `age.jsonb.operators.query-122`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `age.jsonb.operators.query-123`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 31..31
- `age.jsonb.operators.query-124`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 38..38
- `age.jsonb.operators.query-125`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 13..13
- `age.jsonb.operators.query-126`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 12..12
- `age.jsonb.operators.query-127`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 12..12
- `age.jsonb.operators.query-128`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 12..12
- `age.jsonb.operators.query-129`: expected EOI, WHERE, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 28..28
- `age.jsonb.operators.query-130`: expected EOI, WHERE, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 28..28
- `age.jsonb.operators.query-131`: expected EOI, WHERE, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 33..33
- `age.jsonb.operators.query-132`: expected EOI, WHERE, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 33..33
- `age.jsonb.operators.query-133`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 18..18
- `age.jsonb.operators.query-134`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 18..18
- `age.jsonb.operators.query-135`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 28..28
- `age.jsonb.operators.query-136`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-137`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-138`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-139`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-140`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 12..12
- `age.jsonb.operators.query-141`: expected EOI, WHERE, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-142`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 24..24
- `age.jsonb.operators.query-143`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 22..22
- `age.jsonb.operators.query-144`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-145`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-146`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-147`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-148`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-149`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 30..30
- `age.jsonb.operators.query-150`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 22..22
- `age.jsonb.operators.query-151`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 40..40
- `age.jsonb.operators.query-152`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 40..40
- `age.jsonb.operators.query-153`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 27..27
- `age.jsonb.operators.query-154`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 30..30
- `age.jsonb.operators.query-155`: expected not_expression at byte 30..30
- `age.jsonb.operators.query-156`: expected not_expression at byte 28..28
- `age.jsonb.operators.query-157`: expected primary_expression at byte 25..25
- `age.jsonb.operators.query-158`: expected not_expression at byte 28..28
- `age.jsonb.operators.query-159`: expected primary_expression at byte 24..24
- `age.list.comprehension.query-42`: expected EOI, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 22..22
- `age.list.comprehension.query-58`: expected primary_expression at byte 62..62
- `age.list.comprehension.query-64`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 43..51
- `age.list.comprehension.query-67`: expected property_target at byte 45..45
- `age.list.comprehension.query-68`: expected property_target at byte 74..74
- `age.list.comprehension.query-73`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..24; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q
- `age.list.comprehension.query-74`: query execution failed: Parse error: unknown variable `x` at byte 40..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.list.comprehension.query-75`: query execution failed: Parse error: unknown variable `x` at byte 41..42; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..55
- `age.list.comprehension.query-76`: query execution failed: Parse error: unknown variable `n` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 23..54
- `age.list.comprehension.query-77`: query execution failed: Parse error: unknown variable `x` at byte 60..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..67
- `age.list.comprehension.query-88`: query execution failed: Parse error: unknown variable `i` at byte 30..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.list.comprehension.query-89`: query execution failed: Parse error: unknown variable `i` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..48
- `age.list.comprehension.query-90`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 29..29
- `age.list.comprehension.query-91`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 29..29
- `age.list.comprehension.query-92`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-93`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-96`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 23..23
- `age.list.comprehension.query-97`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-100`: expected primary_expression at byte 54..54
- `age.list.comprehension.query-101`: expected node_labels or map_literal at byte 9..9
- `age.list.comprehension.query-102`: expected node_labels or map_literal at byte 9..9
- `age.list.comprehension.query-104`: expected node_labels or map_literal at byte 9..9
- `age.list.comprehension.query-105`: expected node_labels or map_literal at byte 9..9
- `age.list.comprehension.query-107`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 43..51
- `age.list.comprehension.query-108`: expected property_target at byte 75..75
- `age.list.comprehension.query-121`: expected not_expression at byte 28..28
- `age.map.projection.query-2`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-3`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-4`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-5`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 59..59
- `age.map.projection.query-6`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-7`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-8`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-9`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-10`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 59..59
- `age.map.projection.query-11`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-12`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 28..28
- `age.map.projection.query-13`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 25..25
- `age.map.projection.query-14`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 35..35
- `age.map.projection.query-15`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.map.projection.query-16`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 26..26
- `age.map.projection.query-17`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `age.map.projection.query-18`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 61..61
- `age.name.validation.query-5`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.name.validation.query-6`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.name.validation.query-7`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.name.validation.query-8`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.name.validation.query-9`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..94
- `age.name.validation.query-10`: query execution failed: Parse error: no such function: is_valid_label_name; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..93
- `age.pattern.expression.query-15`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 36..60; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..96
- `age.pattern.expression.query-16`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 36..59; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..109
- `age.pattern.expression.query-19`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 21..67; mutation execution failed: Cypher mutation binding failed: modifiers on a mutation RETURN clause is not supported in the initial graph slice at byte 67..113
- `age.pattern.expression.query-20`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 42..66; mutation execution failed: Cypher mutation binding failed: pattern expressions in projections is not supported in the initial graph slice at byte 42..66
- `age.pg.trgm.query-1`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.pg.trgm.query-2`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.pg.trgm.query-3`: expected not_expression at byte 23..23
- `age.pg.trgm.query-4`: query execution failed: Parse error: no such function: show_trgm; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..34
- `age.pg.trgm.query-5`: expected not_expression at byte 37..37
- `age.pg.trgm.query-6`: expected not_expression at byte 42..42
- `age.pgvector.query-1`: expected DISTINCT, not_expression, or star_argument at byte 23..23
- `age.pgvector.query-2`: expected DISTINCT, not_expression, or star_argument at byte 23..23
- `age.pgvector.query-3`: expected DISTINCT or projection_items at byte 7..7
- `age.pgvector.query-4`: expected DISTINCT or projection_items at byte 7..7
- `age.pgvector.query-5`: expected DISTINCT or projection_items at byte 7..7
- `age.pgvector.query-6`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-7`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-8`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-9`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-10`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-11`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-12`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-13`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-14`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 25..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.pgvector.query-15`: call target is not a plain or namespaced function name at byte 7..34
- `age.pgvector.query-16`: call target is not a plain or namespaced function name at byte 7..34
- `age.pgvector.query-17`: call target is not a plain or namespaced function name at byte 7..34
- `age.pgvector.query-18`: expected DISTINCT, not_expression, or star_argument at byte 19..19
- `age.pgvector.query-19`: expected DISTINCT, not_expression, or star_argument at byte 21..21
- `age.pgvector.query-20`: expected DISTINCT, not_expression, or star_argument at byte 23..23
- `age.pgvector.query-21`: expected DISTINCT, not_expression, or star_argument at byte 19..19
- `age.pgvector.query-22`: expected DISTINCT, not_expression, or star_argument at byte 19..19
- `age.pgvector.query-23`: expected DISTINCT, not_expression, or star_argument at byte 19..19
- `age.pgvector.query-24`: expected DISTINCT, not_expression, or star_argument at byte 20..20
- `age.pgvector.query-25`: expected DISTINCT, not_expression, or star_argument at byte 20..20
- `age.pgvector.query-26`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.pgvector.query-27`: expected DISTINCT, not_expression, or star_argument at byte 17..17
- `age.pgvector.query-28`: expected DISTINCT, not_expression, or star_argument at byte 23..23
- `age.pgvector.query-29`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 16..22; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.pgvector.query-30`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 16..22; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.pgvector.query-31`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 16..22; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.pgvector.query-32`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-33`: expected primary_expression at byte 24..24
- `age.pgvector.query-34`: expected primary_expression at byte 25..25
- `age.pgvector.query-35`: expected primary_expression at byte 24..24
- `age.pgvector.query-36`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-37`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-38`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-39`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-40`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-41`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-42`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-43`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-44`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-45`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-46`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-47`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-48`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-49`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-50`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-51`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-52`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 23..23
- `age.pgvector.query-53`: expected not_expression at byte 23..23
- `age.pgvector.query-54`: query execution failed: Parse error: casts to this type name is not supported in the initial graph slice at byte 47..53; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..53
- `age.pgvector.query-55`: query execution failed: Parse error: no such function: vector_dims; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..56
- `age.pgvector.query-56`: expected not_expression at byte 39..39
- `age.pgvector.query-57`: expected not_expression at byte 39..39
- `age.pgvector.query-58`: expected not_expression at byte 39..39
- `age.pgvector.query-59`: expected not_expression at byte 39..39
- `age.pgvector.query-60`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 16..66; mutation execution failed: Cypher mutation binding failed: casts to this type name is not supported in the initial graph slice at byte 59..65
- `age.pgvector.query-61`: expected not_expression at byte 39..39
- `age.pgvector.query-62`: expected not_expression at byte 39..39
- `age.pgvector.query-63`: expected not_expression at byte 39..39
- `age.pgvector.query-64`: expected not_expression at byte 39..39
- `age.pgvector.query-65`: expected not_expression at byte 39..39
- `age.pgvector.query-66`: expected primary_expression at byte 81..81
- `age.pgvector.query-67`: expected primary_expression at byte 81..81
- `age.pgvector.query-68`: expected not_expression at byte 39..39
- `age.pgvector.query-69`: expected not_expression at byte 39..39
- `age.pgvector.query-70`: expected primary_expression at byte 81..81
- `age.pgvector.query-71`: expected primary_expression at byte 81..81
- `age.predicate.functions.query-43`: query execution failed: Parse error: unknown variable `x` at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.predicate.functions.query-44`: query execution failed: Parse error: unknown variable `x` at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.predicate.functions.query-45`: query execution failed: Parse error: unknown variable `x` at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.predicate.functions.query-46`: query execution failed: Parse error: unknown variable `x` at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47
- `age.predicate.functions.query-47`: query execution failed: Parse error: unknown variable `x` at byte 40..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..48
- `age.predicate.functions.query-48`: query execution failed: Parse error: unknown variable `x` at byte 40..41; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..48
- `age.predicate.functions.query-49`: query execution failed: Parse error: unknown variable `x` at byte 42..43; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..50
- `age.predicate.functions.query-50`: query execution failed: Parse error: unknown variable `x` at byte 42..43; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..50
- `age.predicate.functions.query-51`: query execution failed: Parse error: unknown variable `x` at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..77
- `age.predicate.functions.query-52`: query execution failed: Parse error: unknown variable `x` at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..80
- `age.predicate.functions.query-53`: query execution failed: Parse error: unknown variable `x` at byte 66..67; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..78
- `age.predicate.functions.query-54`: query execution failed: Parse error: unknown variable `x` at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..77
- `age.predicate.functions.query-55`: query execution failed: Parse error: unknown variable `x` at byte 62..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..81
- `age.predicate.functions.query-56`: query execution failed: Parse error: unknown variable `x` at byte 62..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..78
- `age.predicate.functions.query-57`: query execution failed: Parse error: unknown variable `x` at byte 64..65; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..79
- `age.predicate.functions.query-58`: query execution failed: Parse error: unknown variable `x` at byte 69..70; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..81
- `age.predicate.functions.query-61`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 58..59; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..61
- `age.reserved.keyword.alias.query-4`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-5`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-6`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-7`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-8`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-9`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-10`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-11`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-20`: expected identifier at byte 37..37
- `age.reserved.keyword.alias.query-23`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-24`: expected identifier at byte 12..12
- `age.reserved.keyword.alias.query-25`: expected identifier at byte 12..12
- `age.scan.query-2`: expected clause at byte 0..0
- `age.scan.query-10`: integer literal is outside the supported i64 range at byte 7..28
- `age.scan.query-18`: integer literal is outside the supported i64 range at byte 26..44
- `age.scan.query-19`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 10..10
- `age.scan.query-20`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 8..8
- `age.scan.query-21`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 8..8
- `age.scan.query-22`: expected identifier at byte 10..10
- `age.scan.query-23`: expected identifier at byte 9..9
- `age.scan.query-24`: expected not_expression at byte 31..31
- `age.scan.query-25`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 14..14
- `age.scan.query-26`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 15..15
- `age.scan.query-27`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-28`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-29`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-34`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-35`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-36`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-37`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-38`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-39`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-40`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-41`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-42`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-43`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-44`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-45`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-46`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-47`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-48`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-49`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 8..8
- `age.scan.query-50`: query execution failed: Parse error: unknown variable `A` at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..8
- `age.scan.query-51`: query execution failed: Parse error: unknown variable `z` at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..8
- `age.scan.query-52`: query execution failed: Parse error: unknown variable `$` at byte 7..10; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.scan.query-53`: query execution failed: Parse error: unknown variable `0` at byte 7..10; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.scan.query-54`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 9..9
- `age.scan.query-55`: query execution failed: Parse error: unknown variable `` at byte 7..9; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..9
- `age.scan.query-56`: expected DISTINCT or projection_items at byte 7..7
- `age.scan.query-57`: query execution failed: Parse error: unknown parameter `$0` at byte 7..9; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..9
- `age.security.query-23`: expected property_target at byte 39..39
- `age.security.query-45`: query execution failed: Parse error: no such function: endNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..44
- `age.security.query-46`: query execution failed: Parse error: no such function: startNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..46
- `age.security.query-47`: query execution failed: Parse error: property access requires a node or relationship at byte 34..46; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..68
- `age.security.query-131`: query execution failed: Parse error: no such function: endNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 52..69
- `age.security.query-132`: query execution failed: Parse error: no such function: startNode; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 50..69
- `age.security.query-133`: query execution failed: Parse error: property access requires a node or relationship at byte 84..96; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 77..118

## Latest `cqlite-deep` run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Commit: `aeb0c662831c736bacd67f988d1a6a878f60a196` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 137
- Passed: 113
- Unsupported: 0
- Failed or changed: 24

### Outcome changes from `20260719T210524.711499Z-51b9a1bd28bf-corpus-deep`

- No outcome changes.

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `cqlite.basic-queries.run-a-to-b.query-1` | `Conformance` | basic_queries | `Passed` | 7.978 ms |
| `cqlite.basic-queries.run-a-to-b.query-2` | `Conformance` | basic_queries | `Passed` | 7.480 ms |
| `cqlite.basic-queries.run-a-to-b.query-3` | `Conformance` | basic_queries | `Passed` | 7.782 ms |
| `cqlite.basic-queries.run-a-edge-b.query-1` | `Conformance` | basic_queries | `Passed` | 8.338 ms |
| `cqlite.basic-queries.run-a-edge-b.query-2` | `Conformance` | basic_queries | `Passed` | 7.593 ms |
| `cqlite.basic-queries.run-a-to-a.query-1` | `Conformance` | basic_queries | `Passed` | 9.013 ms |
| `cqlite.basic-queries.run-a-to-a.query-2` | `Conformance` | basic_queries | `Passed` | 7.906 ms |
| `cqlite.basic-queries.run-a-edge-a.query-1` | `Conformance` | basic_queries | `Passed` | 9.948 ms |
| `cqlite.basic-queries.run-a-edge-a.query-2` | `Conformance` | basic_queries | `Passed` | 7.856 ms |
| `cqlite.basic-queries.run-a-knows-b.query-1` | `Conformance` | basic_queries | `Passed` | 9.228 ms |
| `cqlite.basic-queries.run-a-knows-b.query-2` | `Conformance` | basic_queries | `Passed` | 7.695 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-1` | `Conformance` | basic_queries | `Passed` | 9.474 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-2` | `Conformance` | basic_queries | `Passed` | 8.619 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-1` | `Conformance` | basic_queries | `Passed` | 8.908 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-2` | `Conformance` | basic_queries | `Passed` | 8.208 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-1` | `Conformance` | basic_queries | `Passed` | 8.017 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-2` | `Conformance` | basic_queries | `Passed` | 7.229 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-1` | `Conformance` | basic_queries | `Passed` | 8.563 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-2` | `Conformance` | basic_queries | `Failed` | 7.405 ms |
| `cqlite.basic-queries.run-set.query-1` | `Conformance` | basic_queries | `Passed` | 7.364 ms |
| `cqlite.basic-queries.run-set.query-2` | `Conformance` | basic_queries | `Failed` | 7.163 ms |
| `cqlite.basic-queries.run-set.query-3` | `Conformance` | basic_queries | `Passed` | 7.622 ms |
| `cqlite.basic-queries.return-from-set.query-1` | `Conformance` | basic_queries | `Passed` | 7.615 ms |
| `cqlite.basic-queries.return-from-set.query-2` | `Conformance` | basic_queries | `Passed` | 7.652 ms |
| `cqlite.basic-queries.return-from-set.query-3` | `Conformance` | basic_queries | `Passed` | 7.570 ms |
| `cqlite.basic-queries.run-delete-node.query-1` | `Conformance` | basic_queries | `Passed` | 7.423 ms |
| `cqlite.basic-queries.run-delete-node.query-2` | `Conformance` | basic_queries | `Passed` | 7.032 ms |
| `cqlite.basic-queries.run-delete-node.query-3` | `Conformance` | basic_queries | `Passed` | 7.094 ms |
| `cqlite.basic-queries.run-delete-edge.query-1` | `Conformance` | basic_queries | `Passed` | 7.877 ms |
| `cqlite.basic-queries.run-delete-edge.query-2` | `Conformance` | basic_queries | `Passed` | 7.343 ms |
| `cqlite.basic-queries.run-delete-edge.query-3` | `Conformance` | basic_queries | `Passed` | 7.348 ms |
| `cqlite.basic-queries.run-bad-delete.query-1` | `Conformance` | basic_queries | `Passed` | 8.004 ms |
| `cqlite.basic-queries.run-bad-delete.query-2` | `Conformance` | basic_queries | `Passed` | 6.889 ms |
| `cqlite.basic-queries.run-return-label.query-1` | `Conformance` | basic_queries | `Passed` | 8.236 ms |
| `cqlite.basic-queries.run-return-label.query-2` | `Conformance` | basic_queries | `Failed` | 7.838 ms |
| `cqlite.basic-queries.match-return-count.query-1` | `Conformance` | basic_queries | `Passed` | 7.892 ms |
| `cqlite.basic-queries.match-return-count.query-2` | `Conformance` | basic_queries | `Passed` | 7.683 ms |
| `cqlite.basic-queries.match-return-count.query-3` | `Conformance` | basic_queries | `Passed` | 7.292 ms |
| `cqlite.basic-queries.match-multiple-edges.query-1` | `Conformance` | basic_queries | `Passed` | 9.463 ms |
| `cqlite.basic-queries.match-multiple-edges.query-2` | `Conformance` | basic_queries | `Failed` | 0.037 ms |
| `cqlite.create-queries.create-label-only.query-1` | `Conformance` | create_queries | `Passed` | 8.325 ms |
| `cqlite.create-queries.create-label-only.query-2` | `Conformance` | create_queries | `Passed` | 7.271 ms |
| `cqlite.create-queries.create-with-properties.query-1` | `Conformance` | create_queries | `Passed` | 8.774 ms |
| `cqlite.create-queries.create-with-properties.query-2` | `Conformance` | create_queries | `Passed` | 7.805 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-1` | `Conformance` | create_queries | `Failed` | 7.694 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-2` | `Conformance` | create_queries | `Passed` | 7.825 ms |
| `cqlite.create-queries.create-edges-with-label.query-1` | `Conformance` | create_queries | `Passed` | 9.227 ms |
| `cqlite.create-queries.create-edges-with-label.query-2` | `Conformance` | create_queries | `Failed` | 7.827 ms |
| `cqlite.delete-queries.delete-node.query-1` | `Conformance` | delete_queries | `Passed` | 7.369 ms |
| `cqlite.delete-queries.delete-node.query-2` | `Conformance` | delete_queries | `Passed` | 7.012 ms |
| `cqlite.delete-queries.delete-node.query-3` | `Conformance` | delete_queries | `Passed` | 6.920 ms |
| `cqlite.delete-queries.delete-node.query-4` | `Conformance` | delete_queries | `Passed` | 6.915 ms |
| `cqlite.delete-queries.double-delete-node.query-1` | `Conformance` | delete_queries | `Passed` | 10.968 ms |
| `cqlite.delete-queries.double-delete-node.query-2` | `Conformance` | delete_queries | `Passed` | 8.349 ms |
| `cqlite.delete-queries.double-delete-node.query-3` | `Conformance` | delete_queries | `Passed` | 7.399 ms |
| `cqlite.delete-queries.double-delete-node.query-4` | `Conformance` | delete_queries | `Passed` | 7.720 ms |
| `cqlite.delete-queries.delete-edge.query-1` | `Conformance` | delete_queries | `Passed` | 9.125 ms |
| `cqlite.delete-queries.delete-edge.query-2` | `Conformance` | delete_queries | `Failed` | 0.046 ms |
| `cqlite.delete-queries.delete-edge.query-3` | `Conformance` | delete_queries | `Passed` | 7.487 ms |
| `cqlite.delete-queries.delete-edge.query-4` | `Conformance` | delete_queries | `Failed` | 0.005 ms |
| `cqlite.delete-queries.connected-delete-fails.query-1` | `Conformance` | delete_queries | `Passed` | 9.104 ms |
| `cqlite.delete-queries.connected-delete-fails.query-2` | `Conformance` | delete_queries | `Passed` | 7.445 ms |
| `cqlite.delete-queries.connected-delete-fails.query-3` | `Conformance` | delete_queries | `Passed` | 7.651 ms |
| `cqlite.delete-queries.connected-delete-fails.query-4` | `Conformance` | delete_queries | `Passed` | 7.133 ms |
| `cqlite.match-queries.create-test-graph.query-1` | `Conformance` | match_queries | `Passed` | 11.921 ms |
| `cqlite.match-queries.match-all-nodes.query-1` | `Conformance` | match_queries | `Passed` | 7.317 ms |
| `cqlite.match-queries.match-multiple-nodes.query-1` | `Conformance` | match_queries | `Passed` | 7.389 ms |
| `cqlite.match-queries.match-multiple-nodes.query-2` | `Conformance` | match_queries | `Passed` | 7.139 ms |
| `cqlite.match-queries.match-single-directed-edge.query-1` | `Conformance` | match_queries | `Passed` | 8.267 ms |
| `cqlite.match-queries.match-single-undirected-edge.query-1` | `Conformance` | match_queries | `Passed` | 7.628 ms |
| `cqlite.match-queries.match-single-path.query-1` | `Conformance` | match_queries | `Failed` | 0.036 ms |
| `cqlite.match-queries.match-path-with-multiple-clauses.query-1` | `Conformance` | match_queries | `Failed` | 0.029 ms |
| `cqlite.match-queries.match-long-path.query-1` | `Conformance` | match_queries | `Failed` | 0.027 ms |
| `cqlite.match-queries.match-labeled-nodes.query-1` | `Conformance` | match_queries | `Passed` | 7.262 ms |
| `cqlite.match-queries.match-labeled-nodes.query-2` | `Conformance` | match_queries | `Passed` | 6.999 ms |
| `cqlite.match-queries.match-labeled-nodes.query-3` | `Conformance` | match_queries | `Passed` | 7.087 ms |
| `cqlite.match-queries.match-labeled-edges.query-1` | `Conformance` | match_queries | `Passed` | 7.792 ms |
| `cqlite.match-queries.match-labeled-edges.query-2` | `Conformance` | match_queries | `Passed` | 7.484 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-1` | `Conformance` | match_queries | `Passed` | 7.566 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-2` | `Conformance` | match_queries | `Passed` | 7.189 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-3` | `Conformance` | match_queries | `Passed` | 7.831 ms |
| `cqlite.match-queries.match-edges-with-properties.query-1` | `Conformance` | match_queries | `Passed` | 7.762 ms |
| `cqlite.match-queries.match-nodes-with-label.query-1` | `Conformance` | match_queries | `Passed` | 7.075 ms |
| `cqlite.match-queries-where.create-test-graph.query-1` | `Conformance` | match_queries_where | `Passed` | 16.108 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 7.384 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-2` | `Conformance` | match_queries_where | `Failed` | 7.443 ms |
| `cqlite.match-queries-where.match-where-node-id-eq-non-id.query-1` | `Conformance` | match_queries_where | `Passed` | 7.179 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-1` | `Conformance` | match_queries_where | `Passed` | 7.012 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-2` | `Conformance` | match_queries_where | `Passed` | 7.276 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 7.249 ms |
| `cqlite.match-queries-where.match-where-node-prop.query-1` | `Conformance` | match_queries_where | `Passed` | 7.669 ms |
| `cqlite.match-queries-where.match-where-not-node-prop.query-1` | `Conformance` | match_queries_where | `Passed` | 7.439 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-1` | `Conformance` | match_queries_where | `Passed` | 7.473 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-2` | `Conformance` | match_queries_where | `Passed` | 7.446 ms |
| `cqlite.match-queries-where.match-where-node-prop-ne-null.query-1` | `Conformance` | match_queries_where | `Passed` | 7.436 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-1` | `Conformance` | match_queries_where | `Passed` | 7.813 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-2` | `Conformance` | match_queries_where | `Passed` | 8.205 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-3` | `Conformance` | match_queries_where | `Passed` | 8.069 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 7.908 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-2` | `Conformance` | match_queries_where | `Failed` | 7.649 ms |
| `cqlite.match-queries-where.match-where-edge-prop-eq.query-1` | `Conformance` | match_queries_where | `Passed` | 8.160 ms |
| `cqlite.match-queries-where.match-where-edge-prop-gt.query-1` | `Conformance` | match_queries_where | `Passed` | 8.471 ms |
| `cqlite.match-queries-where.match-where-a-or-b.query-1` | `Conformance` | match_queries_where | `Passed` | 8.445 ms |
| `cqlite.match-queries-where.match-long-path-with-id-constraint.query-1` | `Conformance` | match_queries_where | `Failed` | 0.038 ms |
| `cqlite.match-queries-where.match-long-path-with-id-constraint.query-2` | `Conformance` | match_queries_where | `Failed` | 0.031 ms |
| `cqlite.match-queries-where.match-short-path-with-id-constraint.query-1` | `Conformance` | match_queries_where | `Failed` | 0.026 ms |
| `cqlite.return-queries.return-parameter.query-1` | `Conformance` | return_queries | `Failed` | 7.067 ms |
| `cqlite.return-queries.return-id-of.query-1` | `Conformance` | return_queries | `Passed` | 8.416 ms |
| `cqlite.return-queries.return-id-of.query-2` | `Conformance` | return_queries | `Passed` | 7.203 ms |
| `cqlite.return-queries.return-label-of.query-1` | `Conformance` | return_queries | `Passed` | 8.344 ms |
| `cqlite.return-queries.return-label-of.query-2` | `Conformance` | return_queries | `Passed` | 7.256 ms |
| `cqlite.return-queries.create-and-return.query-1` | `Conformance` | return_queries | `Passed` | 7.921 ms |
| `cqlite.return-queries.create-and-return.query-2` | `Conformance` | return_queries | `Passed` | 7.109 ms |
| `cqlite.return-queries.set-and-return.query-1` | `Conformance` | return_queries | `Passed` | 7.552 ms |
| `cqlite.return-queries.set-and-return.query-2` | `Conformance` | return_queries | `Passed` | 10.208 ms |
| `cqlite.return-queries.delete-and-return.query-1` | `Conformance` | return_queries | `Passed` | 7.701 ms |
| `cqlite.return-queries.delete-and-return.query-2` | `Conformance` | return_queries | `Passed` | 7.269 ms |
| `cqlite.return-queries.return-out-of-bounds.query-1` | `Conformance` | return_queries | `Passed` | 7.018 ms |
| `cqlite.set-queries.set-once.query-1` | `Conformance` | set_queries | `Passed` | 7.812 ms |
| `cqlite.set-queries.set-once.query-2` | `Conformance` | set_queries | `Passed` | 7.155 ms |
| `cqlite.set-queries.set-once.query-3` | `Conformance` | set_queries | `Passed` | 6.935 ms |
| `cqlite.set-queries.set-after-create.query-1` | `Conformance` | set_queries | `Passed` | 8.162 ms |
| `cqlite.set-queries.set-after-create.query-2` | `Conformance` | set_queries | `Passed` | 7.064 ms |
| `cqlite.set-queries.set-multiple-times.query-1` | `Conformance` | set_queries | `Passed` | 8.886 ms |
| `cqlite.set-queries.set-multiple-times.query-2` | `Conformance` | set_queries | `Passed` | 7.188 ms |
| `cqlite.set-queries.delete-property.query-1` | `Conformance` | set_queries | `Passed` | 7.481 ms |
| `cqlite.set-queries.delete-property.query-2` | `Conformance` | set_queries | `Passed` | 7.086 ms |
| `cqlite.set-queries.delete-property.query-3` | `Conformance` | set_queries | `Passed` | 7.020 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-1` | `Conformance` | txn_semantics | `Passed` | 7.855 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-2` | `Conformance` | txn_semantics | `Passed` | 7.287 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-3` | `Conformance` | txn_semantics | `Passed` | 7.662 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-4` | `Conformance` | txn_semantics | `Passed` | 7.306 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-5` | `Conformance` | txn_semantics | `Passed` | 7.377 ms |
| `cqlite.where-conditions.where-a-and-b.query-1` | `Conformance` | where_conditions | `Failed` | 0.020 ms |
| `cqlite.where-conditions.where-a-or-b.query-1` | `Conformance` | where_conditions | `Failed` | 0.011 ms |
| `cqlite.where-conditions.where-a.query-1` | `Conformance` | where_conditions | `Failed` | 0.009 ms |
| `cqlite.where-conditions.where-not-a.query-1` | `Conformance` | where_conditions | `Failed` | 0.010 ms |

## Latest `deep` run

- Run: `20260718T013941.952713Z-e1d73880b749-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 34
- Passed: 29
- Unsupported: 5
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `tck.with.with1.scenario-1` | `Conformance` | scope | `Passed` | 1.099 ms |
| `grafeo.match.directed-edge` | `Conformance` | match | `Passed` | 0.899 ms |
| `age.vle.zero-length` | `Conformance` | traversal | `Passed` | 2.688 ms |
| `pggraph.traversal.exact-two-hops` | `Conformance` | traversal | `Passed` | 2.352 ms |
| `sparrow.path.two-hop-multiplicity` | `Conformance` | traversal | `Passed` | 2.116 ms |
| `sparrow.merge.existing-node` | `Conformance` | mutation | `Passed` | 0.641 ms |
| `cqlite.match.labeled-node-scan` | `Conformance` | match | `Passed` | 0.511 ms |
| `cqlite.create.properties` | `Conformance` | mutation | `Passed` | 1.188 ms |
| `samyama.aggregate.global-count` | `Conformance` | aggregation | `Passed` | 0.321 ms |
| `age.vle.unbounded-traversal` | `Conformance` | traversal | `Passed` | 2.113 ms |
| `tck.call.subquery-scope` | `Conformance` | subquery | `Unsupported` | 0.020 ms |
| `grafeo.path.all-shortest-paths` | `Conformance` | shortest-path | `Unsupported` | 0.135 ms |
| `pggraph.path.weight-expression` | `Conformance` | shortest-path | `Unsupported` | 0.147 ms |
| `sparrow.path.shortest-function` | `Conformance` | shortest-path | `Unsupported` | 0.139 ms |
| `samyama.planner.independent-patterns` | `Conformance` | planning | `Unsupported` | 0.183 ms |
| `grafeo.match.incoming-edge` | `Conformance` | match | `Passed` | 0.710 ms |
| `age.vle.fixed-multi-hop` | `Conformance` | traversal | `Passed` | 1.976 ms |
| `grafeo.with.projected-expression` | `Conformance` | scope | `Passed` | 0.419 ms |
| `age.unwind.literal-list` | `Conformance` | unwind | `Passed` | 0.442 ms |
| `grafeo.order-by.nonprojected-property` | `Conformance` | ordering | `Passed` | 0.360 ms |
| `grafeo.pagination.skip-limit` | `Conformance` | pagination | `Passed` | 0.427 ms |
| `grafeo.optional.where-null-extends-pattern` | `Conformance` | optional-match | `Passed` | 0.804 ms |
| `tck.where.numeric-comparison` | `Conformance` | filter | `Passed` | 0.583 ms |
| `cqlite.set.property` | `Conformance` | mutation | `Passed` | 1.185 ms |
| `age.remove.property` | `Conformance` | mutation | `Passed` | 0.933 ms |
| `age.delete.relationship` | `Conformance` | mutation | `Passed` | 1.538 ms |
| `sparrow.merge.absent-node` | `Conformance` | mutation | `Passed` | 0.894 ms |
| `grafeo.regression.wrong-relationship-direction` | `BugRegression` | match | `Passed` | 0.666 ms |
| `age.regression.zero-length-preserves-identity` | `Regression` | traversal | `Passed` | 2.141 ms |
| `sparrow.regression.missing-property-is-null` | `BugRegression` | null | `Passed` | 0.430 ms |
| `sparrow.regression.variable-path-terminal-label` | `BugRegression` | traversal | `Passed` | 2.035 ms |
| `cqlite.regression.parameterized-property` | `Regression` | parameters | `Passed` | 0.376 ms |
| `grafeo.regression.optional-count-preserves-rows` | `BugRegression` | aggregation | `Passed` | 0.614 ms |
| `turso.regression.constraint-index-drop-error` | `BugRegression` | mutation | `Passed` | 0.431 ms |

## Latest `grafeo-deep` run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Commit: `aeb0c662831c736bacd67f988d1a6a878f60a196` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 399
- Passed: 253
- Unsupported: 0
- Failed or changed: 146

### Outcome changes from `20260719T210524.711499Z-51b9a1bd28bf-corpus-deep`

- No outcome changes.

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `grafeo.spec.common.index.correctness.create.index.then.query` | `Conformance` | common | `Failed` | 0.025 ms |
| `grafeo.spec.common.index.correctness.index.query.no.match` | `Conformance` | common | `Failed` | 0.018 ms |
| `grafeo.spec.common.index.correctness.index.multiple.matches` | `Conformance` | common | `Failed` | 0.017 ms |
| `grafeo.spec.common.index.correctness.index.with.null.property` | `Conformance` | common | `Failed` | 0.017 ms |
| `grafeo.spec.common.index.correctness.index.after.property.update` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.index.correctness.index.old.value.gone.after.update` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.index.correctness.index.after.delete` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.index.correctness.index.remaining.after.delete` | `Conformance` | common | `Failed` | 0.002 ms |
| `grafeo.spec.common.index.correctness.index.reinsert.after.delete` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.index.correctness.numeric.index.exact.lookup` | `Conformance` | common | `Failed` | 0.016 ms |
| `grafeo.spec.common.index.correctness.numeric.index.range.query` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.index.correctness.bulk.insert.then.index` | `Conformance` | common | `Failed` | 0.328 ms |
| `grafeo.spec.common.index.correctness.index.count.all` | `Conformance` | common | `Failed` | 0.016 ms |
| `grafeo.spec.common.index.correctness.drop.index.query.still.works` | `Conformance` | common | `Failed` | 0.003 ms |
| `grafeo.spec.common.null.semantics.negative.limit.returns.empty.cypher.cypher-variant` | `Conformance` | common | `Failed` | 6.769 ms |
| `grafeo.spec.common.numeric.edge.cases.min.int64.cypher.cypher-variant` | `Conformance` | common | `Failed` | 6.698 ms |
| `grafeo.spec.common.numeric.edge.cases.nan.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Failed` | 7.261 ms |
| `grafeo.spec.common.numeric.edge.cases.inf.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Failed` | 10.853 ms |
| `grafeo.spec.lpg.cypher.admin.create.index.on.label.property` | `Conformance` | lpg | `Failed` | 0.005 ms |
| `grafeo.spec.lpg.cypher.admin.create.index.and.query` | `Conformance` | lpg | `Failed` | 0.026 ms |
| `grafeo.spec.lpg.cypher.admin.drop.index` | `Conformance` | lpg | `Failed` | 0.003 ms |
| `grafeo.spec.lpg.cypher.admin.show.indexes.empty` | `Conformance` | lpg | `Failed` | 0.012 ms |
| `grafeo.spec.lpg.cypher.admin.show.indexes.after.create` | `Conformance` | lpg | `Failed` | 0.004 ms |
| `grafeo.spec.lpg.cypher.admin.explain.match` | `Conformance` | lpg | `Failed` | 0.015 ms |
| `grafeo.spec.lpg.cypher.admin.profile.match` | `Conformance` | lpg | `Failed` | 0.012 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.basic` | `Conformance` | lpg | `Failed` | 0.124 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.filter` | `Conformance` | lpg | `Failed` | 0.110 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.size` | `Conformance` | lpg | `Failed` | 0.174 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.property.extraction` | `Conformance` | lpg | `Failed` | 0.103 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.basic` | `Conformance` | lpg | `Passed` | 77.323 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.transform` | `Conformance` | lpg | `Passed` | 65.974 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.filter.and.transform` | `Conformance` | lpg | `Failed` | 64.832 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.nested` | `Conformance` | lpg | `Failed` | 75.162 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.exists.subquery.actors.with.action.movies` | `Conformance` | lpg | `Passed` | 68.232 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.not.exists.subquery` | `Conformance` | lpg | `Failed` | 68.848 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.movies.per.actor` | `Conformance` | lpg | `Passed` | 68.092 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.prolific.directors` | `Conformance` | lpg | `Passed` | 66.804 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.basic` | `Conformance` | lpg | `Failed` | 0.045 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.with.aggregation` | `Conformance` | lpg | `Failed` | 0.035 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.set.property` | `Conformance` | lpg | `Failed` | 66.000 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.create.relationships` | `Conformance` | lpg | `Failed` | 0.281 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.actor.collaboration.via.comprehension` | `Conformance` | lpg | `Failed` | 0.142 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.genre.diversity.per.actor` | `Conformance` | lpg | `Failed` | 0.201 ms |
| `grafeo.spec.lpg.cypher.constraints.create.unique.constraint` | `Conformance` | lpg | `Failed` | 0.022 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.allows.distinct.values` | `Conformance` | lpg | `Failed` | 7.104 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.violation` | `Conformance` | lpg | `Failed` | 6.878 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.null.allowed` | `Conformance` | lpg | `Failed` | 7.094 ms |
| `grafeo.spec.lpg.cypher.constraints.create.not.null.constraint` | `Conformance` | lpg | `Failed` | 0.026 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.satisfied` | `Conformance` | lpg | `Failed` | 6.984 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation` | `Conformance` | lpg | `Failed` | 6.886 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation.on.set` | `Conformance` | lpg | `Failed` | 6.875 ms |
| `grafeo.spec.lpg.cypher.constraints.create.node.key.constraint` | `Conformance` | lpg | `Failed` | 0.026 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.allows.different.combinations` | `Conformance` | lpg | `Failed` | 6.822 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.duplicate` | `Conformance` | lpg | `Failed` | 6.809 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.missing.property` | `Conformance` | lpg | `Failed` | 6.733 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.constraint` | `Conformance` | lpg | `Failed` | 0.014 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.nonexistent.constraint` | `Conformance` | lpg | `Passed` | 0.011 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.constraint.if.exists` | `Conformance` | lpg | `Failed` | 0.012 ms |
| `grafeo.spec.lpg.cypher.constraints.show.constraints.after.create` | `Conformance` | lpg | `Failed` | 0.008 ms |
| `grafeo.spec.lpg.cypher.constraints.show.constraints.empty` | `Conformance` | lpg | `Failed` | 0.001 ms |
| `grafeo.spec.lpg.cypher.expressions.addition` | `Conformance` | lpg | `Passed` | 8.164 ms |
| `grafeo.spec.lpg.cypher.expressions.subtraction` | `Conformance` | lpg | `Passed` | 8.077 ms |
| `grafeo.spec.lpg.cypher.expressions.multiplication` | `Conformance` | lpg | `Passed` | 8.086 ms |
| `grafeo.spec.lpg.cypher.expressions.division` | `Conformance` | lpg | `Passed` | 18.890 ms |
| `grafeo.spec.lpg.cypher.expressions.modulo` | `Conformance` | lpg | `Passed` | 8.884 ms |
| `grafeo.spec.lpg.cypher.expressions.power` | `Conformance` | lpg | `Passed` | 8.207 ms |
| `grafeo.spec.lpg.cypher.expressions.unary.minus` | `Conformance` | lpg | `Failed` | 0.046 ms |
| `grafeo.spec.lpg.cypher.expressions.string.concat` | `Conformance` | lpg | `Failed` | 8.424 ms |
| `grafeo.spec.lpg.cypher.expressions.equals` | `Conformance` | lpg | `Passed` | 8.277 ms |
| `grafeo.spec.lpg.cypher.expressions.not.equals` | `Conformance` | lpg | `Passed` | 8.813 ms |
| `grafeo.spec.lpg.cypher.expressions.less.than` | `Conformance` | lpg | `Passed` | 9.076 ms |
| `grafeo.spec.lpg.cypher.expressions.greater.equal` | `Conformance` | lpg | `Passed` | 9.023 ms |
| `grafeo.spec.lpg.cypher.expressions.starts.with` | `Conformance` | lpg | `Passed` | 9.354 ms |
| `grafeo.spec.lpg.cypher.expressions.ends.with` | `Conformance` | lpg | `Passed` | 9.093 ms |
| `grafeo.spec.lpg.cypher.expressions.contains` | `Conformance` | lpg | `Passed` | 9.087 ms |
| `grafeo.spec.lpg.cypher.expressions.in.list` | `Conformance` | lpg | `Passed` | 9.870 ms |
| `grafeo.spec.lpg.cypher.expressions.regex.match` | `Conformance` | lpg | `Failed` | 0.064 ms |
| `grafeo.spec.lpg.cypher.expressions.is.null` | `Conformance` | lpg | `Passed` | 9.250 ms |
| `grafeo.spec.lpg.cypher.expressions.is.not.null` | `Conformance` | lpg | `Passed` | 8.982 ms |
| `grafeo.spec.lpg.cypher.expressions.case.simple` | `Conformance` | lpg | `Passed` | 8.316 ms |
| `grafeo.spec.lpg.cypher.expressions.case.searched` | `Conformance` | lpg | `Passed` | 8.242 ms |
| `grafeo.spec.lpg.cypher.expressions.list.literal` | `Conformance` | lpg | `Failed` | 8.318 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension` | `Conformance` | lpg | `Failed` | 9.710 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension.filter.only` | `Conformance` | lpg | `Failed` | 9.311 ms |
| `grafeo.spec.lpg.cypher.expressions.list.slice` | `Conformance` | lpg | `Failed` | 8.440 ms |
| `grafeo.spec.lpg.cypher.expressions.index.access` | `Conformance` | lpg | `Passed` | 8.222 ms |
| `grafeo.spec.lpg.cypher.expressions.coalesce` | `Conformance` | lpg | `Passed` | 8.763 ms |
| `grafeo.spec.lpg.cypher.expressions.reduce` | `Conformance` | lpg | `Failed` | 0.255 ms |
| `grafeo.spec.lpg.cypher.expressions.all.predicate` | `Conformance` | lpg | `Passed` | 8.684 ms |
| `grafeo.spec.lpg.cypher.expressions.any.predicate` | `Conformance` | lpg | `Passed` | 8.407 ms |
| `grafeo.spec.lpg.cypher.expressions.none.predicate` | `Conformance` | lpg | `Passed` | 8.424 ms |
| `grafeo.spec.lpg.cypher.expressions.single.predicate` | `Conformance` | lpg | `Passed` | 8.405 ms |
| `grafeo.spec.lpg.cypher.expressions.any.with.labels.in.where` | `Conformance` | lpg | `Passed` | 9.383 ms |
| `grafeo.spec.lpg.cypher.expressions.comparison.in.return` | `Conformance` | lpg | `Passed` | 8.050 ms |
| `grafeo.spec.lpg.cypher.expressions.aggregate.comparison.in.return` | `Conformance` | lpg | `Passed` | 8.067 ms |
| `grafeo.spec.lpg.cypher.functions.id.of.node` | `Conformance` | lpg | `Passed` | 7.934 ms |
| `grafeo.spec.lpg.cypher.functions.labels.single` | `Conformance` | lpg | `Failed` | 8.100 ms |
| `grafeo.spec.lpg.cypher.functions.labels.multiple` | `Conformance` | lpg | `Passed` | 8.567 ms |
| `grafeo.spec.lpg.cypher.functions.type.of.relationship` | `Conformance` | lpg | `Failed` | 8.982 ms |
| `grafeo.spec.lpg.cypher.functions.keys.of.node` | `Conformance` | lpg | `Failed` | 8.278 ms |
| `grafeo.spec.lpg.cypher.functions.properties.of.node` | `Conformance` | lpg | `Failed` | 15.834 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.true` | `Conformance` | lpg | `Failed` | 8.287 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.false` | `Conformance` | lpg | `Failed` | 8.001 ms |
| `grafeo.spec.lpg.cypher.functions.head.of.list` | `Conformance` | lpg | `Passed` | 8.449 ms |
| `grafeo.spec.lpg.cypher.functions.last.of.list` | `Conformance` | lpg | `Passed` | 8.351 ms |
| `grafeo.spec.lpg.cypher.functions.tail.of.list` | `Conformance` | lpg | `Failed` | 8.369 ms |
| `grafeo.spec.lpg.cypher.functions.range.default.step` | `Conformance` | lpg | `Failed` | 8.190 ms |
| `grafeo.spec.lpg.cypher.functions.range.with.step` | `Conformance` | lpg | `Failed` | 8.293 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.list` | `Conformance` | lpg | `Passed` | 8.291 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.string` | `Conformance` | lpg | `Passed` | 8.220 ms |
| `grafeo.spec.lpg.cypher.functions.to.lower` | `Conformance` | lpg | `Passed` | 8.074 ms |
| `grafeo.spec.lpg.cypher.functions.to.upper` | `Conformance` | lpg | `Passed` | 8.137 ms |
| `grafeo.spec.lpg.cypher.functions.trim.whitespace` | `Conformance` | lpg | `Passed` | 8.100 ms |
| `grafeo.spec.lpg.cypher.functions.replace.substring` | `Conformance` | lpg | `Passed` | 8.220 ms |
| `grafeo.spec.lpg.cypher.functions.substring.from.start` | `Conformance` | lpg | `Failed` | 8.193 ms |
| `grafeo.spec.lpg.cypher.functions.substring.to.end` | `Conformance` | lpg | `Failed` | 8.103 ms |
| `grafeo.spec.lpg.cypher.functions.split.string` | `Conformance` | lpg | `Failed` | 8.090 ms |
| `grafeo.spec.lpg.cypher.functions.left.string` | `Conformance` | lpg | `Passed` | 8.121 ms |
| `grafeo.spec.lpg.cypher.functions.right.string` | `Conformance` | lpg | `Passed` | 8.196 ms |
| `grafeo.spec.lpg.cypher.functions.reverse.string` | `Conformance` | lpg | `Passed` | 8.467 ms |
| `grafeo.spec.lpg.cypher.functions.abs.positive` | `Conformance` | lpg | `Passed` | 8.019 ms |
| `grafeo.spec.lpg.cypher.functions.ceil.float` | `Conformance` | lpg | `Passed` | 8.105 ms |
| `grafeo.spec.lpg.cypher.functions.floor.float` | `Conformance` | lpg | `Passed` | 8.049 ms |
| `grafeo.spec.lpg.cypher.functions.round.float` | `Conformance` | lpg | `Passed` | 7.943 ms |
| `grafeo.spec.lpg.cypher.functions.sign.positive` | `Conformance` | lpg | `Passed` | 8.028 ms |
| `grafeo.spec.lpg.cypher.functions.sign.negative` | `Conformance` | lpg | `Passed` | 8.045 ms |
| `grafeo.spec.lpg.cypher.functions.sign.zero` | `Conformance` | lpg | `Passed` | 7.947 ms |
| `grafeo.spec.lpg.cypher.functions.sqrt.perfect.square` | `Conformance` | lpg | `Passed` | 8.025 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.string` | `Conformance` | lpg | `Passed` | 8.036 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.float` | `Conformance` | lpg | `Passed` | 7.942 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.string` | `Conformance` | lpg | `Passed` | 8.026 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.integer` | `Conformance` | lpg | `Passed` | 8.021 ms |
| `grafeo.spec.lpg.cypher.functions.to.string.from.integer` | `Conformance` | lpg | `Passed` | 8.053 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.true` | `Conformance` | lpg | `Passed` | 8.249 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.false` | `Conformance` | lpg | `Passed` | 8.149 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.string` | `Conformance` | lpg | `Passed` | 8.263 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.map` | `Conformance` | lpg | `Passed` | 8.341 ms |
| `grafeo.spec.lpg.cypher.functions.datetime.from.string` | `Conformance` | lpg | `Passed` | 8.152 ms |
| `grafeo.spec.lpg.cypher.functions.duration.from.string` | `Conformance` | lpg | `Passed` | 8.014 ms |
| `grafeo.spec.lpg.cypher.functions.path.length` | `Conformance` | lpg | `Passed` | 12.679 ms |
| `grafeo.spec.lpg.cypher.functions.path.length.single.hop` | `Conformance` | lpg | `Passed` | 9.689 ms |
| `grafeo.spec.lpg.cypher.functions.collect.names` | `Conformance` | lpg | `Passed` | 8.944 ms |
| `grafeo.spec.lpg.cypher.functions.collect.distinct` | `Conformance` | lpg | `Failed` | 9.656 ms |
| `grafeo.spec.lpg.cypher.functions.count.with.distinct` | `Conformance` | lpg | `Passed` | 10.199 ms |
| `grafeo.spec.lpg.cypher.functions.sum.values` | `Conformance` | lpg | `Passed` | 9.774 ms |
| `grafeo.spec.lpg.cypher.functions.avg.values` | `Conformance` | lpg | `Passed` | 9.472 ms |
| `grafeo.spec.lpg.cypher.functions.min.values` | `Conformance` | lpg | `Passed` | 9.579 ms |
| `grafeo.spec.lpg.cypher.functions.max.values` | `Conformance` | lpg | `Passed` | 9.521 ms |
| `grafeo.spec.lpg.cypher.functions.chained.string.functions` | `Conformance` | lpg | `Passed` | 8.311 ms |
| `grafeo.spec.lpg.cypher.functions.nested.list.functions` | `Conformance` | lpg | `Passed` | 8.727 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log.of.e` | `Conformance` | lpg | `Failed` | 8.408 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log10.of.100` | `Conformance` | lpg | `Passed` | 8.071 ms |
| `grafeo.spec.lpg.cypher.functions.extended.exp.of.zero` | `Conformance` | lpg | `Passed` | 8.079 ms |
| `grafeo.spec.lpg.cypher.functions.extended.e.constant` | `Conformance` | lpg | `Failed` | 8.314 ms |
| `grafeo.spec.lpg.cypher.functions.extended.pi.constant` | `Conformance` | lpg | `Passed` | 7.999 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rand.in.range` | `Conformance` | lpg | `Passed` | 8.734 ms |
| `grafeo.spec.lpg.cypher.functions.extended.sin.of.zero` | `Conformance` | lpg | `Passed` | 8.050 ms |
| `grafeo.spec.lpg.cypher.functions.extended.cos.of.zero` | `Conformance` | lpg | `Passed` | 8.338 ms |
| `grafeo.spec.lpg.cypher.functions.extended.tan.of.zero` | `Conformance` | lpg | `Passed` | 8.150 ms |
| `grafeo.spec.lpg.cypher.functions.extended.asin.of.one` | `Conformance` | lpg | `Passed` | 8.152 ms |
| `grafeo.spec.lpg.cypher.functions.extended.acos.of.one` | `Conformance` | lpg | `Passed` | 8.287 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan.of.one` | `Conformance` | lpg | `Passed` | 8.092 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan2.unit` | `Conformance` | lpg | `Passed` | 8.086 ms |
| `grafeo.spec.lpg.cypher.functions.extended.degrees.from.pi` | `Conformance` | lpg | `Passed` | 8.031 ms |
| `grafeo.spec.lpg.cypher.functions.extended.radians.from.180` | `Conformance` | lpg | `Passed` | 8.079 ms |
| `grafeo.spec.lpg.cypher.functions.extended.ltrim.whitespace` | `Conformance` | lpg | `Passed` | 8.150 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rtrim.whitespace` | `Conformance` | lpg | `Passed` | 8.310 ms |
| `grafeo.spec.lpg.cypher.functions.extended.char.length.string` | `Conformance` | lpg | `Passed` | 8.284 ms |
| `grafeo.spec.lpg.cypher.functions.extended.length.of.string` | `Conformance` | lpg | `Passed` | 8.257 ms |
| `grafeo.spec.lpg.cypher.functions.extended.reverse.list` | `Conformance` | lpg | `Failed` | 8.290 ms |
| `grafeo.spec.lpg.cypher.functions.extended.keys.of.map` | `Conformance` | lpg | `Passed` | 8.659 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdev.sample` | `Conformance` | lpg | `Failed` | 12.313 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdevp.population` | `Conformance` | lpg | `Failed` | 12.425 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.cont.median` | `Conformance` | lpg | `Failed` | 11.043 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.disc.median` | `Conformance` | lpg | `Failed` | 10.762 ms |
| `grafeo.spec.lpg.cypher.functions.extended.element.id.not.null` | `Conformance` | lpg | `Failed` | 8.177 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.star` | `Conformance` | lpg | `Passed` | 8.668 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.expr` | `Conformance` | lpg | `Passed` | 8.713 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.path` | `Conformance` | lpg | `Passed` | 10.311 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.path` | `Conformance` | lpg | `Passed` | 10.330 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.multi.hop.path` | `Conformance` | lpg | `Passed` | 13.326 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.multi.hop.path` | `Conformance` | lpg | `Passed` | 13.316 ms |
| `grafeo.spec.lpg.cypher.functions.extended.date.no.args` | `Conformance` | lpg | `Passed` | 8.340 ms |
| `grafeo.spec.lpg.cypher.functions.extended.now.returns.value` | `Conformance` | lpg | `Failed` | 8.038 ms |
| `grafeo.spec.lpg.cypher.functions.extended.year.accessor` | `Conformance` | lpg | `Failed` | 8.140 ms |
| `grafeo.spec.lpg.cypher.functions.extended.month.accessor` | `Conformance` | lpg | `Failed` | 8.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.day.accessor` | `Conformance` | lpg | `Failed` | 8.134 ms |
| `grafeo.spec.lpg.cypher.functions.extended.time.from.string` | `Conformance` | lpg | `Passed` | 8.043 ms |
| `grafeo.spec.lpg.cypher.functions.extended.duration.from.map` | `Conformance` | lpg | `Passed` | 8.625 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.node` | `Conformance` | lpg | `Passed` | 7.723 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.binding` | `Conformance` | lpg | `Passed` | 7.696 ms |
| `grafeo.spec.lpg.cypher.patterns.single.label` | `Conformance` | lpg | `Passed` | 8.368 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.labels` | `Conformance` | lpg | `Passed` | 8.894 ms |
| `grafeo.spec.lpg.cypher.patterns.property.filter` | `Conformance` | lpg | `Passed` | 8.890 ms |
| `grafeo.spec.lpg.cypher.patterns.outgoing.relationship` | `Conformance` | lpg | `Passed` | 9.305 ms |
| `grafeo.spec.lpg.cypher.patterns.incoming.relationship` | `Conformance` | lpg | `Passed` | 9.960 ms |
| `grafeo.spec.lpg.cypher.patterns.undirected.relationship` | `Conformance` | lpg | `Passed` | 9.959 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.relationship.types` | `Conformance` | lpg | `Passed` | 12.298 ms |
| `grafeo.spec.lpg.cypher.patterns.relationship.properties` | `Conformance` | lpg | `Passed` | 9.971 ms |
| `grafeo.spec.lpg.cypher.patterns.untyped.relationship` | `Conformance` | lpg | `Failed` | 9.667 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.relationship` | `Conformance` | lpg | `Passed` | 9.377 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.unbounded` | `Conformance` | lpg | `Passed` | 12.266 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.exact` | `Conformance` | lpg | `Passed` | 13.675 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.range` | `Conformance` | lpg | `Passed` | 12.979 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.max.only` | `Conformance` | lpg | `Passed` | 12.876 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.min.only` | `Conformance` | lpg | `Passed` | 12.738 ms |
| `grafeo.spec.lpg.cypher.patterns.path.alias` | `Conformance` | lpg | `Passed` | 9.767 ms |
| `grafeo.spec.lpg.cypher.patterns.shortest.path` | `Conformance` | lpg | `Failed` | 0.033 ms |
| `grafeo.spec.lpg.cypher.patterns.all.shortest.paths` | `Conformance` | lpg | `Failed` | 0.025 ms |
| `grafeo.spec.lpg.cypher.patterns.pattern.comprehension` | `Conformance` | lpg | `Failed` | 0.118 ms |
| `grafeo.spec.lpg.cypher.patterns.exists.subquery` | `Conformance` | lpg | `Passed` | 11.108 ms |
| `grafeo.spec.lpg.cypher.patterns.not.exists` | `Conformance` | lpg | `Passed` | 9.741 ms |
| `grafeo.spec.lpg.cypher.patterns.count.subquery` | `Conformance` | lpg | `Passed` | 11.157 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.single.node` | `Conformance` | lpg | `Passed` | 24.075 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.label` | `Conformance` | lpg | `Passed` | 12.125 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.property` | `Conformance` | lpg | `Passed` | 9.030 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multi.label` | `Conformance` | lpg | `Passed` | 8.824 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.comma.patterns` | `Conformance` | lpg | `Passed` | 8.557 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multiple.clauses` | `Conformance` | lpg | `Passed` | 8.664 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.outgoing` | `Conformance` | lpg | `Passed` | 9.334 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.incoming` | `Conformance` | lpg | `Passed` | 9.513 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.undirected` | `Conformance` | lpg | `Passed` | 10.207 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.with.result` | `Conformance` | lpg | `Passed` | 9.469 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.null` | `Conformance` | lpg | `Passed` | 8.825 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.comparison` | `Conformance` | lpg | `Passed` | 9.165 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.and` | `Conformance` | lpg | `Passed` | 9.337 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.or` | `Conformance` | lpg | `Passed` | 9.597 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.not` | `Conformance` | lpg | `Passed` | 9.206 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.xor` | `Conformance` | lpg | `Passed` | 10.168 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.projection` | `Conformance` | lpg | `Passed` | 8.232 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.distinct` | `Conformance` | lpg | `Passed` | 8.968 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.where` | `Conformance` | lpg | `Passed` | 8.846 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.star` | `Conformance` | lpg | `Passed` | 9.144 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.list` | `Conformance` | lpg | `Passed` | 7.453 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.with.match` | `Conformance` | lpg | `Passed` | 8.920 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union` | `Conformance` | lpg | `Passed` | 8.905 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union.all` | `Conformance` | lpg | `Passed` | 8.378 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.labels` | `Conformance` | lpg | `Passed` | 8.549 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.relationship.types` | `Conformance` | lpg | `Passed` | 8.727 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.property.keys` | `Conformance` | lpg | `Failed` | 7.720 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.basic` | `Conformance` | lpg | `Failed` | 0.025 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.with.outer.scope` | `Conformance` | lpg | `Failed` | 0.057 ms |
| `grafeo.spec.lpg.cypher.regression.not.exists.with.type.filter` | `Conformance` | lpg | `Failed` | 11.540 ms |
| `grafeo.spec.lpg.cypher.regression.sum.case.when` | `Conformance` | lpg | `Passed` | 15.678 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.matches` | `Conformance` | lpg | `Passed` | 8.918 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.no.match` | `Conformance` | lpg | `Passed` | 9.037 ms |
| `grafeo.spec.lpg.cypher.regression.any.with.single.match` | `Conformance` | lpg | `Passed` | 8.813 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.max` | `Conformance` | lpg | `Failed` | 0.237 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.min` | `Conformance` | lpg | `Failed` | 0.221 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.conditional.sum` | `Conformance` | lpg | `Failed` | 0.218 ms |
| `grafeo.spec.lpg.cypher.regression.outgoing.target.property.filter` | `Conformance` | lpg | `Passed` | 11.673 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.count` | `Conformance` | lpg | `Passed` | 11.580 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.no.match` | `Conformance` | lpg | `Passed` | 9.355 ms |
| `grafeo.spec.lpg.cypher.regression.edge.property.filter` | `Conformance` | lpg | `Passed` | 12.149 ms |
| `grafeo.spec.lpg.cypher.regression.optional.match.count.preserves.all.rows` | `Conformance` | lpg | `Passed` | 10.421 ms |
| `grafeo.spec.lpg.cypher.regression.union.deduplicates` | `Conformance` | lpg | `Passed` | 7.678 ms |
| `grafeo.spec.lpg.cypher.regression.union.all.preserves` | `Conformance` | lpg | `Passed` | 7.077 ms |
| `grafeo.spec.lpg.cypher.regression.two.hop.equivalence` | `Conformance` | lpg | `Passed` | 12.342 ms |
| `grafeo.spec.lpg.cypher.regression.merge.creates.new.after.delete` | `Conformance` | lpg | `Passed` | 9.642 ms |
| `grafeo.spec.lpg.cypher.regression.replace.edge` | `Conformance` | lpg | `Passed` | 13.029 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.forward` | `Conformance` | lpg | `Passed` | 9.701 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.reverse` | `Conformance` | lpg | `Passed` | 9.671 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.wrong.direction` | `Conformance` | lpg | `Passed` | 9.600 ms |
| `grafeo.spec.lpg.cypher.regression.null.equals.null.is.unknown` | `Conformance` | lpg | `Passed` | 8.090 ms |
| `grafeo.spec.lpg.cypher.regression.null.is.null.is.true` | `Conformance` | lpg | `Passed` | 8.095 ms |
| `grafeo.spec.lpg.cypher.regression.bool.to.string` | `Conformance` | lpg | `Passed` | 8.612 ms |
| `grafeo.spec.lpg.cypher.regression.int.to.string` | `Conformance` | lpg | `Passed` | 8.624 ms |
| `grafeo.spec.lpg.cypher.regression.string.false.ne.bool.false` | `Conformance` | lpg | `Failed` | 13.411 ms |
| `grafeo.spec.lpg.cypher.regression.neq.excludes.null` | `Conformance` | lpg | `Passed` | 10.898 ms |
| `grafeo.spec.lpg.cypher.regression.skip.plus.limit` | `Conformance` | lpg | `Passed` | 13.985 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.values` | `Conformance` | lpg | `Passed` | 9.702 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.collapses.nulls` | `Conformance` | lpg | `Passed` | 9.824 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.property.matching.return.alias.with.edge` | `Conformance` | lpg | `Passed` | 13.363 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.desc.with.relationship.traversal` | `Conformance` | lpg | `Passed` | 13.287 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.expression` | `Conformance` | lpg | `Passed` | 8.017 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.alias` | `Conformance` | lpg | `Passed` | 7.737 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.distinct` | `Conformance` | lpg | `Passed` | 8.900 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.star` | `Conformance` | lpg | `Passed` | 7.617 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.count.star` | `Conformance` | lpg | `Passed` | 8.271 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.arithmetic` | `Conformance` | lpg | `Passed` | 7.920 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.boolean.expression` | `Conformance` | lpg | `Passed` | 7.984 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.asc` | `Conformance` | lpg | `Passed` | 8.657 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.desc` | `Conformance` | lpg | `Passed` | 8.659 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.multiple.keys` | `Conformance` | lpg | `Passed` | 10.040 ms |
| `grafeo.spec.lpg.cypher.return.ordering.limit` | `Conformance` | lpg | `Passed` | 10.984 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip` | `Conformance` | lpg | `Passed` | 10.771 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip.and.limit` | `Conformance` | lpg | `Passed` | 10.777 ms |
| `grafeo.spec.lpg.cypher.types.integer.decimal` | `Conformance` | lpg | `Passed` | 7.956 ms |
| `grafeo.spec.lpg.cypher.types.integer.negative` | `Conformance` | lpg | `Passed` | 9.292 ms |
| `grafeo.spec.lpg.cypher.types.integer.zero` | `Conformance` | lpg | `Passed` | 10.181 ms |
| `grafeo.spec.lpg.cypher.types.integer.hex` | `Conformance` | lpg | `Passed` | 8.193 ms |
| `grafeo.spec.lpg.cypher.types.integer.octal` | `Conformance` | lpg | `Passed` | 7.835 ms |
| `grafeo.spec.lpg.cypher.types.float.decimal` | `Conformance` | lpg | `Passed` | 7.890 ms |
| `grafeo.spec.lpg.cypher.types.float.scientific` | `Conformance` | lpg | `Passed` | 7.919 ms |
| `grafeo.spec.lpg.cypher.types.float.negative` | `Conformance` | lpg | `Passed` | 7.875 ms |
| `grafeo.spec.lpg.cypher.types.string.single.quoted` | `Conformance` | lpg | `Passed` | 7.902 ms |
| `grafeo.spec.lpg.cypher.types.string.double.quoted` | `Conformance` | lpg | `Failed` | 0.048 ms |
| `grafeo.spec.lpg.cypher.types.string.empty` | `Conformance` | lpg | `Passed` | 8.054 ms |
| `grafeo.spec.lpg.cypher.types.boolean.true` | `Conformance` | lpg | `Passed` | 7.891 ms |
| `grafeo.spec.lpg.cypher.types.boolean.false` | `Conformance` | lpg | `Passed` | 8.551 ms |
| `grafeo.spec.lpg.cypher.types.null.literal` | `Conformance` | lpg | `Passed` | 8.049 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.null` | `Conformance` | lpg | `Passed` | 8.021 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.not.null` | `Conformance` | lpg | `Passed` | 7.967 ms |
| `grafeo.spec.lpg.cypher.types.null.equality.returns.null` | `Conformance` | lpg | `Failed` | 7.996 ms |
| `grafeo.spec.lpg.cypher.types.missing.property.is.null` | `Conformance` | lpg | `Passed` | 8.242 ms |
| `grafeo.spec.lpg.cypher.types.list.of.integers` | `Conformance` | lpg | `Failed` | 8.004 ms |
| `grafeo.spec.lpg.cypher.types.list.empty` | `Conformance` | lpg | `Passed` | 7.984 ms |
| `grafeo.spec.lpg.cypher.types.list.nested` | `Conformance` | lpg | `Passed` | 8.306 ms |
| `grafeo.spec.lpg.cypher.types.list.size` | `Conformance` | lpg | `Passed` | 8.470 ms |
| `grafeo.spec.lpg.cypher.types.map.literal` | `Conformance` | lpg | `Passed` | 8.165 ms |
| `grafeo.spec.lpg.cypher.types.map.key.count` | `Conformance` | lpg | `Passed` | 8.576 ms |
| `grafeo.spec.lpg.cypher.types.node.return` | `Conformance` | lpg | `Passed` | 7.702 ms |
| `grafeo.spec.lpg.cypher.types.relationship.return` | `Conformance` | lpg | `Passed` | 9.468 ms |
| `grafeo.spec.lpg.cypher.types.path.return` | `Conformance` | lpg | `Passed` | 9.285 ms |
| `grafeo.spec.lpg.cypher.types.date.from.string` | `Conformance` | lpg | `Passed` | 8.062 ms |
| `grafeo.spec.lpg.cypher.types.time.from.string` | `Conformance` | lpg | `Passed` | 7.938 ms |
| `grafeo.spec.lpg.cypher.types.datetime.from.string` | `Conformance` | lpg | `Passed` | 8.214 ms |
| `grafeo.spec.lpg.cypher.types.duration.from.string` | `Conformance` | lpg | `Passed` | 8.134 ms |
| `grafeo.spec.lpg.cypher.types.date.stored.as.property` | `Conformance` | lpg | `Passed` | 8.331 ms |
| `grafeo.spec.lpg.cypher.types.integer.to.float.arithmetic` | `Conformance` | lpg | `Passed` | 7.961 ms |
| `grafeo.spec.lpg.cypher.types.to.integer.truncation` | `Conformance` | lpg | `Passed` | 8.019 ms |
| `grafeo.spec.lpg.cypher.types.to.float.from.integer` | `Conformance` | lpg | `Passed` | 8.084 ms |
| `grafeo.spec.lpg.cypher.types.to.string.from.boolean` | `Conformance` | lpg | `Failed` | 8.048 ms |
| `grafeo.spec.lpg.cypher.types.to.boolean.from.string.false` | `Conformance` | lpg | `Passed` | 8.125 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node` | `Conformance` | lpg | `Passed` | 7.630 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node.multi.label` | `Conformance` | lpg | `Passed` | 8.569 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship` | `Conformance` | lpg | `Passed` | 12.542 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship.with.properties` | `Conformance` | lpg | `Passed` | 12.181 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.path.pattern` | `Conformance` | lpg | `Passed` | 10.042 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.node` | `Conformance` | lpg | `Passed` | 8.619 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.multiple` | `Conformance` | lpg | `Passed` | 9.493 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete` | `Conformance` | lpg | `Passed` | 9.938 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete.with.return` | `Conformance` | lpg | `Passed` | 9.786 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.property` | `Conformance` | lpg | `Passed` | 8.685 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.properties` | `Conformance` | lpg | `Passed` | 9.388 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.replace.all` | `Conformance` | lpg | `Failed` | 0.121 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.merge.map` | `Conformance` | lpg | `Failed` | 0.051 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label` | `Conformance` | lpg | `Failed` | 0.048 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.labels` | `Conformance` | lpg | `Failed` | 0.048 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label.preserves.variable.binding` | `Conformance` | lpg | `Failed` | 0.047 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.star.after.set.label` | `Conformance` | lpg | `Failed` | 0.029 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.var.after.set.label` | `Conformance` | lpg | `Failed` | 0.028 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.property` | `Conformance` | lpg | `Passed` | 8.659 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label` | `Conformance` | lpg | `Failed` | 0.102 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label.preserves.variable.binding` | `Conformance` | lpg | `Failed` | 0.049 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.no.phantoms` | `Conformance` | lpg | `Passed` | 9.627 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.correct.endpoints` | `Conformance` | lpg | `Passed` | 10.146 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.create` | `Conformance` | lpg | `Passed` | 7.973 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.match` | `Conformance` | lpg | `Passed` | 8.143 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set` | `Conformance` | lpg | `Failed` | 0.051 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set` | `Conformance` | lpg | `Failed` | 0.110 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set.self.reference.increment` | `Conformance` | lpg | `Failed` | 0.048 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set.self.reference.coalesce` | `Conformance` | lpg | `Failed` | 0.048 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship` | `Conformance` | lpg | `Passed` | 10.283 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship.set` | `Conformance` | lpg | `Passed` | 11.545 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.foreach.create` | `Conformance` | lpg | `Passed` | 10.149 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.same.source.and.target.variable.cypher-variant` | `Conformance` | regression | `Failed` | 6.849 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.no.cycle.returns.empty.cypher-variant` | `Conformance` | regression | `Failed` | 6.760 ms |
| `grafeo.spec.rosetta.aggregation.count.products.cypher-variant` | `Conformance` | rosetta | `Failed` | 33.135 ms |
| `grafeo.spec.rosetta.aggregation.sum.order.totals.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.033 ms |
| `grafeo.spec.rosetta.aggregation.avg.product.price.cypher-variant` | `Conformance` | rosetta | `Failed` | 23.764 ms |
| `grafeo.spec.rosetta.aggregation.min.max.price.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 23.607 ms |
| `grafeo.spec.rosetta.aggregation.count.by.status.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.026 ms |
| `grafeo.spec.rosetta.aggregation.orders.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.043 ms |
| `grafeo.spec.rosetta.aggregation.total.spend.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.044 ms |
| `grafeo.spec.rosetta.aggregation.customers.with.multiple.orders.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.042 ms |
| `grafeo.spec.rosetta.aggregation.avg.review.rating.cypher-variant` | `Conformance` | rosetta | `Failed` | 23.640 ms |
| `grafeo.spec.rosetta.basic.queries.count.all.nodes.cypher-variant` | `Conformance` | rosetta | `Passed` | 15.474 ms |
| `grafeo.spec.rosetta.basic.queries.match.by.label.cypher-variant` | `Conformance` | rosetta | `Passed` | 15.593 ms |
| `grafeo.spec.rosetta.basic.queries.filter.by.age.cypher-variant` | `Conformance` | rosetta | `Passed` | 16.156 ms |
| `grafeo.spec.rosetta.basic.queries.edge.traversal.cypher-variant` | `Conformance` | rosetta | `Passed` | 16.425 ms |
| `grafeo.spec.rosetta.basic.queries.two.hop.path.cypher-variant` | `Conformance` | rosetta | `Passed` | 16.874 ms |
| `grafeo.spec.rosetta.basic.queries.aggregation.group.by.cypher-variant` | `Conformance` | rosetta | `Passed` | 15.994 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.and.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.725 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.read.properties.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.864 ms |
| `grafeo.spec.rosetta.crud.operations.create.edge.and.traverse.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.891 ms |
| `grafeo.spec.rosetta.crud.operations.match.count.multiple.nodes.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.661 ms |
| `grafeo.spec.rosetta.crud.operations.set.property.and.read.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.597 ms |
| `grafeo.spec.rosetta.crud.operations.delete.node.and.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.534 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.sum.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.814 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.760 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.avg.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.728 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.name.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.975 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.928 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.edge.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.651 ms |
| `grafeo.spec.rosetta.data.fidelity.int.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.760 ms |
| `grafeo.spec.rosetta.data.fidelity.bool.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.722 ms |
| `grafeo.spec.rosetta.data.fidelity.string.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.699 ms |
| `grafeo.spec.rosetta.data.fidelity.missing.property.null.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.735 ms |
| `grafeo.spec.rosetta.data.fidelity.multi.label.visible.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.709 ms |
| `grafeo.spec.rosetta.data.fidelity.edge.type.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.718 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.all.read.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 6.683 ms |
| `grafeo.spec.rosetta.pattern.matching.count.actors.cypher-variant` | `Conformance` | rosetta | `Passed` | 65.106 ms |
| `grafeo.spec.rosetta.pattern.matching.find.actor.by.name.cypher-variant` | `Conformance` | rosetta | `Passed` | 64.780 ms |
| `grafeo.spec.rosetta.pattern.matching.actors.in.heist.cypher-variant` | `Conformance` | rosetta | `Passed` | 65.717 ms |
| `grafeo.spec.rosetta.pattern.matching.genres.of.vincent.cypher-variant` | `Conformance` | rosetta | `Passed` | 66.581 ms |
| `grafeo.spec.rosetta.pattern.matching.movies.per.director.cypher.cypher-variant` | `Conformance` | rosetta | `Passed` | 66.154 ms |
| `grafeo.spec.rosetta.pattern.matching.actor.roles.in.movie.cypher-variant` | `Conformance` | rosetta | `Passed` | 79.549 ms |
| `grafeo.spec.rosetta.pattern.matching.high.rated.movies.cypher-variant` | `Conformance` | rosetta | `Passed` | 64.488 ms |

## Latest `performance-deep` run

- Run: `20260718T013944.410388Z-e1d73880b749-performance-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 10
- Passed: 10
- Unsupported: 0
- Failed or changed: 0

| Test | Operation | Scale | Outcome | Duration | Throughput/s |
|---|---|---:|---|---:|---:|
| `perf.line.create.s000100` | create | 100 | `Passed` | 40.369 ms | 2477.13 |
| `perf.line.bulk-load.s000100` | bulk-load | 100 | `Passed` | 12.613 ms | 15776.95 |
| `perf.line.load.s000100` | load | 100 | `Passed` | 3.720 ms | 53493.43 |
| `perf.line.query.s000100` | query | 100 | `Passed` | 7.510 ms | 1331.52 |
| `perf.line.delete.s000100` | delete | 100 | `Passed` | 38.866 ms | 2572.93 |
| `perf.line.create.s001000` | create | 1000 | `Passed` | 370.248 ms | 2700.89 |
| `perf.line.bulk-load.s001000` | bulk-load | 1000 | `Passed` | 282.280 ms | 7081.63 |
| `perf.line.load.s001000` | load | 1000 | `Passed` | 13.975 ms | 143038.16 |
| `perf.line.query.s001000` | query | 1000 | `Passed` | 61.859 ms | 161.66 |
| `perf.line.delete.s001000` | delete | 1000 | `Passed` | 503.315 ms | 1986.83 |

## Latest `performance-smoke` run

- Run: `20260718T013943.199409Z-e1d73880b749-performance-smoke`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 10
- Passed: 10
- Unsupported: 0
- Failed or changed: 0

| Test | Operation | Scale | Outcome | Duration | Throughput/s |
|---|---|---:|---|---:|---:|
| `perf.line.create.s000010` | create | 10 | `Passed` | 5.460 ms | 1831.53 |
| `perf.line.bulk-load.s000010` | bulk-load | 10 | `Passed` | 1.454 ms | 13064.78 |
| `perf.line.load.s000010` | load | 10 | `Passed` | 2.374 ms | 8003.79 |
| `perf.line.query.s000010` | query | 10 | `Passed` | 1.088 ms | 2756.61 |
| `perf.line.delete.s000010` | delete | 10 | `Passed` | 3.783 ms | 2643.64 |
| `perf.line.create.s000100` | create | 100 | `Passed` | 33.450 ms | 2989.58 |
| `perf.line.bulk-load.s000100` | bulk-load | 100 | `Passed` | 11.948 ms | 16655.45 |
| `perf.line.load.s000100` | load | 100 | `Passed` | 3.062 ms | 64996.40 |
| `perf.line.query.s000100` | query | 100 | `Passed` | 2.131 ms | 1407.82 |
| `perf.line.delete.s000100` | delete | 100 | `Passed` | 37.953 ms | 2634.83 |

## Latest `smoke` run

- Run: `20260718T013940.911425Z-e1d73880b749-smoke`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 11
- Passed: 11
- Unsupported: 0
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `tck.with.with1.scenario-1` | `Conformance` | scope | `Passed` | 1.219 ms |
| `grafeo.match.directed-edge` | `Conformance` | match | `Passed` | 1.133 ms |
| `age.vle.zero-length` | `Conformance` | traversal | `Passed` | 2.745 ms |
| `pggraph.traversal.exact-two-hops` | `Conformance` | traversal | `Passed` | 2.592 ms |
| `sparrow.path.two-hop-multiplicity` | `Conformance` | traversal | `Passed` | 2.105 ms |
| `sparrow.merge.existing-node` | `Conformance` | mutation | `Passed` | 0.610 ms |
| `cqlite.match.labeled-node-scan` | `Conformance` | match | `Passed` | 0.402 ms |
| `cqlite.create.properties` | `Conformance` | mutation | `Passed` | 1.104 ms |
| `samyama.aggregate.global-count` | `Conformance` | aggregation | `Passed` | 0.269 ms |
| `grafeo.regression.wrong-relationship-direction` | `BugRegression` | match | `Passed` | 0.663 ms |
| `cqlite.regression.parameterized-property` | `Regression` | parameters | `Passed` | 0.394 ms |

## Latest `sparrowdb-deep` run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Commit: `aeb0c662831c736bacd67f988d1a6a878f60a196` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 2253
- Passed: 2099
- Unsupported: 0
- Failed or changed: 154

### Outcome changes from `20260719T210524.711499Z-51b9a1bd28bf-corpus-deep`

- `sparrowdb.mcp-cypher-templates.mcp-template-add-property-count-query-parses-and-executes.query-1`: Passed
- `sparrowdb.mcp-cypher-templates.mcp-template-add-property-set-query-parses-and-executes.query-1`: Passed
- `sparrowdb.mcp-cypher-templates.mcp-template-merge-node-match-only-path-node-unchanged.query-1`: Passed
- `sparrowdb.regression-406.mixed-anonymous-rel-patterns.query-1`: Passed
- `sparrowdb.regression-406.mixed-anonymous-rel-patterns.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-3`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-4`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-5`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-6`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-7`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-8`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-9`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-10`: Passed
- `sparrowdb.spa-151-kms-query-validation.setup-kms-graph.query-11`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q02-create-knowledge-node.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q03-delete-knowledge-node.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q04-create-related-to-relationship.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q04-create-related-to-relationship.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q05-create-about-relationship.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q05-create-about-relationship.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q09-contains-predicate-on-properties.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q09-contains-predicate-on-properties.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q09-contains-predicate-on-properties.query-3`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q16-operational-nodes-by-label.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q16-operational-nodes-by-label.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q27-coalesce-function.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q27b-coalesce-with-present-properties.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q29-where-with-and-compound-filter.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q29-where-with-and-compound-filter.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q32-merge-relationship-pattern.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q32-merge-relationship-pattern.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q35-any-predicate-over-labels-in-where.query-1`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q35-any-predicate-over-labels-in-where.query-2`: Passed
- `sparrowdb.spa-151-kms-query-validation.kms-q35-any-predicate-over-labels-in-where.query-3`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-float-prop-roundtrip.query-1`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-float-prop-roundtrip.query-2`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-float-zero-does-not-panic.query-1`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-float-zero-does-not-panic.query-2`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-mixed-int-and-float-props.query-1`: Passed
- `sparrowdb.spa-229-edge-prop-float.edge-mixed-int-and-float-props.query-2`: Passed
- `sparrowdb.spa-233-merge-relationship.merge-creates-relationship-when-absent.query-1`: Passed
- `sparrowdb.spa-233-merge-relationship.merge-creates-relationship-when-absent.query-2`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-list-then-match-filters-by-id.query-1`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-list-then-match-filters-by-id.query-2`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-list-then-match-filters-by-id.query-3`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-returns-correct-row-count.query-1`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-returns-correct-row-count.query-2`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-returns-correct-row-count.query-3`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-empty-list-yields-no-rows.query-1`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-list-element-with-no-match-is-skipped.query-1`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-with-return-pipeline.query-1`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-with-return-pipeline.query-2`: Passed
- `sparrowdb.spa-237-unwind-match.unwind-match-with-return-pipeline.query-3`: Passed
- `sparrowdb.spa-240-coalesce.coalesce-with-two-props-returns-second.query-1`: Passed
- `sparrowdb.vector-index.set-vector-param-populates-hnsw.query-1`: Passed
- `sparrowdb.vector-index.set-vector-param-hnsw-roundtrip-survives-reopen.query-1`: Passed
- `sparrowdb.vector-index.anonymous-match-set-vector-populates-hnsw.query-1`: Passed
- `sparrowdb.vector-index.set-vector-hnsw-label-derived-from-node-id.query-1`: Passed
- `sparrowdb.vector-index.set-vector-hnsw-label-derived-from-node-id.query-2`: Passed

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| acceptance | `failed` | 5 |
| acceptance | `passed` | 33 |
| call_subquery | `failed` | 5 |
| call_subquery | `passed` | 13 |
| cypher_range_function_test | `passed` | 3 |
| debug_case_when | `passed` | 4 |
| debug_so_subclass | `passed` | 2 |
| delete_edge | `failed` | 1 |
| delete_edge | `passed` | 6 |
| export_import | `passed` | 19 |
| fts_index | `failed` | 8 |
| gap_10_parameterized_queries | `passed` | 19 |
| hybrid_search | `failed` | 5 |
| match_after_create | `failed` | 1 |
| match_after_create | `passed` | 6 |
| match_property_index | `passed` | 13 |
| mcp_cypher_templates | `failed` | 3 |
| mcp_cypher_templates | `passed` | 16 |
| merge_node | `passed` | 4 |
| path_semantics | `passed` | 5 |
| property_range_index | `passed` | 6 |
| readtx_query | `failed` | 1 |
| readtx_query | `passed` | 22 |
| regression_355 | `passed` | 9 |
| regression_363 | `passed` | 13 |
| regression_364 | `passed` | 3 |
| regression_366 | `failed` | 4 |
| regression_367 | `passed` | 10 |
| regression_368 | `failed` | 4 |
| regression_368 | `passed` | 5 |
| regression_369 | `failed` | 2 |
| regression_369 | `passed` | 2 |
| regression_372 | `passed` | 11 |
| regression_373 | `passed` | 18 |
| regression_379 | `failed` | 1 |
| regression_379 | `passed` | 30 |
| regression_380 | `failed` | 7 |
| regression_380 | `passed` | 8 |
| regression_406 | `passed` | 14 |
| regression_real_world | `passed` | 23 |
| reverse_arrow_294 | `passed` | 14 |
| spa157_cypher_mutations | `passed` | 5 |
| spa163_164_read_path | `passed` | 3 |
| spa191_rel_type_persistence | `passed` | 17 |
| spa_100_order_by_spill | `passed` | 6 |
| spa_111_ldbc_snb | `passed` | 2 |
| spa_119_compat_fixture | `passed` | 4 |
| spa_130_with_clause | `passed` | 9 |
| spa_131_optional_match | `passed` | 20 |
| spa_132_union | `passed` | 20 |
| spa_134_multi_clause | `passed` | 19 |
| spa_136_shortest_path | `failed` | 3 |
| spa_136_shortest_path | `passed` | 10 |
| spa_137_exists_subquery | `passed` | 12 |
| spa_138_case_when | `passed` | 9 |
| spa_139_phase9_path_acceptance | `failed` | 2 |
| spa_139_phase9_path_acceptance | `passed` | 38 |
| spa_140_143_functions | `failed` | 3 |
| spa_140_143_functions | `passed` | 25 |
| spa_148_import_bridge | `passed` | 11 |
| spa_149_visualizer | `passed` | 14 |
| spa_151_kms_query_validation | `failed` | 7 |
| spa_151_kms_query_validation | `passed` | 83 |
| spa_155_unwind_param | `passed` | 3 |
| spa_156_161 | `passed` | 16 |
| spa_165_col_prefix_property | `passed` | 9 |
| spa_168_degree_cache_wiring | `passed` | 16 |
| spa_168_match_create | `passed` | 9 |
| spa_169_string_props | `passed` | 19 |
| spa_172_count_distinct | `passed` | 17 |
| spa_178_edge_properties | `failed` | 12 |
| spa_178_edge_properties | `passed` | 15 |
| spa_182_create_path_rhs | `passed` | 5 |
| spa_183_match_create_bindings | `passed` | 16 |
| spa_185_rel_table_id | `passed` | 24 |
| spa_186_csr_nodeid | `passed` | 13 |
| spa_187_column_slot_alignment | `passed` | 17 |
| spa_188_two_hop_where | `passed` | 32 |
| spa_189_checkpoint_optimize | `failed` | 4 |
| spa_189_checkpoint_optimize | `passed` | 6 |
| spa_192_match_no_label | `passed` | 17 |
| spa_193_undirected_pattern | `passed` | 12 |
| spa_194_count_node_var | `passed` | 12 |
| spa_195_type_function | `failed` | 2 |
| spa_195_type_function | `passed` | 12 |
| spa_196_id_function | `passed` | 14 |
| spa_197_count_label_fastpath | `passed` | 16 |
| spa_197_missing_prop_null | `passed` | 7 |
| spa_198_limit_pushdown | `passed` | 8 |
| spa_198_unlabeled_rel_endpoint | `failed` | 6 |
| spa_198_unlabeled_rel_endpoint | `passed` | 2 |
| spa_199_bfs_early_exit | `passed` | 6 |
| spa_200_batch_hop_perf | `failed` | 1 |
| spa_200_batch_hop_perf | `passed` | 16 |
| spa_201_csr_backward | `passed` | 32 |
| spa_206_contains_predicate | `passed` | 16 |
| spa_206_mlm_benchmark | `passed` | 1 |
| spa_207_labels_function | `passed` | 13 |
| spa_207_null_sentinel | `passed` | 16 |
| spa_208_reserved_labels | `passed` | 10 |
| spa_208_string_heap | `passed` | 13 |
| spa_209_schema_introspection | `failed` | 6 |
| spa_209_schema_introspection | `passed` | 13 |
| spa_211_unlabeled_match_create | `passed` | 14 |
| spa_212_string_truncation | `passed` | 18 |
| spa_213_return_node_var | `passed` | 9 |
| spa_214_skip_clause | `passed` | 6 |
| spa_215_merge_return | `passed` | 7 |
| spa_216_delete_node | `passed` | 20 |
| spa_217_info_counts | `passed` | 13 |
| spa_222_csr_lazy_load | `passed` | 1 |
| spa_224_regression_no_so_label | `passed` | 4 |
| spa_224_varpath_reserved_label | `passed` | 9 |
| spa_229_add_property | `passed` | 13 |
| spa_229_edge_prop_float | `passed` | 15 |
| spa_233_merge_relationship | `passed` | 9 |
| spa_235_234_create_index_constraint | `failed` | 7 |
| spa_235_234_create_index_constraint | `passed` | 14 |
| spa_236_labels_predicate | `passed` | 19 |
| spa_237_unwind_match | `passed` | 19 |
| spa_240_coalesce | `passed` | 11 |
| spa_241_multihop_props | `passed` | 15 |
| spa_242_count_rel_var | `passed` | 16 |
| spa_243_create_entity | `failed` | 1 |
| spa_243_create_entity | `passed` | 9 |
| spa_244_mcp_errors | `failed` | 2 |
| spa_244_mcp_errors | `passed` | 3 |
| spa_245_unknown_label_returns_empty | `passed` | 10 |
| spa_249_property_index | `passed` | 37 |
| spa_250_batch_write | `passed` | 2 |
| spa_251_text_search_index | `passed` | 30 |
| spa_252_three_hop_binding | `passed` | 15 |
| spa_254_query_timeout | `passed` | 2 |
| spa_259_inline_prop_filter | `passed` | 10 |
| spa_261_edge_props_perf | `failed` | 5 |
| spa_261_edge_props_perf | `passed` | 6 |
| spa_263_two_hop_agg | `passed` | 23 |
| spa_263_two_hop_null | `passed` | 25 |
| spa_264_boolean_props | `passed` | 14 |
| spa_265_backtick_escaping | `failed` | 5 |
| spa_265_backtick_escaping | `passed` | 22 |
| spa_266_265_bugs | `failed` | 1 |
| spa_266_265_bugs | `passed` | 5 |
| spa_267_float_codec | `passed` | 17 |
| spa_268_bfs_bugs | `passed` | 21 |
| spa_272_degree_cache | `passed` | 11 |
| spa_272_q7_count_fastpath | `failed` | 5 |
| spa_272_q7_count_fastpath | `passed` | 19 |
| spa_272_q7_cypher_wiring | `failed` | 4 |
| spa_272_q7_cypher_wiring | `passed` | 20 |
| spa_273_planner_stats | `passed` | 22 |
| spa_289_multi_label | `passed` | 28 |
| spa_296_bulk_loader | `passed` | 1 |
| spa_299_chunked_pipeline | `passed` | 31 |
| spa_299_phase2_parity | `passed` | 35 |
| spa_299_phase3_parity | `passed` | 66 |
| spa_299_phase4_parity | `passed` | 66 |
| spa_306_constraint_persistence | `failed` | 3 |
| spa_306_constraint_persistence | `passed` | 12 |
| spa_354_varlength_terminal_label | `passed` | 24 |
| spa_98_wal_encryption | `failed` | 1 |
| spa_98_wal_encryption | `passed` | 1 |
| spa_aggregation | `passed` | 26 |
| spa_collect_agg | `passed` | 16 |
| spa_datetime_fns | `failed` | 1 |
| spa_datetime_fns | `passed` | 6 |
| spa_fulltext | `failed` | 7 |
| spa_in_operator | `passed` | 16 |
| spa_is_null | `passed` | 15 |
| spa_list_predicates | `failed` | 10 |
| spa_list_predicates | `passed` | 28 |
| spa_type_labels | `failed` | 1 |
| spa_type_labels | `passed` | 17 |
| spa_variable_paths | `passed` | 37 |
| test_pole | `passed` | 8 |
| test_reactome | `passed` | 5 |
| uc1_social_graph | `passed` | 2 |
| uc7_unwind | `failed` | 2 |
| uc7_unwind | `passed` | 5 |
| uc_tracing | `passed` | 1 |
| vector_index | `failed` | 6 |
| vector_index | `passed` | 7 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 72 |
| `execution` | `passed` | 2099 |
| `parser` | `failed` | 82 |

### Failures (154)

- `sparrowdb.acceptance.check-4-checkpoint-optimize-no-error.query-3`: expected clause at byte 0..0
- `sparrowdb.acceptance.check-4-checkpoint-optimize-no-error.query-4`: expected clause at byte 0..0
- `sparrowdb.acceptance.check-14-fulltext-search.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..74
- `sparrowdb.acceptance.check-14-fulltext-search.query-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..72
- `sparrowdb.acceptance.check-14-fulltext-search.query-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..72
- `sparrowdb.call-subquery.unit-subquery-returns-all-rows.query-3`: expected identifier at byte 5..5
- `sparrowdb.call-subquery.unit-subquery-empty-when-no-match.query-1`: expected identifier at byte 5..5
- `sparrowdb.call-subquery.unit-subquery-with-limit.query-4`: expected identifier at byte 5..5
- `sparrowdb.call-subquery.correlated-subquery-counts-friends.query-4`: expected identifier at byte 22..22
- `sparrowdb.call-subquery.correlated-subquery-collects-friend-names.query-6`: expected identifier at byte 38..38
- `sparrowdb.delete-edge.cypher-match-delete-rel-removes-edge.query-1`: expected not_expression at byte 23..23
- `sparrowdb.fts-index.test-auto-index-on-create.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 76..92
- `sparrowdb.fts-index.test-full-text-search-predicate.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 68..81
- `sparrowdb.fts-index.test-full-text-search-predicate.query-2`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 70..83
- `sparrowdb.fts-index.test-full-text-search-predicate.query-3`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 75..88
- `sparrowdb.fts-index.test-bm25-score-order-by.query-1`: query execution failed: Parse error: unknown variable `score` at byte 169..175; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 86..188
- `sparrowdb.fts-index.test-multiword-query-union-scoring.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 66..79
- `sparrowdb.fts-index.test-fts-index-survives-restart.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 68..81
- `sparrowdb.fts-index.test-bm25-ranking-50-nodes.query-1`: query execution failed: Parse error: unknown variable `score` at byte 164..170; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..183
- `sparrowdb.hybrid-search.hybrid-search-20-nodes-rrf.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..98
- `sparrowdb.hybrid-search.hybrid-search-weighted-fusion-alpha.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..89
- `sparrowdb.hybrid-search.hybrid-search-weighted-fusion-alpha.query-2`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..89
- `sparrowdb.hybrid-search.hybrid-search-missing-fts-falls-back.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..76
- `sparrowdb.hybrid-search.hybrid-search-k-zero-returns-null.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..63
- `sparrowdb.match-after-create.match-finds-all-nodes-after-wal-only-creates.query-1`: expected clause at byte 0..0
- `sparrowdb.mcp-cypher-templates.mcp-template-merge-node-count-query-integer-parses-and-executes.query-1`: expected identifier at byte 9..9
- `sparrowdb.mcp-cypher-templates.mcp-template-merge-node-count-query-integer-parses-and-executes.query-2`: expected identifier at byte 9..9
- `sparrowdb.mcp-cypher-templates.mcp-template-add-property-escaped-match-val-parses-and-executes.query-1`: expected not_expression at byte 23..23
- `sparrowdb.readtx-query.readtx-query-rejects-checkpoint.query-1`: expected clause at byte 0..0
- `sparrowdb.regression-366.create-with-return-scalar-props.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-366.create-with-return-id.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-366.create-with-return-whole-node.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-366.create-multi-pattern-with-return.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-368.create-two-nodes-in-one-statement.query-1`: expected not_expression at byte 23..23
- `sparrowdb.regression-368.create-inline-path-with-edge.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-368.create-inline-path-without-edge-props.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-368.create-three-nodes-one-statement.query-1`: expected not_expression at byte 23..23
- `sparrowdb.regression-369.setup-graph.query-1`: expected not_expression at byte 24..24
- `sparrowdb.regression-369.one-hop-unlabeled-return-aliases.query-1`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..70
- `sparrowdb.regression-379.detach-delete-after-checkpoint.query-4`: expected clause at byte 0..0
- `sparrowdb.regression-380.on-create-set-fires-when-node-is-new.query-1`: expected EOI, UNION, clause, or relationship_pattern at byte 33..33
- `sparrowdb.regression-380.on-match-set-fires-when-node-exists.query-2`: expected EOI, UNION, clause, or relationship_pattern at byte 33..33
- `sparrowdb.regression-380.on-create-set-does-not-fire-on-existing-node.query-2`: expected EOI, UNION, clause, or relationship_pattern at byte 31..31
- `sparrowdb.regression-380.on-match-set-does-not-fire-on-new-node.query-1`: expected EOI, UNION, clause, or relationship_pattern at byte 33..33
- `sparrowdb.regression-380.both-on-clauses-first-call-fires-create.query-1`: expected EOI, UNION, clause, or relationship_pattern at byte 32..32
- `sparrowdb.regression-380.both-on-clauses-second-call-fires-match.query-1`: expected EOI, UNION, clause, or relationship_pattern at byte 31..31
- `sparrowdb.regression-380.both-on-clauses-second-call-fires-match.query-2`: expected EOI, UNION, clause, or relationship_pattern at byte 31..31
- `sparrowdb.spa-136-shortest-path.shortest-path-direct.query-4`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-136-shortest-path.shortest-path-2-hops.query-6`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-136-shortest-path.shortest-path-no-path.query-3`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-139-phase9-path-acceptance.shortest-path-prefers-minimum-hops.query-7`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..87
- `sparrowdb.spa-139-phase9-path-acceptance.shortest-path-null-when-disconnected.query-3`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..93
- `sparrowdb.spa-140-143-functions.spa143-isnull-true.query-1`: query execution failed: Parse error: generated relational SQL did not parse: near "isNull": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `sparrowdb.spa-140-143-functions.spa143-isnotnull-true.query-1`: query execution failed: Parse error: no such function: isNotNull; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..25
- `sparrowdb.spa-140-143-functions.spa143-id-function-in-match-return.query-1`: query execution failed: Parse error: generated relational SQL did not parse: near "isNull": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `sparrowdb.spa-151-kms-query-validation.kms-q18-fulltext-search-call-procedure.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..80
- `sparrowdb.spa-151-kms-query-validation.kms-q18b-fulltext-search-yield-node-only.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..74
- `sparrowdb.spa-151-kms-query-validation.kms-q19-fulltext-search-no-results.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..80
- `sparrowdb.spa-151-kms-query-validation.kms-q31-variable-length-path-traversal.query-1`: query execution failed: Parse error: unknown variable `distance` at byte 174..183; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 121..191
- `sparrowdb.spa-151-kms-query-validation.kms-q32-merge-relationship-pattern.query-3`: expected EOI, UNION, clause, or relationship_pattern at byte 79..79
- `sparrowdb.spa-151-kms-query-validation.kms-q33-create-unique-constraint.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-151-kms-query-validation.kms-q34-create-property-index.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-178-edge-properties.test-edge-prop-basic-write-read.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-filter-match.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-filter-no-match.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-string-survives-checkpoint.query-1`: expected not_expression at byte 30..30
- `sparrowdb.spa-178-edge-properties.test-edge-prop-where-filter-match.query-1`: expected not_expression at byte 21..21
- `sparrowdb.spa-178-edge-properties.test-edge-prop-where-filter-match.query-2`: expected not_expression at byte 21..21
- `sparrowdb.spa-178-edge-properties.test-no-edge-var-no-read.query-1`: expected not_expression at byte 21..21
- `sparrowdb.spa-178-edge-properties.test-edge-prop-float-where-filter.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-float-where-filter.query-2`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-float-where-filter.query-3`: expected not_expression at byte 23..23
- `sparrowdb.spa-178-edge-properties.test-edge-prop-float-where-filter.query-4`: expected not_expression at byte 22..22
- `sparrowdb.spa-178-edge-properties.test-edge-prop-float-where-filter.query-5`: expected not_expression at byte 22..22
- `sparrowdb.spa-189-checkpoint-optimize.checkpoint-command-runs-without-error.query-1`: expected clause at byte 0..0
- `sparrowdb.spa-189-checkpoint-optimize.checkpoint-command-runs-after-writes.query-3`: expected clause at byte 0..0
- `sparrowdb.spa-189-checkpoint-optimize.optimize-command-runs-without-error.query-1`: expected clause at byte 0..0
- `sparrowdb.spa-189-checkpoint-optimize.optimize-command-runs-after-writes.query-3`: expected clause at byte 0..0
- `sparrowdb.spa-195-type-function.type-r-unlabeled-pattern-returns-type-name.query-4`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..33
- `sparrowdb.spa-195-type-function.type-r-multiple-rel-types-returns-correct-names.query-6`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..33
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-unlabeled-dst-matches-any-node.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-unlabeled-dst-matches-any-node.query-2`: expected not_expression at byte 29..29
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-unlabeled-dst-matches-any-node.query-3`: expected not_expression at byte 22..22
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-labeled-src-and-dst-still-works.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-labeled-src-and-dst-still-works.query-2`: expected not_expression at byte 29..29
- `sparrowdb.spa-198-unlabeled-rel-endpoint.spa198-labeled-src-and-dst-still-works.query-3`: expected not_expression at byte 22..22
- `sparrowdb.spa-200-batch-hop-perf.two-hop-returns-valid-names.query-2`: expected clause at byte 0..0
- `sparrowdb.spa-209-schema-introspection.schema-result-has-named-columns.query-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-contains-node-labels-and-properties.query-4`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-contains-relationship-types.query-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-empty-db-returns-no-rows.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-label-with-no-properties.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.query-result-row-as-map.query-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-235-234-create-index-constraint.create-index-supports-equality-lookup.query-3`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.create-index-on-missing-label-is-noop.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.unique-constraint-allows-first-insert.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.unique-constraint-rejects-duplicate.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.unique-constraint-is-label-scoped.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.unique-constraint-rejects-duplicate-within-same-statement.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-235-234-create-index-constraint.unique-constraint-allows-different-values.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-243-create-entity.spa243-empty-class-name-returns-descriptive-error.query-1`: expected identifier at byte 11..11
- `sparrowdb.spa-244-mcp-errors.spa244-empty-query-returns-meaningful-error.query-1`: expected clause at byte 0..0
- `sparrowdb.spa-244-mcp-errors.spa244-syntax-error-returns-meaningful-error.query-1`: expected clause at byte 0..0
- `sparrowdb.spa-261-edge-props-perf.test-hop-without-edge-props.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-261-edge-props-perf.test-hop-with-edge-props.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-261-edge-props-perf.test-edge-props-cache-hit.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-261-edge-props-perf.test-edge-props-cache-invalidation.query-1`: expected not_expression at byte 23..23
- `sparrowdb.spa-261-edge-props-perf.test-edge-props-cache-invalidation.query-3`: expected not_expression at byte 23..23
- `sparrowdb.spa-265-backtick-escaping.bare-keyword-label-order.query-1`: expected identifier at byte 10..10
- `sparrowdb.spa-265-backtick-escaping.bare-keyword-label-order.query-2`: expected identifier at byte 9..9
- `sparrowdb.spa-265-backtick-escaping.unterminated-backtick-is-error.query-1`: expected identifier at byte 10..10
- `sparrowdb.spa-265-backtick-escaping.backtick-and-bare-keyword-same-case-are-interchangeable.query-1`: expected identifier at byte 10..10
- `sparrowdb.spa-265-backtick-escaping.backtick-and-bare-keyword-same-case-are-interchangeable.query-4`: expected identifier at byte 9..9
- `sparrowdb.spa-266-265-bugs.match-nonexistent-label-with-props-filter-returns-empty.query-1`: expected not_expression at byte 22..22
- `sparrowdb.spa-272-q7-count-fastpath.q7-count-f-order-by-alias-desc-limit.query-11`: query execution failed: Parse error: unknown variable `deg` at byte 78..82; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 38..94
- `sparrowdb.spa-272-q7-count-fastpath.q7-exact-facebook-query-shape.query-4`: query execution failed: Parse error: unknown variable `deg` at byte 75..79; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..92
- `sparrowdb.spa-272-q7-count-fastpath.q7-count-fastpath-unknown-label-returns-empty.query-1`: query execution failed: Parse error: unknown variable `deg` at byte 84..88; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..101
- `sparrowdb.spa-272-q7-count-fastpath.q7-count-with-where-falls-through-to-normal-path.query-4`: query execution failed: Parse error: unknown variable `deg` at byte 97..101; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 57..114
- `sparrowdb.spa-272-q7-count-fastpath.q7-count-column-names-correct.query-4`: query execution failed: Parse error: unknown variable `deg` at byte 78..82; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 38..94
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-out-degree-returns-top-k.query-11`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..73
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-alias-returns-top-k.query-7`: query execution failed: Parse error: no such function: degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..65
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-unknown-label-returns-empty.query-1`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..79
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-desc-ordering-correct.query-2`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 15..71
- `sparrowdb.spa-306-constraint-persistence.unique-constraint-persists-across-reopen.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-306-constraint-persistence.multiple-constraints-persist.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.spa-306-constraint-persistence.multiple-constraints-persist.query-2`: expected node_pattern at byte 7..7
- `sparrowdb.spa-98-wal-encryption.spa-98-wrong-key-fails.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-datetime-fns.timestamp-alias.query-1`: query execution failed: Parse error: no such function: timestamp; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `sparrowdb.spa-fulltext.create-index-and-search.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.search-partial-match.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..77
- `sparrowdb.spa-fulltext.search-partial-match.query-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..66
- `sparrowdb.spa-fulltext.call-yield-node-usable-in-return.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.call-yield-node-usable-in-return.query-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.unknown-procedure-returns-error.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `sparrowdb.spa-fulltext.call-missing-index-returns-empty.query-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..73
- `sparrowdb.spa-list-predicates.any-matches.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 14..77
- `sparrowdb.spa-list-predicates.any-no-match.query-3`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 14..75
- `sparrowdb.spa-list-predicates.all-matches.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..78
- `sparrowdb.spa-list-predicates.all-fails.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..78
- `sparrowdb.spa-list-predicates.none-matches.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 15..84
- `sparrowdb.spa-list-predicates.none-fails.query-3`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 15..84
- `sparrowdb.spa-list-predicates.any-on-empty-list.query-2`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 25..77
- `sparrowdb.spa-list-predicates.all-on-empty-list.query-2`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 25..81
- `sparrowdb.spa-list-predicates.single-matches.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..83
- `sparrowdb.spa-list-predicates.single-fails-multiple.query-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..83
- `sparrowdb.spa-type-labels.type-fn-variable-path.query-6`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..58
- `sparrowdb.uc7-unwind.unwind-param-returns-empty-without-binding.query-1`: query execution failed: Parse error: unknown parameter `$items` at byte 7..14; mutation execution failed: Cypher mutation binding failed: unknown parameter `$items` at byte 7..14
- `sparrowdb.uc7-unwind.unwind-return-wrong-variable-yields-null.query-1`: query execution failed: Parse error: unknown variable `y` at byte 29..30; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..30
- `sparrowdb.vector-index.create-vector-index-ddl.query-1`: expected node_pattern at byte 7..7
- `sparrowdb.vector-index.create-vector-index-ddl.query-2`: expected node_pattern at byte 7..7
- `sparrowdb.vector-index.vector-similarity-function.query-3`: query execution failed: Parse error: no such function: vector_similarity; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..65
- `sparrowdb.vector-index.vector-similarity-orthogonal-is-zero.query-1`: query execution failed: Parse error: no such function: vector_similarity; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..55
- `sparrowdb.vector-index.vector-distance-function.query-1`: query execution failed: Parse error: no such function: vector_distance; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..51
- `sparrowdb.vector-index.vector-dot-function.query-1`: query execution failed: Parse error: no such function: vector_dot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47

## Latest `tck-deep` run

- Run: `20260719T211010.696297Z-aeb0c662831c-corpus-deep`
- Commit: `aeb0c662831c736bacd67f988d1a6a878f60a196` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 3926
- Passed: 2793
- Unsupported: 0
- Failed or changed: 1133

### Outcome changes from `20260719T210524.711499Z-51b9a1bd28bf-corpus-deep`

- `tck.clauses.create.create1.scenario-9`: Passed
- `tck.clauses.create.create1.scenario-10`: Passed
- `tck.clauses.create.create1.scenario-11`: Passed
- `tck.clauses.create.create1.scenario-12`: Passed
- `tck.clauses.create.create2.scenario-15`: Passed
- `tck.clauses.create.create2.scenario-16`: Passed
- `tck.clauses.create.create2.scenario-17`: Passed
- `tck.clauses.create.create3.scenario-13`: Passed
- `tck.clauses.match-where.matchwhere2.scenario-1`: Passed
- `tck.clauses.match-where.matchwhere3.scenario-2`: Passed
- `tck.clauses.merge.merge9.scenario-1`: Passed
- `tck.clauses.return.return3.scenario-1`: Passed
- `tck.clauses.return.return6.scenario-2`: Passed
- `tck.clauses.return-orderby.returnorderby2.scenario-8`: Passed
- `tck.clauses.return-orderby.returnorderby2.scenario-10`: Passed
- `tck.clauses.with.with2.scenario-1`: Passed
- `tck.clauses.with-skip-limit.withskiplimit2.scenario-2`: Passed
- `tck.clauses.with-where.withwhere2.scenario-1`: Passed
- `tck.clauses.with-where.withwhere3.scenario-2`: Passed
- `tck.expressions.comparison.comparison1.scenario-1`: Passed

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| clauses | `failed` | 391 |
| clauses | `passed` | 860 |
| expressions | `failed` | 721 |
| expressions | `passed` | 1924 |
| useCases | `failed` | 21 |
| useCases | `passed` | 9 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 938 |
| `execution` | `passed` | 2750 |
| `fixture-execution` | `failed` | 19 |
| `parameter-binding` | `failed` | 20 |
| `parser` | `failed` | 132 |
| `parser` | `passed` | 43 |
| `setup-execution` | `failed` | 4 |
| `side-effect-comparison` | `failed` | 20 |

### Failures (1133)

- `tck.clauses.call.call1.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..20; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.doNothing()
- `tck.clauses.call.call1.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..20; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..21; query:
CALL test.doNothing
- `tck.clauses.call.call1.scenario-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 16..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 11..33; query:
MATCH (n)
CALL test.doNothing()
RETURN n
- `tck.clauses.call.call1.scenario-4`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 16..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 11..33; query:
MATCH (n)
CALL test.doNothing()
RETURN n.name AS `name`
- `tck.clauses.call.call1.scenario-5`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..20; query:
CALL test.labels()
- `tck.clauses.call.call1.scenario-6`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.labels() YIELD label
RETURN label
- `tck.clauses.call.call2.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..56; query:
CALL test.my.proc('Stefan', 1) YIELD city, country_code
RETURN city, country_code
- `tck.clauses.call.call2.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.my.proc('Stefan', 1)
- `tck.clauses.call.call2.scenario-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..19; query:
CALL test.my.proc
- `tck.clauses.call.call3.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.my.proc(42)
- `tck.clauses.call.call3.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..33; query:
CALL test.my.proc(42) YIELD out
RETURN out
- `tck.clauses.call.call3.scenario-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..25; query:
CALL test.my.proc(42.3)
- `tck.clauses.call.call3.scenario-4`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(42.3) YIELD out
RETURN out
- `tck.clauses.call.call3.scenario-5`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.my.proc(42)
- `tck.clauses.call.call3.scenario-6`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..33; query:
CALL test.my.proc(42) YIELD out
RETURN out
- `tck.clauses.call.call4.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..25; query:
CALL test.my.proc(null)
- `tck.clauses.call.call4.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN out
- `tck.clauses.call.call5.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN out
- `tck.clauses.call.call5.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN *
- `tck.clauses.call.call5.scenario-3.examples-1-row-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD a, b
RETURN a, b
- `tck.clauses.call.call5.scenario-3.examples-1-row-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD b, a
RETURN a, b
- `tck.clauses.call.call5.scenario-4.examples-1-row-1`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-2`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-3`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-4`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-5`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-6`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-7`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-8`: expected EOI, UNION, or clause at byte 35..35
- `tck.clauses.call.call5.scenario-4.examples-1-row-9`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-10`: expected EOI, UNION, or clause at byte 32..32
- `tck.clauses.call.call5.scenario-4.examples-1-row-11`: expected EOI, UNION, or clause at byte 35..35
- `tck.clauses.call.call5.scenario-8`: expected identifier at byte 37..37
- `tck.clauses.call.call6.scenario-1`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.labels() YIELD label
WITH count(*) AS c
CALL test.labels() YIELD label
RETURN *
- `tck.clauses.call.call6.scenario-2`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
WITH out RETURN out
- `tck.clauses.call.call6.scenario-3`: query execution failed: Parse error: procedures outside the built-in registry is not supported in the initial graph slice at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
WITH out AS a RETURN a
- `tck.clauses.create.create3.scenario-3`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..20; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 27..36; query:
MATCH ()
CREATE ()
WITH *
MATCH ()
CREATE ()
- `tck.clauses.create.create3.scenario-5`: expected [["( {num: 1})", "( {num: 1})"]], observed [["1", "1"]]
- `tck.clauses.create.create3.scenario-6`: expected [["(:X)"]], observed [["1"]]
- `tck.clauses.create.create3.scenario-7`: expected [["( {name: 'A'})", "( {name: 'A'})"]], observed [["1", "1"]]
- `tck.clauses.create.create3.scenario-8`: expected [["( {num: 5})"]], observed [["1"]]
- `tck.clauses.create.create3.scenario-11`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..12; mutation execution failed: MERGE requires at least one property to identify the entity; query:
CREATE (a)
WITH a
MERGE ()
CREATE (b)
CREATE (a)<-[:T]-(b)
- `tck.clauses.create.create3.scenario-12`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..12; mutation execution failed: MERGE requires at least one property to identify the entity; query:
CREATE (a)
WITH a
MERGE (x)
MERGE (y)
MERGE (x)-[:T]->(y)
CREATE (b)
CREATE (a)<-[:T]-(b)
- `tck.clauses.create.create4.scenario-1`: expected not_expression at byte 22059..22059
- `tck.clauses.create.create5.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 19..19
- `tck.clauses.create.create5.scenario-5`: expected EOI, UNION, clause, or relationship_pattern at byte 11..11
- `tck.clauses.delete.delete1.scenario-5`: expected [["<null>"]], observed []
- `tck.clauses.delete.delete2.scenario-3`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 39..48; mutation execution failed: graph mutation database operation failed: Parse error: no such column: b1; query:
MATCH p = ()-[r:T]-()
WHERE r.id = 42
DELETE r
- `tck.clauses.delete.delete2.scenario-4`: expected [["<null>"]], observed []
- `tck.clauses.delete.delete3.scenario-1`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 31..47; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 45..46; query:
MATCH p = (:X)-->()-->()-->()
DETACH DELETE p
- `tck.clauses.delete.delete3.scenario-2`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 28..44; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 42..43; query:
OPTIONAL MATCH p = ()-->()
DETACH DELETE p
- `tck.clauses.delete.delete5.scenario-1`: expected EOI, UNION, or clause at byte 77..77
- `tck.clauses.delete.delete5.scenario-2`: expected EOI, UNION, or clause at byte 85..85
- `tck.clauses.delete.delete5.scenario-3`: expected EOI, UNION, or clause at byte 50..50
- `tck.clauses.delete.delete5.scenario-4`: expected EOI, UNION, or clause at byte 60..60
- `tck.clauses.delete.delete5.scenario-5`: expected EOI, UNION, or clause at byte 70..70
- `tck.clauses.delete.delete5.scenario-6`: expected EOI, UNION, or clause at byte 76..76
- `tck.clauses.delete.delete5.scenario-7`: expected EOI, UNION, or clause at byte 83..83
- `tck.clauses.match.match2.scenario-2`: expected [["[:T1]"]], observed [["[:T1]"], ["[:T4]"]]
- `tck.clauses.match.match2.scenario-3`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..37; query:
MATCH ()-[r]-()
RETURN type(r) AS r
- `tck.clauses.match.match2.scenario-4`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..38; query:
MATCH ()-[r]->()
RETURN type(r) AS r
- `tck.clauses.match.match2.scenario-7`: query execution failed: Parse error: duplicate variable `r` at byte 45..46; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 45..46; query:
MATCH (a1)-[r:T]->()
WITH r, a1
MATCH (a1)-[r:Y]->(b2)
RETURN a1, r, b2
- `tck.clauses.match.match3.scenario-6`: expected [["(:Foo)"]], observed [["()"], ["(:Foo)"]]
- `tck.clauses.match.match3.scenario-7`: expected [["(:A:B:C:D:E:F:G:H:I:J:K:L:M)", "(:Z:Y:X:W:V:U)"]], observed [["(:A:B:C:D:E:F:G:H:I:J:K:L:M)", "(:U:V:W:X:Y:Z)"]]
- `tck.clauses.match.match3.scenario-15`: expected [["(:A)", "[:T1]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:A)", "[:T1]", "(:Looper)", "[:T2]", "(:B)"]], observed [["(:A)", "[:T1]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:A)", "[:T1]", "(:Looper)", "[:T1]", "(:A)"], ["(:A)", "[:T1]", "(:Looper)", "[:T2]", "(:B)"]]
- `tck.clauses.match.match3.scenario-16`: expected [["(:A)", "[:T1]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:A)", "[:T1]", "(:Looper)", "[:T2]", "(:B)"], ["(:B)", "[:T2]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:B)", "[:T2]", "(:Looper)", "[:T1]", "(:A)"], ["(:Looper)", "[:LOOP]", "(:Looper)", "[:T1]", "(:A)"], ["(:Looper)", "[:LOOP]", "(:Looper)", "[:T2]", "(:B)"]], observed [["(:A)", "[:T1]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:A)", "[:T1]", "(:Looper)", "[:T1]", "(:A)"], ["(:A)", "[:T1]", "(:Looper)", "[:T2]", "(:B)"], ["(:B)", "[:T2]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:B)", "[:T2]", "(:Looper)", "[:T1]", "(:A)"], ["(:B)", "[:T2]", "(:Looper)", "[:T2]", "(:B)"], ["(:Looper)", "[:LOOP]", "(:Looper)", "[:LOOP]", "(:Looper)"], ["(:Looper)", "[:LOOP]", "(:Looper)", "[:T1]", "(:A)"], ["(:Looper)", "[:LOOP]", "(:Looper)", "[:T2]", "(:B)"], ["(:Looper)", "[:T1]", "(:A)", "[:T1]", "(:Looper)"], ["(:Looper)", "[:T2]", "(:B)", "[:T2]", "(:Looper)"]]
- `tck.clauses.match.match3.scenario-19`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 54..54
- `tck.clauses.match.match3.scenario-24`: query execution failed: Parse error: duplicate variable `r` at byte 45..46; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 45..46; query:
MATCH (a1)-[r:T]->()
WITH r, a1
MATCH (a1)-[r:T]->(b2)
RETURN a1, r, b2
- `tck.clauses.match.match3.scenario-25`: query execution failed: Parse error: duplicate variable `r` at byte 45..46; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 45..46; query:
MATCH (a1)-[r]->()
WITH r, a1
MATCH (a1:X)-[r]->(b2)
RETURN a1, r, b2
- `tck.clauses.match.match3.scenario-26`: query execution failed: Parse error: duplicate variable `r` at byte 49..50; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 49..50; query:
MATCH (a1:X:Y)-[r]->()
WITH r, a1
MATCH (a1:Y)-[r]->(b2)
RETURN a1, r, b2
- `tck.clauses.match.match4.scenario-1`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 25..34; query:
MATCH (a)-[r*1..1]->(b)
RETURN r
- `tck.clauses.match.match4.scenario-4`: TCK setup query failed: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..45; mutation execution failed: graph mutation database operation failed: Parse error: no such function: collect; query:
CREATE (a {var: 'start'}), (b {var: 'end'})
WITH *
UNWIND range(1, 20) AS i
CREATE (n {var: i})
WITH a, b, [a] + collect(n) + [b] AS nodeList
UNWIND range(0, size(nodeList) - 2, 1) AS i
WITH nodeList[i] AS n1, nodeList[i+1] AS n2
CREATE (n1)-[:T]->(n2)
; query:
CREATE (a {var: 'start'}), (b {var: 'end'})
WITH *
UNWIND range(1, 20) AS i
CREATE (n {var: i})
WITH a, b, [a] + collect(n) + [b] AS nodeList
UNWIND range(0, size(nodeList) - 2, 1) AS i
WITH nodeList[i] AS n1, nodeList[i+1] AS n2
CREATE (n1)-[:T]->(n2)
- `tck.clauses.match.match4.scenario-5`: relationship range is outside the supported u32 range at byte 30..32
- `tck.clauses.match.match4.scenario-6`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..42; query:
MATCH (a:A)
MATCH (a)-[r*2]->()
RETURN r
- `tck.clauses.match.match4.scenario-7`: query execution failed: Parse error: duplicate variable `r` at byte 48..49; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 48..49; query:
MATCH ()-[r:EDGE]-()
MATCH p = (n)-[*0..1]-()-[r]-()-[*0..1]-(m)
RETURN count(p) AS c
- `tck.clauses.match.match4.scenario-8`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: rs; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: rs; query:
MATCH ()-[r1]->()-[r2]->()
WITH [r1, r2] AS rs
  LIMIT 1
MATCH (first)-[rs*]->(second)
RETURN first, second
- `tck.clauses.match.match5.scenario-11`: query execution failed: Parse error: invalid relationship range 2..1 at byte 30..35; mutation execution failed: Cypher mutation binding failed: invalid relationship range 2..1 at byte 30..35; query:
MATCH (a:A)
MATCH (a)-[:LIKES*2..1]->(c)
RETURN c.name
- `tck.clauses.match.match5.scenario-12`: query execution failed: Parse error: invalid relationship range 1..0 at byte 30..35; mutation execution failed: Cypher mutation binding failed: invalid relationship range 1..0 at byte 30..35; query:
MATCH (a:A)
MATCH (a)-[:LIKES*1..0]->(c)
RETURN c.name
- `tck.clauses.match.match5.scenario-13`: query execution failed: Parse error: invalid relationship range 1..0 at byte 30..34; mutation execution failed: Cypher mutation binding failed: invalid relationship range 1..0 at byte 30..34; query:
MATCH (a:A)
MATCH (a)-[:LIKES*..0]->(c)
RETURN c.name
- `tck.clauses.match.match5.scenario-25`: expected [["n00000"], ["n00001"], ["n00010"], ["n00011"], ["n00100"], ["n00101"], ["n00110"], ["n00111"], ["n01000"], ["n01001"], ["n01010"], ["n01011"], ["n01100"], ["n01101"], ["n01110"], ["n01111"]], observed [["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"]]
- `tck.clauses.match.match5.scenario-26`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.match.match5.scenario-27`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 34..34
- `tck.clauses.match.match5.scenario-28`: expected [["n00000"], ["n00001"], ["n00010"], ["n00011"], ["n00100"], ["n00101"], ["n00110"], ["n00111"], ["n01000"], ["n01001"], ["n01010"], ["n01011"], ["n01100"], ["n01101"], ["n01110"], ["n01111"]], observed [["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"]]
- `tck.clauses.match.match5.scenario-29`: expected [["n00000"], ["n00001"], ["n00010"], ["n00011"], ["n00100"], ["n00101"], ["n00110"], ["n00111"], ["n01000"], ["n01001"], ["n01010"], ["n01011"], ["n01100"], ["n01101"], ["n01110"], ["n01111"]], observed [["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["0"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"], ["1"]]
- `tck.clauses.match.match6.scenario-1`: expected [[""]], observed [["<()>"]]
- `tck.clauses.match.match6.scenario-5`: expected [[""]], observed [["<(:Label1)<-[:TYPE]-(:Label2)>"]]
- `tck.clauses.match.match6.scenario-6`: expected [[""]], observed [["<(:B)<-[:T]-(:A)>"]]
- `tck.clauses.match.match6.scenario-8`: expected [], observed [["<()-[:T]->()<-[:T]-()>"], ["<()-[:T]->()<-[:T]-()>"]]
- `tck.clauses.match.match6.scenario-9`: TCK setup query failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 17..17; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 17..17; query:
CREATE (:Label1)(:Label3)
; query:
CREATE (:Label1)(:Label3)
- `tck.clauses.match.match6.scenario-10`: expected [["(:B)-[:T]->(:A)>"]], observed [["<(:B)-[:T]->(:A)<-[:T]-(:B)>"], ["<(:C)-[:T]->(:B)-[:T]->(:A)>"], ["<(:C)-[:T]->(:B)<-[:T]-(:C)>"]]
- `tck.clauses.match.match6.scenario-11`: expected [["(:C)-[:T]->(:B)-[:T]->(:A)>"]], observed [["<(:B)-[:T]->(:A)<-[:T]-(:B)-[:T]->(:A)>"], ["<(:B)-[:T]->(:A)<-[:T]-(:B)<-[:T]-(:C)>"], ["<(:C)-[:T]->(:B)-[:T]->(:A)<-[:T]-(:B)>"], ["<(:C)-[:T]->(:B)<-[:T]-(:C)-[:T]->(:B)>"], ["<(:C)-[:T]->(:B)<-[:T]-(:C)<-[:T]-(:D)>"], ["<(:D)-[:T]->(:C)-[:T]->(:B)-[:T]->(:A)>"], ["<(:D)-[:T]->(:C)-[:T]->(:B)<-[:T]-(:C)>"], ["<(:D)-[:T]->(:C)<-[:T]-(:D)-[:T]->(:C)>"]]
- `tck.clauses.match.match6.scenario-12`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 11..11
- `tck.clauses.match.match6.scenario-13`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 13..13
- `tck.clauses.match.match6.scenario-14`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 82..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 75..91; query:
MATCH topRoute = (:Start)<-[:CONNECTED_TO]-()-[:CONNECTED_TO*3..3]-(:End)
RETURN topRoute
- `tck.clauses.match.match6.scenario-15`: expected [[""]], observed [["<()>"]]
- `tck.clauses.match.match6.scenario-17`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 73..75; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 66..75; query:
MATCH p = (a {name: 'A'})-[:KNOWS*0..1]->(b)-[:FRIEND*0..1]->(c)
RETURN p
- `tck.clauses.match.match6.scenario-18`: expected [[""]], observed [["<(:Movie)<-[:T]-()>"]]
- `tck.clauses.match.match7.scenario-1`: expected [["<null>"]], observed []
- `tck.clauses.match.match7.scenario-3`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["(:B {num: 46})"], ["(:C)"], ["(:Single)"], ["(:Single)"]]
- `tck.clauses.match.match7.scenario-4`: query execution failed: Parse error: duplicate variable `r` at byte 63..64; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 63..64; query:
MATCH (a1)-[r]->()
WITH r, a1
  LIMIT 1
OPTIONAL MATCH (a1)<-[r]-(b2)
RETURN a1, r, b2
- `tck.clauses.match.match7.scenario-5`: query execution failed: Parse error: duplicate variable `r` at byte 56..57; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 56..57; query:
MATCH ()-[r]->()
WITH r
  LIMIT 1
OPTIONAL MATCH (a2)-[r]->(b2)
RETURN a2, r, b2
- `tck.clauses.match.match7.scenario-6`: query execution failed: Parse error: duplicate variable `r` at byte 62..63; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 62..63; query:
MATCH (a1)-[r]->()
WITH r, a1
  LIMIT 1
OPTIONAL MATCH (a1)-[r]->(b2)
RETURN a1, r, b2
- `tck.clauses.match.match7.scenario-8`: expected [["<null>"]], observed [["(:C)"]]
- `tck.clauses.match.match7.scenario-9`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["(:B {num: 46})"]]
- `tck.clauses.match.match7.scenario-10`: expected [["<null>"]], observed []
- `tck.clauses.match.match7.scenario-11`: expected [["(:A {num: 1})", "(:B {num: 2})", "(:C {num: 3})"], ["(:B {num: 2})", "(:A {num: 1})", "<null>"]], observed [["(:A {num: 1})", "(:B {num: 2})", "(:C {num: 3})"], ["(:A {num: 1})", "(:B {num: 2})", "<null>"], ["(:B {num: 2})", "(:A {num: 1})", "<null>"], ["(:B {num: 2})", "<null>", "<null>"], ["(:C {num: 3})", "<null>", "<null>"]]
- `tck.clauses.match.match7.scenario-12`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 33..45; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 33..45; query:
MATCH (a:Single)
OPTIONAL MATCH (a)-[*]->(b)
RETURN b
- `tck.clauses.match.match7.scenario-13`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 40..52; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 40..52; query:
MATCH (a:Single), (x:C)
OPTIONAL MATCH (a)-[*]->(x)
RETURN x
- `tck.clauses.match.match7.scenario-14`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 33..47; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 33..47; query:
MATCH (a:Single)
OPTIONAL MATCH (a)-[*3..]-(b)
RETURN b
- `tck.clauses.match.match7.scenario-15`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 61..79; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 61..79; query:
MATCH (a:A)
OPTIONAL MATCH (a)-[:FOO]->(b:B)
OPTIONAL MATCH (b)<-[:BAR*]-(c:B)
RETURN a, b, c
- `tck.clauses.match.match7.scenario-16`: expected [["<null>"]], observed [["{\"nodes\":[2,null],\"relationships\":[null]}"]]
- `tck.clauses.match.match7.scenario-17`: expected [["( {name: 'B'})", "<( {name: 'A'})-[:X]->( {name: 'B'})>"], ["( {name: 'C'})", "<null>"]], observed [["( {name: 'B'})", "<( {name: 'A'})-[:X]->( {name: 'B'})>"], ["( {name: 'C'})", "{\"nodes\":[1,3],\"relationships\":[null]}"]]
- `tck.clauses.match.match7.scenario-18`: expected [["<null>"]], observed [["{\"nodes\":[2,3],\"relationships\":[null]}"]]
- `tck.clauses.match.match7.scenario-19`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 38..60; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 38..60; query:
MATCH (a {name: 'A'})
OPTIONAL MATCH p = (a)-->(b)-[*]->(c)
RETURN p
- `tck.clauses.match.match7.scenario-20`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 35..51; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 35..51; query:
MATCH (a:A), (b:B)
OPTIONAL MATCH p = (a)-[*]->(b)
RETURN p
- `tck.clauses.match.match7.scenario-21`: expected [["<null>", "<null>", "<null>"]], observed []
- `tck.clauses.match.match7.scenario-22`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 124..125; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 124..125; query:
MATCH (a:Single)
OPTIONAL MATCH (a)-->(b:NonExistent)
OPTIONAL MATCH (a)-->(c:NonExistent)
WITH coalesce(b, c) AS x
MATCH (x)-->(d)
RETURN d
- `tck.clauses.match.match7.scenario-24`: expected [["[:LOOP]"]], observed [["<null>"], ["[:LOOP]"]]
- `tck.clauses.match.match7.scenario-25`: expected [["<null>"], ["<null>"], ["<null>"]], observed [["<null>"], ["<null>"], ["<null>"], ["<null>"]]
- `tck.clauses.match.match7.scenario-27`: expected [["<null>", "(:B {num: 46})", "<null>"]], observed []
- `tck.clauses.match.match7.scenario-28`: expected [["<null>"]], observed [["[:REL]"], ["[:REL]"]]
- `tck.clauses.match.match8.scenario-2`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 11..21; mutation execution failed: Cypher mutation binding failed: MATCH after a mutation clause is not supported in the initial graph slice at byte 28..52; query:
MATCH (a)
MERGE (b)
WITH *
OPTIONAL MATCH (a)--(b)
RETURN count(*)
- `tck.clauses.match.match8.scenario-3`: expected [["776"]], observed [["1216"]]
- `tck.clauses.match.match9.scenario-1`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..42; query:
MATCH ()-[r*0..1]-()
RETURN last(r) AS l
- `tck.clauses.match.match9.scenario-2`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..42; query:
MATCH (a)-[r:REL*2..2]->(b:End)
RETURN r
- `tck.clauses.match.match9.scenario-3`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..41; query:
MATCH (a)-[r:REL*2..2]-(b:End)
RETURN r
- `tck.clauses.match.match9.scenario-4`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..43; query:
MATCH (a:Start)-[r:REL*2..2]-(b)
RETURN r
- `tck.clauses.match.match9.scenario-5`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..48; query:
MATCH (a:Blue)-[r*]->(b:Green)
RETURN count(r)
- `tck.clauses.match.match9.scenario-6`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: rs; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: rs; query:
MATCH (a)-[r1]->()-[r2]->(b)
WITH [r1, r2] AS rs, a AS first, b AS second
  LIMIT 1
MATCH (first)-[rs*]->(second)
RETURN first, second
- `tck.clauses.match.match9.scenario-7`: query execution failed: Parse error: graph IR invariant failed: duplicate binding name: rs; mutation execution failed: Cypher mutation binding failed: graph IR invariant failed: duplicate binding name: rs; query:
MATCH (a)-[r1]->()-[r2]->(b)
WITH [r1, r2] AS rs, a AS second, b AS first
  LIMIT 1
MATCH (first)-[rs*]->(second)
RETURN first, second
- `tck.clauses.match.match9.scenario-8`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 35..47; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 35..47; query:
MATCH (a:A), (b:B)
OPTIONAL MATCH (a)-[r*]-(b)
WHERE r IS NULL
  AND a <> b
RETURN b
- `tck.clauses.match.match9.scenario-9`: query execution failed: Parse error: optional variable-length relationships is not supported in the initial graph slice at byte 70..87; mutation execution failed: Cypher mutation binding failed: optional variable-length relationships is not supported in the initial graph slice at byte 70..87; query:
MATCH (a {name: 'A'}), (x)
WHERE x.name IN ['B', 'C']
OPTIONAL MATCH p = (a)-[r*]->(x)
RETURN r, x, p
- `tck.clauses.match-where.matchwhere1.scenario-2`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 9..9
- `tck.clauses.match-where.matchwhere1.scenario-5`: TCK setup query failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 27..27; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 27..27; query:
CREATE ({name: 'Someone'})({name: 'Andres'})
; query:
CREATE ({name: 'Someone'})({name: 'Andres'})
- `tck.clauses.match-where.matchwhere1.scenario-6`: expected [["[:T{name:\"bar\"}]"]], observed [["[:T {name: 'bar'}]"]]
- `tck.clauses.match-where.matchwhere1.scenario-7`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 56..65; query:
MATCH (n {name: 'A'})-[r]->(x)
WHERE type(r) = 'KNOWS'
RETURN x
- `tck.clauses.match-where.matchwhere1.scenario-11`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 65..74; query:
MATCH (n)-[r]->(x)
WHERE type(r) = 'KNOWS' OR type(r) = 'HATES'
RETURN r
- `tck.clauses.match-where.matchwhere4.scenario-2`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 93..111; query:
MATCH (a), (b)
WHERE a.id = 0
  AND (a)-[:T]->(b:TheLabel)
  OR (a)-[:T*]->(b:MissingLabel)
RETURN DISTINCT b
- `tck.clauses.match-where.matchwhere6.scenario-1`: query execution failed: Parse error: no such column: b3; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 62..76; query:
MATCH (a)-->(b)
WHERE b:B
OPTIONAL MATCH (a)-->(c)
WHERE c:C
RETURN a.name
- `tck.clauses.match-where.matchwhere6.scenario-2`: query execution failed: Parse error: no such column: b3; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 65..74; query:
MATCH (n:Single)
OPTIONAL MATCH (n)-[r]-(m)
WHERE m:NonExistent
RETURN r
- `tck.clauses.match-where.matchwhere6.scenario-3`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["<null>"]]
- `tck.clauses.match-where.matchwhere6.scenario-4`: expected [["Mark"]], observed [["<null>"], ["Mark"]]
- `tck.clauses.match-where.matchwhere6.scenario-5`: query execution failed: Parse error: duplicate variable `r` at byte 63..64; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 63..64; query:
MATCH (a1)-[r]->()
WITH r, a1
  LIMIT 1
OPTIONAL MATCH (a2)<-[r]-(b2)
WHERE a1 = a2
RETURN a1, r, b2, a2
- `tck.clauses.match-where.matchwhere6.scenario-7`: expected [["(:X {val: 1})", "(:Y {val: 2})", "(:Z {val: 3})"], ["(:X {val: 4})", "<null>", "<null>"], ["(:X {val: 6})", "<null>", "<null>"]], observed [["(:X {val: 1})", "(:Y {val: 2})", "(:Z {val: 3})"], ["(:X {val: 4})", "(:Y {val: 5})", "<null>"], ["(:X {val: 6})", "<null>", "<null>"]]
- `tck.clauses.merge.merge1.scenario-1`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..11; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MERGE (a)
RETURN count(*) AS n
- `tck.clauses.merge.merge1.scenario-2`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..20; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MERGE (a:TheLabel)
RETURN labels(a)
- `tck.clauses.merge.merge1.scenario-3`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..20; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MERGE (a:TheLabel)
RETURN a.id
- `tck.clauses.merge.merge1.scenario-7`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..13; mutation execution failed: MERGE requires at least one property to identify the entity; query:
CREATE (:X)
CREATE (:X)
MERGE (:X)
- `tck.clauses.merge.merge1.scenario-8`: side effect +nodes expected 1, observed 0
- `tck.clauses.merge.merge1.scenario-9`: side effect +nodes expected 15, observed 6
- `tck.clauses.merge.merge1.scenario-10`: side effect +nodes expected 1, observed 0
- `tck.clauses.merge.merge1.scenario-13`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..24; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q; query:
MERGE p = (a {num: 1})
RETURN p
- `tck.clauses.merge.merge1.scenario-14`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 13..22; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MATCH (a:A)
DELETE a
MERGE (a2:A)
RETURN a2.num
- `tck.clauses.merge.merge2.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.merge.merge2.scenario-2`: expected EOI, UNION, clause, or relationship_pattern at byte 12..12
- `tck.clauses.merge.merge2.scenario-3`: expected EOI, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.merge.merge2.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.merge.merge2.scenario-5`: expected EOI, UNION, clause, or relationship_pattern at byte 42..42
- `tck.clauses.merge.merge3.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 12..12
- `tck.clauses.merge.merge3.scenario-2`: expected EOI, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.merge.merge3.scenario-3`: expected EOI, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.merge.merge3.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 42..42
- `tck.clauses.merge.merge4.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 23..23
- `tck.clauses.merge.merge4.scenario-2`: expected EOI, UNION, clause, or relationship_pattern at byte 42..42
- `tck.clauses.merge.merge5.scenario-3`: expected [["2"]], observed [["1"]]
- `tck.clauses.merge.merge5.scenario-9`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..13; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MERGE (a:A)
MERGE (b:B)
MERGE (a)-[:FOO]->(b)
- `tck.clauses.merge.merge5.scenario-10`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..20; mutation execution failed: graph mutation database operation failed: Parse error: no such table: q; query:
MERGE (a {num: 1})
MERGE (b {num: 2})
MERGE p = (a)-[:R]->(b)
RETURN p
- `tck.clauses.merge.merge5.scenario-11`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..33; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 64..76; query:
CREATE (a {id: 2}), (b {id: 1})
MERGE (a)-[r:KNOWS]-(b)
RETURN startNode(r).id AS s, endNode(r).id AS e
- `tck.clauses.merge.merge5.scenario-12`: expected [["[:KNOWS]"]], observed [["2"]]
- `tck.clauses.merge.merge5.scenario-13`: expected [["[:KNOWS{name:\"ab\"}]"], ["[:KNOWS{name:\"cd\"}]"]], observed [["1"], ["3"]]
- `tck.clauses.merge.merge5.scenario-14`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..25; mutation execution failed: graph mutation database operation failed: Parse error: no such function: split; query:
CREATE (a:Foo), (b:Bar)
WITH a, b
UNWIND ['a,b', 'a,b'] AS str
WITH a, b, split(str, ',') AS roles
MERGE (a)-[r:FB {foobar: roles}]->(b)
RETURN count(*)
- `tck.clauses.merge.merge5.scenario-18`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 41..60; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MATCH (n)
MATCH (m)
WITH n AS a, m AS b
MERGE (a)-[:T]->(b)
WITH a AS x, b AS y
MERGE (a)
MERGE (b)
MERGE (a)-[:T]->(b)
RETURN x.id AS x, y.id AS y
- `tck.clauses.merge.merge5.scenario-19`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 23..33; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MATCH (n)
WITH n AS a
MERGE (c)
MERGE (a)-[:T]->(c)
WITH a AS x
MERGE (c)
MERGE (x)-[:T]->(c)
RETURN x.id AS x
- `tck.clauses.merge.merge5.scenario-20`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 37..56; mutation execution failed: MERGE requires at least one property to identify the entity; query:
MATCH (a:A)-[ab]->(b:B)-[bc]->(c:C)
DELETE ab, bc, b, c
MERGE (newB:B {num: 1})
MERGE (a)-[:REL]->(newB)
MERGE (newC:C)
MERGE (newB)-[:REL]->(newC)
- `tck.clauses.merge.merge5.scenario-21`: side effect +relationships expected 1, observed 0
- `tck.clauses.merge.merge6.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 45..45
- `tck.clauses.merge.merge6.scenario-2`: expected EOI, UNION, clause, or relationship_pattern at byte 45..45
- `tck.clauses.merge.merge6.scenario-3`: expected EOI, UNION, clause, or relationship_pattern at byte 65..65
- `tck.clauses.merge.merge6.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 65..65
- `tck.clauses.merge.merge6.scenario-6`: expected EOI, UNION, clause, or relationship_pattern at byte 65..65
- `tck.clauses.merge.merge6.scenario-7`: expected EOI, UNION, clause, or relationship_pattern at byte 63..63
- `tck.clauses.merge.merge7.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 45..45
- `tck.clauses.merge.merge7.scenario-2`: expected EOI, UNION, clause, or relationship_pattern at byte 46..46
- `tck.clauses.merge.merge7.scenario-3`: expected EOI, UNION, clause, or relationship_pattern at byte 45..45
- `tck.clauses.merge.merge7.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 65..65
- `tck.clauses.merge.merge7.scenario-5`: expected EOI, UNION, clause, or relationship_pattern at byte 65..65
- `tck.clauses.merge.merge8.scenario-1`: expected EOI, UNION, clause, or relationship_pattern at byte 45..45
- `tck.clauses.remove.remove1.scenario-5`: expected [["<null>"]], observed []
- `tck.clauses.remove.remove1.scenario-6`: expected [["( {num: 42})"]], observed [["1"]]
- `tck.clauses.remove.remove1.scenario-7`: expected [["0"]], observed [["3"]]
- `tck.clauses.remove.remove2.scenario-1`: expected property_target at byte 17..17
- `tck.clauses.remove.remove2.scenario-2`: expected property_target at byte 17..17
- `tck.clauses.remove.remove2.scenario-3`: expected property_target at byte 17..17
- `tck.clauses.remove.remove2.scenario-4`: expected property_target at byte 17..17
- `tck.clauses.remove.remove2.scenario-5`: expected property_target at byte 39..39
- `tck.clauses.remove.remove3.scenario-8`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-9`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-10`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-11`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-12`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-13`: expected property_target at byte 19..19
- `tck.clauses.remove.remove3.scenario-14`: expected property_target at byte 19..19
- `tck.clauses.return.return1.scenario-1`: expected [["( {numbers: [1, 2, 3]})"]], observed [["( {numbers: '[1,2,3]'})"]]
- `tck.clauses.return.return2.scenario-7`: expected [["[4, 5, 1, 2, 3]"]], observed [["0"]]
- `tck.clauses.return.return2.scenario-9`: expected [["{a: 1, b: 'foo'}"]], observed [["{\"a\":1,\"b\":\"foo\"}"]]
- `tck.clauses.return.return2.scenario-12`: expected [["[(:A), [:T], (:B)]"]], observed [["[1, 1, 2]"]]
- `tck.clauses.return.return2.scenario-13`: expected [["{node1: (:A), node2: (:B), rel: [:T]}"]], observed [["{\"node1\":1,\"rel\":1,\"node2\":2}"]]
- `tck.clauses.return.return2.scenario-14`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 18..27; mutation execution failed: Cypher mutation binding failed: returning deleted entities is not supported in the initial graph slice at byte 34..42; query:
MATCH ()-[r]->()
DELETE r
RETURN type(r)
- `tck.clauses.return.return2.scenario-15`: expected an error but execution succeeded
- `tck.clauses.return.return2.scenario-17`: expected an error but execution succeeded
- `tck.clauses.return.return4.scenario-9`: expected [["42", "42", "{name: 1}"]], observed [["42", "42", "{\"name\":1}"]]
- `tck.clauses.return.return4.scenario-11`: query execution failed: Parse error: unknown variable `likeTime` at byte 117..125; mutation execution failed: Cypher mutation binding failed: unknown variable `likeTime` at byte 117..125; query:
MATCH (person:Person)<--(message)<-[like]-(:Person)
WITH like.creationDate AS likeTime, person AS person
  ORDER BY likeTime, message.id
WITH head(collect({likeTime: likeTime})) AS latestLike, person AS person
RETURN latestLike.likeTime AS likeTime
  ORDER BY likeTime
- `tck.clauses.return.return6.scenario-5`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..35; query:
MATCH (a)
RETURN size(collect(a))
- `tck.clauses.return.return6.scenario-6`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 47..112; query:
MATCH (a {name: 'Andres'})<-[:FATHER]-(child)
RETURN a.name, {foo: a.name='Andres', kids: collect(child.name)}
- `tck.clauses.return.return6.scenario-9`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 17..26; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..36; query:
MATCH ()
RETURN count(*) * 10 AS c
- `tck.clauses.return.return6.scenario-10`: expected [["1", "[()]"]], observed [["1", "[1]"]]
- `tck.clauses.return.return6.scenario-13`: expected [["a", "[\"c\",\"b\"]", "1"]], observed [["a", "[\"b\",\"c\"]", "1"]]
- `tck.clauses.return.return6.scenario-16`: expected [["( {name: 'Michael'})", "( {name: 'Andres'})", "-7"], ["( {name: 'Michael'})", "( {name: 'Peter'})", "0"]], observed [["( {name: 'Michael'})", "( {name: 'Andres'})", "-7"], ["( {name: 'Michael'})", "( {name: 'Michael'})", "6"], ["( {name: 'Michael'})", "( {name: 'Peter'})", "0"]]
- `tck.clauses.return.return6.scenario-18`: expected [], observed [["<null>", "<null>"]]
- `tck.clauses.return.return6.scenario-19`: expected [], observed [["<null>", "<null>"]]
- `tck.clauses.return.return6.scenario-20`: expected an error but execution succeeded
- `tck.clauses.return.return7.scenario-1`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..36; query:
MATCH p = (a:Start)-->(b)
RETURN *
- `tck.clauses.return.return7.scenario-2`: expected an error but execution succeeded
- `tck.clauses.return-orderby.returnorderby1.scenario-9`: expected [["[]"], ["[\"a\"]"], ["[\"a\",1]"], ["[1]"], ["[1,\"a\"]"], ["[1, null]"], ["[null, 1]"], ["[null, 2]"]], observed [["[\"a\",1]"], ["[\"a\"]"], ["[1,\"a\"]"], ["[1, null]"], ["[1]"], ["[]"], ["[null, 1]"], ["[null, 2]"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-10`: expected [["[null, 2]"], ["[null, 1]"], ["[1, null]"], ["[1,\"a\"]"], ["[1]"], ["[\"a\",1]"], ["[\"a\"]"], ["[]"]], observed [["[null, 2]"], ["[null, 1]"], ["[]"], ["[1]"], ["[1, null]"], ["[1,\"a\"]"], ["[\"a\"]"], ["[\"a\",1]"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-11`: expected [["{a: 'map'}"], ["(:N)"], ["[:REL]"], ["[\"list\"]"], ["()>"], ["text"], ["0"], ["1.5"], ["NaN"], ["<null>"]], observed [["0"], ["1"], ["1"], ["1.5"], ["[\"list\"]"], ["text"], ["{\"a\":\"map\"}"], ["{\"nodes\":[1,2],\"relationships\":[1]}"], ["<null>"], ["<null>"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-12`: expected [["<null>"], ["NaN"], ["1.5"], ["0"], ["text"], ["()>"], ["[\"list\"]"], ["[:REL]"], ["(:N)"], ["{a: 'map'}"]], observed [["<null>"], ["<null>"], ["{\"nodes\":[1,2],\"relationships\":[1]}"], ["{\"a\":\"map\"}"], ["text"], ["[\"list\"]"], ["1.5"], ["1"], ["1"], ["0"]]
- `tck.clauses.return-orderby.returnorderby2.scenario-3`: query execution failed: Parse error: misuse of aggregate: max(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..63; query:
MATCH (n)
RETURN n.division, max(n.age)
  ORDER BY max(n.age)
- `tck.clauses.return-orderby.returnorderby2.scenario-6`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 39..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..48; query:
MATCH (a)
RETURN a, count(*)
ORDER BY count(*)
- `tck.clauses.return-orderby.returnorderby2.scenario-9`: query execution failed: Parse error: unknown variable `id` at byte 49..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..57; query:
MATCH (n)
RETURN DISTINCT n.id AS id
  ORDER BY id DESC
- `tck.clauses.return-orderby.returnorderby2.scenario-11`: query execution failed: Parse error: unknown variable `x` at byte 70..72; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 20..72; query:
MATCH (a:A), (b:X)
RETURN count(a) * 10 + count(b) * 5 AS x
ORDER BY x
- `tck.clauses.return-orderby.returnorderby2.scenario-12`: query execution failed: Parse error: unknown variable `l` at byte 83..85; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 24..85; query:
MATCH p = (a)-[*]->(b)
RETURN collect(nodes(p)) AS paths, length(p) AS l
ORDER BY l
- `tck.clauses.return-orderby.returnorderby2.scenario-13`: expected an error but execution succeeded
- `tck.clauses.return-orderby.returnorderby3.scenario-1`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 48..57; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..78; query:
MATCH (n)
RETURN n.division, count(*)
ORDER BY count(*) DESC, n.division ASC
- `tck.clauses.return-orderby.returnorderby5.scenario-1`: expected [["-5"], ["1"], ["3"]], observed [["1"], ["3"], ["-5"]]
- `tck.clauses.return-orderby.returnorderby6.scenario-1`: query execution failed: Parse error: misuse of aggregate: avg(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..88; query:
MATCH (person)
RETURN avg(person.age) AS avgAge
ORDER BY $age + avg(person.age) - 1000
- `tck.clauses.return-orderby.returnorderby6.scenario-2`: query execution failed: Parse error: unknown variable `age` at byte 88..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..114; query:
MATCH (me: Person)--(you: Person)
RETURN me.age AS age, count(you.age) AS cnt
ORDER BY age, age + count(you.age)
- `tck.clauses.return-orderby.returnorderby6.scenario-3`: query execution failed: Parse error: misuse of aggregate: count(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..112; query:
MATCH (me: Person)--(you: Person)
RETURN me.age AS age, count(you.age) AS cnt
ORDER BY me.age + count(you.age)
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-6`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-7`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-11`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-7`: query execution failed: Parse error: unknown variable `x` at byte 44..46; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 13..61; query:
MATCH (foo)
RETURN foo.num AS x
  ORDER BY x DESC
  LIMIT 4
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-10`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-12`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-13`: expected an error but execution succeeded
- `tck.clauses.set.set1.scenario-1`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set1.scenario-2`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set1.scenario-3`: expected identifier at byte 16..16
- `tck.clauses.set.set1.scenario-4`: expected identifier at byte 25..25
- `tck.clauses.set.set1.scenario-5`: expected [["[0.5, 1.0, 1.5]"]], observed [["[0, 1, 1]"]]
- `tck.clauses.set.set1.scenario-6`: expected [["[1, 2, 3, 4, 5]"]], observed [["0"]]
- `tck.clauses.set.set1.scenario-7`: expected [["[1, 2, 3, 4, 5]"]], observed [["0"]]
- `tck.clauses.set.set1.scenario-8`: expected [["<null>"]], observed []
- `tck.clauses.set.set1.scenario-10`: expected an error but execution succeeded
- `tck.clauses.set.set1.scenario-11`: expected [["(:X {name: 'A', name2: 'B', num: 5})"]], observed [["1"]]
- `tck.clauses.set.set2.scenario-1`: expected [["(:A {property2: 46})"]], observed [["1"]]
- `tck.clauses.set.set2.scenario-2`: expected [["(:A {age: 35})"]], observed [["1"]]
- `tck.clauses.set.set2.scenario-3`: expected [["[:REL {property2: 24}]"]], observed [["1"]]
- `tck.clauses.set.set3.scenario-1`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-2`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-3`: expected property_target at byte 16..16
- `tck.clauses.set.set3.scenario-4`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-5`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-6`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-7`: expected property_target at byte 14..14
- `tck.clauses.set.set3.scenario-8`: expected property_target at byte 36..36
- `tck.clauses.set.set4.scenario-1`: expected property_target at byte 16..16
- `tck.clauses.set.set4.scenario-2`: expected property_target at byte 28..28
- `tck.clauses.set.set4.scenario-3`: expected property_target at byte 28..28
- `tck.clauses.set.set4.scenario-4`: expected property_target at byte 28..28
- `tck.clauses.set.set4.scenario-5`: expected property_target at byte 36..36
- `tck.clauses.set.set5.scenario-1`: expected property_target at byte 36..36
- `tck.clauses.set.set5.scenario-2`: expected property_target at byte 28..28
- `tck.clauses.set.set5.scenario-3`: expected property_target at byte 28..28
- `tck.clauses.set.set5.scenario-4`: expected property_target at byte 28..28
- `tck.clauses.set.set5.scenario-5`: expected property_target at byte 28..28
- `tck.clauses.set.set6.scenario-1`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-2`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-3`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-4`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-5`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-6`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-7`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-8`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-9`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-10`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-11`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-12`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-13`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-14`: expected property_target at byte 16..16
- `tck.clauses.set.set6.scenario-15`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-16`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-17`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-18`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-19`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-20`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-21`: side effect +properties expected 5, observed 0
- `tck.clauses.unwind.unwind1.scenario-3`: expected [["1"], ["2"], ["3"], ["4"], ["5"], ["6"]], observed [["0"]]
- `tck.clauses.unwind.unwind1.scenario-5`: query execution failed: Parse error: property access requires a node or relationship at byte 66..70; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..74; query:
MATCH (row)
WITH collect(row) AS rows
UNWIND rows AS node
RETURN node.id
- `tck.clauses.unwind.unwind1.scenario-6`: TCK parameter value is not representable by the generic adapter
- `tck.clauses.unwind.unwind1.scenario-12`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 86..90; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 86..90; query:
MATCH (a:S)-[:X]->(b1)
WITH a, collect(b1) AS bees
UNWIND bees AS b2
MATCH (a)-[:Y]->(b2)
RETURN a, b2
- `tck.clauses.unwind.unwind1.scenario-13`: expected [["1", "[1, 2]", "3", "[3, 4]", "5", "[5, 6]"], ["1", "[1, 2]", "3", "[3, 4]", "6", "[5, 6]"], ["1", "[1, 2]", "4", "[3, 4]", "5", "[5, 6]"], ["1", "[1, 2]", "4", "[3, 4]", "6", "[5, 6]"], ["2", "[1, 2]", "3", "[3, 4]", "5", "[5, 6]"], ["2", "[1, 2]", "3", "[3, 4]", "6", "[5, 6]"], ["2", "[1, 2]", "4", "[3, 4]", "5", "[5, 6]"], ["2", "[1, 2]", "4", "[3, 4]", "6", "[5, 6]"]], observed [["[1, 2]", "[3, 4]", "[5, 6]", "1", "3", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "3", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "4", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "4", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "3", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "3", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "4", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "4", "6"]]
- `tck.clauses.unwind.unwind1.scenario-14`: TCK parameter value is not representable by the generic adapter
- `tck.clauses.with.with1.scenario-1`: expected [["(:A)", "(:B)"]], observed [["(:A)", "[:REL]", "(:B)"]]
- `tck.clauses.with.with1.scenario-2`: expected [["(:A)", "(:B)", "(:X)"]], observed [["(:A)", "(:X)", "[:REL]", "(:B)"]]
- `tck.clauses.with.with1.scenario-3`: query execution failed: Parse error: duplicate variable `r2` at byte 45..47; mutation execution failed: Cypher mutation binding failed: duplicate variable `r2` at byte 45..47; query:
MATCH ()-[r1]->(:X)
WITH r1 AS r2
MATCH ()-[r2]->()
RETURN r2 AS rel
- `tck.clauses.with.with1.scenario-4`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..31; query:
MATCH p = (a)
WITH p
RETURN p
- `tck.clauses.with.with2.scenario-2`: query execution failed: Parse error: property access requires a node or relationship at byte 49..58; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..70; query:
WITH {name: {name2: 'baz'}} AS nestedMap
RETURN nestedMap.name.name2
- `tck.clauses.with.with3.scenario-1`: query execution failed: Parse error: duplicate variable `r` at byte 46..47; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 46..47; query:
MATCH (a)-[r]->(b:X)
WITH a, r, b
MATCH (a)-[r]->(b)
RETURN r AS rel
  ORDER BY rel.id
- `tck.clauses.with.with4.scenario-6`: query execution failed: Parse error: unknown variable `likeTime` at byte 117..125; mutation execution failed: Cypher mutation binding failed: unknown variable `likeTime` at byte 117..125; query:
MATCH (person:Person)<--(message)<-[like]-(:Person)
WITH like.creationDate AS likeTime, person AS person
  ORDER BY likeTime, message.id
WITH head(collect({likeTime: likeTime})) AS latestLike, person AS person
WITH latestLike.likeTime AS likeTime
  ORDER BY likeTime
RETURN likeTime
- `tck.clauses.with.with4.scenario-7`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..20; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 58..59; query:
CREATE (m {id: 0})
WITH {first: m.id} AS m
WITH {second: m.first} AS m
RETURN m.second
- `tck.clauses.with.with6.scenario-2`: query execution failed: Parse error: duplicate variable `r2` at byte 60..62; mutation execution failed: Cypher mutation binding failed: duplicate variable `r2` at byte 60..62; query:
MATCH ()-[r1]->(:X)
WITH r1 AS r2, count(*) AS c
MATCH ()-[r2]->()
RETURN r2 AS rel
- `tck.clauses.with.with6.scenario-3`: query execution failed: Parse error: duplicate variable `r2` at byte 69..71; mutation execution failed: Cypher mutation binding failed: duplicate variable `r2` at byte 69..71; query:
MATCH (a)-[r1]->(b:X)
WITH a, r1 AS r2, b, count(*) AS c
MATCH (a)-[r2]->(b)
RETURN r2 AS rel
- `tck.clauses.with.with6.scenario-4`: query execution failed: Parse error: no such column: b1; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..78; query:
MATCH p = ()-[*]->()
WITH count(*) AS count, p AS p
RETURN nodes(p) AS nodes
- `tck.clauses.with.with6.scenario-6`: expected [], observed [["<null>", "<null>"]]
- `tck.clauses.with.with6.scenario-7`: expected [], observed [["<null>", "<null>"]]
- `tck.clauses.with.with6.scenario-8`: expected an error but execution succeeded
- `tck.clauses.with.with7.scenario-1`: query execution failed: Parse error: duplicate variable `r` at byte 92..93; mutation execution failed: Cypher mutation binding failed: duplicate variable `r` at byte 92..93; query:
MATCH (a:A)-[r:REL]->(b:B)
WITH a AS b, b AS tmp, r AS r
WITH b AS a, r
LIMIT 1
MATCH (a)-[r]->(b)
RETURN a, r, b
- `tck.clauses.with-orderby.withorderby1.scenario-9`: expected [["[]"], ["[\"a\"]"], ["[\"a\",1]"], ["[1]"]], observed [["[\"a\",1]"], ["[\"a\"]"], ["[1,\"a\"]"], ["[1, null]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-10`: expected [["[null, 2]"], ["[null, 1]"], ["[1, null]"], ["[1,\"a\"]"]], observed [["[null, 2]"], ["[null, 1]"], ["[]"], ["[1]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-15`: expected [["12:35:15+05:00"], ["12:30:14.645876123+01:01"], ["12:31:14.645876123+01:00"]], observed [["10:35-08:00"], ["12:30:14.645876123+01:01"], ["12:31:14.645876123+01:00"]]
- `tck.clauses.with-orderby.withorderby1.scenario-16`: expected [["10:35-08:00"], ["12:31:14.645876124+01:00"], ["12:31:14.645876123+01:00"]], observed [["12:35:15+05:00"], ["12:31:14.645876124+01:00"], ["12:31:14.645876123+01:00"]]
- `tck.clauses.with-orderby.withorderby1.scenario-19`: expected [["0001-01-01T01:01:01.000000001-11:59"], ["1980-12-11T12:31:14-11:59"], ["1984-10-11T12:31:14.645876123+00:17"]], observed [["0001-01-01T01:01:01.000000001-11:59"], ["1980-12-11T12:31:14-11:59"], ["1984-10-11T12:30:14.000000012+00:15"]]
- `tck.clauses.with-orderby.withorderby1.scenario-20`: expected [["9999-09-09T09:59:59.999999999+11:59"], ["1984-10-11T12:30:14.000000012+00:15"], ["1984-10-11T12:31:14.645876123+00:17"]], observed [["9999-09-09T09:59:59.999999999+11:59"], ["1984-10-11T12:31:14.645876123+00:17"], ["1984-10-11T12:30:14.000000012+00:15"]]
- `tck.clauses.with-orderby.withorderby1.scenario-21`: expected [["()>"], ["(:N)"], ["[\"list\"]"], ["[:REL]"], ["{a: 'map'}"]], observed [["0"], ["1"], ["1"], ["1.5"], ["[\"list\"]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-22`: expected [["0"], ["1.5"], ["<null>"], ["NaN"], ["text"]], observed [["<null>"], ["<null>"], ["text"], ["{\"a\":\"map\"}"], ["{\"nodes\":[1,2],\"relationships\":[1]}"]]
- `tck.clauses.with-orderby.withorderby1.scenario-31.examples-1-row-1`: expected [["(:A {list: [2, -2]})", "[2, -2]"], ["(:B {list: [1, 2]})", "[1, 2]"], ["(:D {list: [1, -20]})", "[1, -20]"]], observed [["(:B {list: '[1,2]'})", "[1, 2]"], ["(:D {list: '[1,-20]'})", "[1, -20]"], ["(:E {list: '[2,-2,100]'})", "[2, -2, 100]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-31.examples-1-row-2`: expected [["(:A {list: [2, -2]})", "[2, -2]"], ["(:B {list: [1, 2]})", "[1, 2]"], ["(:D {list: [1, -20]})", "[1, -20]"]], observed [["(:B {list: '[1,2]'})", "[1, 2]"], ["(:D {list: '[1,-20]'})", "[1, -20]"], ["(:E {list: '[2,-2,100]'})", "[2, -2, 100]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-31.examples-1-row-3`: expected [["(:A {list: [2, -2]})", "[2, -2]"], ["(:B {list: [1, 2]})", "[1, 2]"], ["(:D {list: [1, -20]})", "[1, -20]"]], observed [["(:B {list: '[1,2]'})", "[1, 2]"], ["(:D {list: '[1,-20]'})", "[1, -20]"], ["(:E {list: '[2,-2,100]'})", "[2, -2, 100]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-32.examples-1-row-1`: expected [["(:A {list: [2, -2]})", "[2, -2]"], ["(:C {list: [300, 0]})", "[300, 0]"], ["(:E {list: [2, -2, 100]})", "[2, -2, 100]"]], observed [["(:A {list: '[2,-2]'})", "[2, -2]"], ["(:C {list: '[300,0]'})", "[300, 0]"], ["(:E {list: '[2,-2,100]'})", "[2, -2, 100]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-32.examples-1-row-2`: expected [["(:A {list: [2, -2]})", "[2, -2]"], ["(:C {list: [300, 0]})", "[300, 0]"], ["(:E {list: [2, -2, 100]})", "[2, -2, 100]"]], observed [["(:A {list: '[2,-2]'})", "[2, -2]"], ["(:C {list: '[300,0]'})", "[300, 0]"], ["(:E {list: '[2,-2,100]'})", "[2, -2, 100]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-37.examples-1-row-1`: expected [["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:D {time: '12:35:15+05:00'})", "12:35:15+05:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]], observed [["(:A {time: '10:35-08:00'})", "10:35-08:00"], ["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]]
- `tck.clauses.with-orderby.withorderby1.scenario-37.examples-1-row-2`: expected [["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:D {time: '12:35:15+05:00'})", "12:35:15+05:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]], observed [["(:A {time: '10:35-08:00'})", "10:35-08:00"], ["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]]
- `tck.clauses.with-orderby.withorderby1.scenario-37.examples-1-row-3`: expected [["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:D {time: '12:35:15+05:00'})", "12:35:15+05:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]], observed [["(:A {time: '10:35-08:00'})", "10:35-08:00"], ["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:E {time: '12:30:14.645876123+01:01'})", "12:30:14.645876123+01:01"]]
- `tck.clauses.with-orderby.withorderby1.scenario-38.examples-1-row-1`: expected [["(:A {time: '10:35-08:00'})", "10:35-08:00"], ["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:C {time: '12:31:14.645876124+01:00'})", "12:31:14.645876124+01:00"]], observed [["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:C {time: '12:31:14.645876124+01:00'})", "12:31:14.645876124+01:00"], ["(:D {time: '12:35:15+05:00'})", "12:35:15+05:00"]]
- `tck.clauses.with-orderby.withorderby1.scenario-38.examples-1-row-2`: expected [["(:A {time: '10:35-08:00'})", "10:35-08:00"], ["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:C {time: '12:31:14.645876124+01:00'})", "12:31:14.645876124+01:00"]], observed [["(:B {time: '12:31:14.645876123+01:00'})", "12:31:14.645876123+01:00"], ["(:C {time: '12:31:14.645876124+01:00'})", "12:31:14.645876124+01:00"], ["(:D {time: '12:35:15+05:00'})", "12:35:15+05:00"]]
- `tck.clauses.with-orderby.withorderby1.scenario-41.examples-1-row-1`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})", "1984-10-11T12:31:14.645876123+00:17"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})", "1984-10-11T12:30:14.000000012+00:15"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]]
- `tck.clauses.with-orderby.withorderby1.scenario-41.examples-1-row-2`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})", "1984-10-11T12:31:14.645876123+00:17"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})", "1984-10-11T12:30:14.000000012+00:15"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]]
- `tck.clauses.with-orderby.withorderby1.scenario-41.examples-1-row-3`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})", "1984-10-11T12:31:14.645876123+00:17"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})", "1984-10-11T12:30:14.000000012+00:15"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})", "0001-01-01T01:01:01.000000001-11:59"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})", "1980-12-11T12:31:14-11:59"]]
- `tck.clauses.with-orderby.withorderby2.scenario-7.examples-1-row-1`: expected [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:D {name: 'sit', title: 'dr.'})"]], observed [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:C {name: 'dolor', title: 'prof.'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-7.examples-1-row-2`: expected [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:D {name: 'sit', title: 'dr.'})"]], observed [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:C {name: 'dolor', title: 'prof.'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-7.examples-1-row-3`: expected [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:D {name: 'sit', title: 'dr.'})"]], observed [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:C {name: 'dolor', title: 'prof.'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-8.examples-1-row-1`: expected [["(:C {name: 'dolor', title: 'prof.'})"], ["(:D {name: 'sit', title: 'dr.'})"], ["(:E {name: 'amet', title: 'prof.'})"]], observed [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:C {name: 'dolor', title: 'prof.'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-8.examples-1-row-2`: expected [["(:C {name: 'dolor', title: 'prof.'})"], ["(:D {name: 'sit', title: 'dr.'})"], ["(:E {name: 'amet', title: 'prof.'})"]], observed [["(:A {name: 'lorem', title: 'dr.'})"], ["(:B {name: 'ipsum', title: 'dr.'})"], ["(:C {name: 'dolor', title: 'prof.'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-1`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-2`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-3`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-10.examples-1-row-1`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:D {list: [1, -20], list2: [4, -2]})"], ["(:E {list: [2, -2, 100], list2: [5, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-10.examples-1-row-2`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:D {list: [1, -20], list2: [4, -2]})"], ["(:E {list: [2, -2, 100], list2: [5, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-11.examples-1-row-1`: expected [["(:A {date: '1910-05-06'})"], ["(:E {date: '1980-10-24'})"]], observed [["(:A {date: '1910-05-06'})"], ["(:B {date: '1980-12-24'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-11.examples-1-row-2`: expected [["(:A {date: '1910-05-06'})"], ["(:E {date: '1980-10-24'})"]], observed [["(:A {date: '1910-05-06'})"], ["(:B {date: '1980-12-24'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-11.examples-1-row-3`: expected [["(:A {date: '1910-05-06'})"], ["(:E {date: '1980-10-24'})"]], observed [["(:A {date: '1910-05-06'})"], ["(:B {date: '1980-12-24'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-13.examples-1-row-1`: expected [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:D {time: '12:30:14.645876123'})"]], observed [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-13.examples-1-row-2`: expected [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:D {time: '12:30:14.645876123'})"]], observed [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-13.examples-1-row-3`: expected [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:D {time: '12:30:14.645876123'})"]], observed [["(:A {time: '10:35'})"], ["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-14.examples-1-row-1`: expected [["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"], ["(:E {time: '12:31:15'})"]], observed [["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"], ["(:D {time: '12:30:14.645876123'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-14.examples-1-row-2`: expected [["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"], ["(:E {time: '12:31:15'})"]], observed [["(:B {time: '12:31:14.645876123'})"], ["(:C {time: '12:31:14.645876124'})"], ["(:D {time: '12:30:14.645876123'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-1`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-2`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-3`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-16.examples-1-row-1`: expected [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]], observed [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"], ["(:D {time: '12:35:15+05:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-16.examples-1-row-2`: expected [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]], observed [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"], ["(:D {time: '12:35:15+05:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-1`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-2`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-3`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-21.examples-1-row-2`: expected [["C"], ["C"]], observed [["A"], ["A"]]
- `tck.clauses.with-orderby.withorderby2.scenario-22.examples-1-row-2`: expected [["C", "2"]], observed [["A", "2"]]
- `tck.clauses.with-orderby.withorderby2.scenario-23.examples-1-row-2`: expected [["C", "2"]], observed [["A", "2"]]
- `tck.clauses.with-orderby.withorderby4.scenario-2`: query execution failed: Parse error: unknown variable `sum` at byte 54..60; mutation execution failed: Cypher mutation binding failed: unknown variable `sum` at byte 54..60; query:
MATCH (a:A)
WITH a, a.num + a.num2 AS sum
  ORDER BY sum
  LIMIT 3
RETURN a, sum
- `tck.clauses.with-orderby.withorderby4.scenario-4`: query execution failed: Parse error: unknown variable `sum` at byte 85..91; mutation execution failed: Cypher mutation binding failed: unknown variable `sum` at byte 85..91; query:
MATCH (a:A)
WITH a, a.num + a.num2 AS sum, a.num2 % 3 AS mod
  ORDER BY a.num2 % 3, sum
  LIMIT 3
RETURN a, sum, mod
- `tck.clauses.with-orderby.withorderby4.scenario-5`: query execution failed: Parse error: unknown variable `mod` at byte 73..76; mutation execution failed: Cypher mutation binding failed: unknown variable `mod` at byte 73..76; query:
MATCH (a:A)
WITH a, a.num + a.num2 AS sum, a.num2 % 3 AS mod
  ORDER BY mod, a.num + a.num2
  LIMIT 3
RETURN a, sum, mod
- `tck.clauses.with-orderby.withorderby4.scenario-6`: query execution failed: Parse error: unknown variable `mod` at byte 73..76; mutation execution failed: Cypher mutation binding failed: unknown variable `mod` at byte 73..76; query:
MATCH (a:A)
WITH a, a.num + a.num2 AS sum, a.num2 % 3 AS mod
  ORDER BY mod, sum
  LIMIT 3
RETURN a, sum, mod
- `tck.clauses.with-orderby.withorderby4.scenario-7`: expected [["(:A {num: 1, num2: 4})", "5"], ["(:A {num: 3, num2: 3})", "6"], ["(:A {num: 5, num2: 2})", "7"]], observed [["(:A {num: 1, num2: 4})", "5"], ["(:A {num: 3, num2: 3})", "6"], ["(:A {num: 9, num2: 0})", "9"]]
- `tck.clauses.with-orderby.withorderby4.scenario-9`: expected [["0"], ["0"], ["1"]], observed [["0"], ["1"], ["2"]]
- `tck.clauses.with-orderby.withorderby4.scenario-10`: expected [["1"], ["1"], ["2"]], observed [["0"], ["1"], ["2"]]
- `tck.clauses.with-orderby.withorderby4.scenario-11`: query execution failed: Parse error: misuse of aggregate: sum(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 105..121; query:
MATCH (a:A)
WITH a.num2 % 3 AS mod, sum(a.num + a.num2) AS sum
  ORDER BY sum(a.num + a.num2)
  LIMIT 2
RETURN mod, sum
- `tck.clauses.with-orderby.withorderby4.scenario-12`: query execution failed: Parse error: unknown variable `sum` at byte 75..81; mutation execution failed: Cypher mutation binding failed: unknown variable `sum` at byte 75..81; query:
MATCH (a:A)
WITH a.num2 % 3 AS mod, sum(a.num + a.num2) AS sum
  ORDER BY sum
  LIMIT 2
RETURN mod, sum
- `tck.clauses.with-orderby.withorderby4.scenario-15`: query execution failed: Parse error: unknown variable `c` at byte 61..63; mutation execution failed: Cypher mutation binding failed: unknown variable `c` at byte 61..63; query:
MATCH (a)-[r]->(b:X)
WITH a, r, b, count(*) AS c
  ORDER BY c
MATCH (a)-[r]->(b)
RETURN r AS rel
  ORDER BY rel.id
- `tck.clauses.with-orderby.withorderby4.scenario-16`: query execution failed: Parse error: misuse of aggregate: avg(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 86..100; query:
MATCH (person)
WITH avg(person.age) AS avgAge
ORDER BY $age + avg(person.age) - 1000
RETURN avgAge
- `tck.clauses.with-orderby.withorderby4.scenario-17`: query execution failed: Parse error: unknown variable `age` at byte 86..89; mutation execution failed: Cypher mutation binding failed: unknown variable `age` at byte 86..89; query:
MATCH (me: Person)--(you: Person)
WITH me.age AS age, count(you.age) AS cnt
ORDER BY age, age + count(you.age)
RETURN age
- `tck.clauses.with-orderby.withorderby4.scenario-18`: query execution failed: Parse error: misuse of aggregate: count(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 110..121; query:
MATCH (me: Person)--(you: Person)
WITH me.age AS age, count(you.age) AS cnt
ORDER BY me.age + count(you.age)
RETURN age
- `tck.clauses.with-skip-limit.withskiplimit1.scenario-1`: query execution failed: Parse error: unknown variable `property` at byte 64..75; mutation execution failed: Cypher mutation binding failed: unknown variable `property` at byte 64..75; query:
MATCH (a)
WITH a.name AS property, a.num AS idToUse
  ORDER BY property
  SKIP 1
MATCH (b)
WHERE b.id = idToUse
RETURN DISTINCT b
- `tck.clauses.with-skip-limit.withskiplimit1.scenario-2`: query execution failed: Parse error: unknown variable `c` at byte 56..58; mutation execution failed: Cypher mutation binding failed: unknown variable `c` at byte 56..58; query:
MATCH ()-[r1]->(x)
WITH x, sum(r1.num) AS c
  ORDER BY c SKIP 1
RETURN x, c
- `tck.clauses.with-skip-limit.withskiplimit2.scenario-3`: expected [["(:B)", "(:A)", "(:X)"]], observed [["(:A)", "(:B)", "[:REL]", "(:X)"]]
- `tck.clauses.with-skip-limit.withskiplimit2.scenario-4`: query execution failed: Parse error: unknown variable `c` at byte 56..58; mutation execution failed: Cypher mutation binding failed: unknown variable `c` at byte 56..58; query:
MATCH ()-[r1]->(x)
WITH x, sum(r1.num) AS c
  ORDER BY c LIMIT 1
RETURN x, c
- `tck.clauses.with-where.withwhere1.scenario-2`: query execution failed: Parse error: unknown variable `a` at byte 47..48; mutation execution failed: Cypher mutation binding failed: unknown variable `a` at byte 47..48; query:
MATCH (a)
WITH DISTINCT a.name2 AS name
WHERE a.name2 = 'B'
RETURN *
- `tck.clauses.with-where.withwhere1.scenario-3`: query execution failed: Parse error: unknown variable `r` at byte 73..75; mutation execution failed: Cypher mutation binding failed: unknown variable `r` at byte 73..75; query:
MATCH (a:A), (other:B)
OPTIONAL MATCH (a)-[r]->(other)
WITH other WHERE r IS NULL
RETURN other
- `tck.clauses.with-where.withwhere1.scenario-4`: query execution failed: Parse error: unknown variable `a` at byte 66..68; mutation execution failed: Cypher mutation binding failed: unknown variable `a` at byte 66..68; query:
MATCH (other:B)
OPTIONAL MATCH (a)-[r]->(other)
WITH other WHERE a IS NULL
RETURN other
- `tck.clauses.with-where.withwhere4.scenario-2`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 103..121; query:
MATCH (a), (b)
WITH a, b
WHERE a.id = 0
  AND (a)-[:T]->(b:TheLabel)
  OR (a)-[:T*]->(b:MissingLabel)
RETURN DISTINCT b
- `tck.clauses.with-where.withwhere7.scenario-1`: query execution failed: Parse error: unknown variable `a` at byte 38..39; mutation execution failed: Cypher mutation binding failed: unknown variable `a` at byte 38..39; query:
MATCH (a)
WITH a.name2 AS name
WHERE a.name2 = 'B'
RETURN *
- `tck.clauses.with-where.withwhere7.scenario-3`: query execution failed: Parse error: unknown variable `a` at byte 52..53; mutation execution failed: Cypher mutation binding failed: unknown variable `a` at byte 52..53; query:
MATCH (a)
WITH a.name2 AS name
WHERE name = 'B' OR a.name2 = 'C'
RETURN *
- `tck.expressions.aggregation.aggregation2.scenario-3`: expected [["2.0"]], observed [["2"]]
- `tck.expressions.aggregation.aggregation2.scenario-9`: expected [["[2, 1]"]], observed [["[2]"]]
- `tck.expressions.aggregation.aggregation2.scenario-11`: expected [["1"]], observed [["b"]]
- `tck.expressions.aggregation.aggregation2.scenario-12`: expected [["[1, 2]"]], observed [["0.2"]]
- `tck.expressions.aggregation.aggregation5.scenario-1`: expected [["()", "[]"]], observed [["()", "[null]"]]
- `tck.expressions.aggregation.aggregation5.scenario-2`: expected [["[]", "[42, 43, 44]"]], observed [["[]", "[]"]]
- `tck.expressions.aggregation.aggregation6.scenario-1.examples-1-row-1`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileDisc(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation6.scenario-1.examples-1-row-2`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileDisc(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation6.scenario-1.examples-1-row-3`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileDisc(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation6.scenario-2.examples-1-row-1`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileCont(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation6.scenario-2.examples-1-row-2`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileCont(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation6.scenario-2.examples-1-row-3`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 11..60; query:
MATCH (n)
RETURN percentileCont(n.price, $percentile) AS p
- `tck.expressions.aggregation.aggregation8.scenario-3`: expected [["[]"]], observed [["[null]"]]
- `tck.expressions.aggregation.aggregation8.scenario-4`: expected [["[1]"]], observed [["[null, 1]"]]
- `tck.expressions.btic.btic1.scenario-1.examples-1-row-1`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..26; query:
RETURN btic('1985') AS b
- `tck.expressions.btic.btic1.scenario-1.examples-1-row-2`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..29; query:
RETURN btic('1985-03') AS b
- `tck.expressions.btic.btic1.scenario-1.examples-1-row-3`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..32; query:
RETURN btic('1985-03-15') AS b
- `tck.expressions.btic.btic1.scenario-2`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..31; query:
RETURN btic('1985/1990') AS b
- `tck.expressions.btic.btic1.scenario-3`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..30; query:
RETURN btic('2020-03/') AS b
- `tck.expressions.btic.btic1.scenario-4`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..27; query:
RETURN btic('~1985') AS b
- `tck.expressions.btic.btic1.scenario-5`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..24; query:
RETURN btic(null) AS b
- `tck.expressions.btic.btic1.scenario-6`: query execution failed: Parse error: no such function: btic_lo; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..46; query:
RETURN toString(btic_lo(btic('1985'))) AS lo
- `tck.expressions.btic.btic1.scenario-7`: query execution failed: Parse error: no such function: btic_hi; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..46; query:
RETURN toString(btic_hi(btic('1985'))) AS hi
- `tck.expressions.btic.btic1.scenario-8`: query execution failed: Parse error: no such function: btic_duration; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..43; query:
RETURN btic_duration(btic('1985')) AS dur
- `tck.expressions.btic.btic1.scenario-9`: query execution failed: Parse error: no such function: btic_granularity; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..44; query:
RETURN btic_granularity(btic('1985')) AS g
- `tck.expressions.btic.btic1.scenario-10`: query execution failed: Parse error: no such function: btic_certainty; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..43; query:
RETURN btic_certainty(btic('~1985')) AS c
- `tck.expressions.btic.btic1.scenario-11`: query execution failed: Parse error: no such function: btic_is_finite; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..93; query:
RETURN btic_is_finite(btic('1985')) AS finite,
       btic_is_finite(btic('/')) AS infinite
- `tck.expressions.btic.btic1.scenario-12`: query execution failed: Parse error: no such function: btic_is_unbounded; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..108; query:
RETURN btic_is_unbounded(btic('2020-03/')) AS unbounded,
       btic_is_unbounded(btic('1985')) AS bounded
- `tck.expressions.btic.btic1.scenario-13`: query execution failed: Parse error: no such function: btic_overlaps; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..135; query:
RETURN btic_overlaps(btic('1985'), btic('1985-06/1986-06')) AS overlaps,
       btic_overlaps(btic('1985'), btic('1990')) AS disjoint
- `tck.expressions.btic.btic1.scenario-14`: query execution failed: Parse error: no such function: btic_contains_point; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..122; query:
RETURN btic_contains_point(btic('1985'), 486000000000) AS inside,
       btic_contains_point(btic('1985'), 0) AS outside
- `tck.expressions.btic.btic1.scenario-15`: query execution failed: Parse error: no such function: btic_before; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..128; query:
RETURN btic_before(btic('1985'), btic('1990')) AS before_result,
       btic_after(btic('1990'), btic('1985')) AS after_result
- `tck.expressions.btic.btic1.scenario-16`: query execution failed: Parse error: no such function: btic_equals; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..109; query:
RETURN btic_equals(btic('1985'), btic('1985')) AS eq,
       btic_equals(btic('1985'), btic('1990')) AS neq
- `tck.expressions.btic.btic1.scenario-17`: query execution failed: Parse error: no such function: btic_span; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..54; query:
RETURN btic_span(btic('1985'), btic('1990')) AS span
- `tck.expressions.btic.btic1.scenario-18`: query execution failed: Parse error: no such function: btic_intersection; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..74; query:
RETURN btic_intersection(btic('1985'), btic('1985-06/1986-06')) AS inter
- `tck.expressions.btic.btic1.scenario-19`: query execution failed: Parse error: no such function: btic_gap; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..52; query:
RETURN btic_gap(btic('1985'), btic('1990')) AS gap
- `tck.expressions.btic.btic1.scenario-20`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..33; mutation execution failed: graph mutation database operation failed: Parse error: no such function: btic; query:
CREATE ({period: btic('1985')})
- `tck.expressions.btic.btic1.scenario-21`: TCK setup query failed: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..47; mutation execution failed: graph mutation database operation failed: Parse error: no such function: btic; query:
CREATE ({name: 'test', period: btic('1985')})
; query:
CREATE ({name: 'test', period: btic('1985')})
- `tck.expressions.btic.btic1.scenario-22`: query execution failed: Parse error: no such function: btic_meets; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..110; query:
RETURN btic_meets(btic('1985'), btic('1986')) AS meets,
       btic_meets(btic('1985'), btic('1990')) AS gap
- `tck.expressions.btic.btic1.scenario-23`: query execution failed: Parse error: no such function: btic_adjacent; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..171; query:
RETURN btic_adjacent(btic('1985'), btic('1986')) AS fwd,
       btic_adjacent(btic('1986'), btic('1985')) AS rev,
       btic_adjacent(btic('1985'), btic('2020')) AS gap
- `tck.expressions.btic.btic1.scenario-24`: query execution failed: Parse error: no such function: btic_disjoint; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..138; query:
RETURN btic_disjoint(btic('1985'), btic('2020')) AS disjoint,
       btic_disjoint(btic('1985'), btic('1985-06/1986-06')) AS overlapping
- `tck.expressions.btic.btic1.scenario-25`: query execution failed: Parse error: no such function: btic; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..260; query:
RETURN btic('1985') < btic('2000') AS lt,
       btic('2000') > btic('1985') AS gt,
       btic('1985') = btic('1985') AS eq,
       btic('1985') <> btic('2000') AS neq,
       btic('1985') <= btic('1985') AS lteq,
       btic('1985') >= btic('1985') AS gteq
- `tck.expressions.btic.btic1.scenario-26`: query execution failed: Parse error: no such function: btic_starts; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..211; query:
RETURN btic_starts(btic('1985-01/1985-06'), btic('1985')) AS starts,
       btic_during(btic('1985-03/1985-09'), btic('1985')) AS during,
       btic_finishes(btic('1985-06/1985-12'), btic('1985')) AS finishes
- `tck.expressions.comparison.comparison1.scenario-3`: expected [], observed [["( {id: 0})"]]
- `tck.expressions.comparison.comparison1.scenario-6.examples-1-row-2`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-6.examples-1-row-5`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-7.examples-1-row-12`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.comparison.comparison1.scenario-7.examples-1-row-13`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-7.examples-1-row-14`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-7.examples-1-row-15`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-7.examples-1-row-16`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-1`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-2`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-3`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-4`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-1`: query execution failed: Parse error: unknown variable `i` at byte 200..202; mutation execution failed: Cypher mutation binding failed: unknown variable `i` at byte 200..202; query:
MATCH p = (n)-[r]->()
WITH [n, r, p, '', 1, 3.14, true, null, [], {}] AS types
UNWIND range(0, size(types) - 1) AS i
UNWIND range(0, size(types) - 1) AS j
WITH types[i] AS lhs, types[j] AS rhs
WHERE i <> j
WITH lhs, rhs, lhs < rhs AS result
WHERE result
RETURN lhs, rhs
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-2`: query execution failed: Parse error: unknown variable `i` at byte 200..202; mutation execution failed: Cypher mutation binding failed: unknown variable `i` at byte 200..202; query:
MATCH p = (n)-[r]->()
WITH [n, r, p, '', 1, 3.14, true, null, [], {}] AS types
UNWIND range(0, size(types) - 1) AS i
UNWIND range(0, size(types) - 1) AS j
WITH types[i] AS lhs, types[j] AS rhs
WHERE i <> j
WITH lhs, rhs, lhs <= rhs AS result
WHERE result
RETURN lhs, rhs
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-3`: query execution failed: Parse error: unknown variable `i` at byte 200..202; mutation execution failed: Cypher mutation binding failed: unknown variable `i` at byte 200..202; query:
MATCH p = (n)-[r]->()
WITH [n, r, p, '', 1, 3.14, true, null, [], {}] AS types
UNWIND range(0, size(types) - 1) AS i
UNWIND range(0, size(types) - 1) AS j
WITH types[i] AS lhs, types[j] AS rhs
WHERE i <> j
WITH lhs, rhs, lhs >= rhs AS result
WHERE result
RETURN lhs, rhs
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-4`: query execution failed: Parse error: unknown variable `i` at byte 200..202; mutation execution failed: Cypher mutation binding failed: unknown variable `i` at byte 200..202; query:
MATCH p = (n)-[r]->()
WITH [n, r, p, '', 1, 3.14, true, null, [], {}] AS types
UNWIND range(0, size(types) - 1) AS i
UNWIND range(0, size(types) - 1) AS j
WITH types[i] AS lhs, types[j] AS rhs
WHERE i <> j
WITH lhs, rhs, lhs > rhs AS result
WHERE result
RETURN lhs, rhs
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-1`: expected [["1"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-2`: expected [["1"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-3`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-4`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-1`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-2`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-3`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-6.examples-1-row-3`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-6.examples-1-row-4`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison3.scenario-1`: expected [["2"]], observed [["1"], ["2"], ["3"]]
- `tck.expressions.comparison.comparison3.scenario-2`: expected [["2"], ["3"]], observed [["1"], ["2"], ["3"]]
- `tck.expressions.comparison.comparison3.scenario-3`: expected [["1"], ["2"]], observed [["1"], ["2"], ["3"]]
- `tck.expressions.comparison.comparison3.scenario-5`: expected [["b"]], observed [["a"], ["b"], ["c"]]
- `tck.expressions.comparison.comparison3.scenario-6`: expected [["b"], ["c"]], observed [["a"], ["b"], ["c"]]
- `tck.expressions.comparison.comparison3.scenario-7`: expected [["a"], ["b"]], observed [["a"], ["b"], ["c"]]
- `tck.expressions.comparison.comparison3.scenario-9`: expected [], observed [["3"]]
- `tck.expressions.comparison.comparison4.scenario-1`: expected [["[\"B\"]"]], observed [["[\"A\"]"], ["[\"B\"]"], ["[\"C\"]"]]
- `tck.expressions.conditional.conditional2.scenario-1.examples-1-row-11`: expected [["something else"]], observed [["one"]]
- `tck.expressions.existentialsubqueries.existentialsubquery1.scenario-4`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..72; query:
MATCH (n) WHERE exists {
  (n)-[r]->() WHERE type(r) = 'NA'
}
RETURN n
- `tck.expressions.existentialsubqueries.existentialsubquery2.scenario-2`: expected WHERE, RETURN, or relationship_pattern at byte 45..45
- `tck.expressions.existentialsubqueries.existentialsubquery3.scenario-1`: expected [["(:A {prop: 1})"]], observed [["(:A {prop: 1})"], ["(:B {prop: 1})"], ["(:C {prop: 2})"], ["(:D {prop: 3})"]]
- `tck.expressions.existentialsubqueries.existentialsubquery3.scenario-2`: expected WHERE, RETURN, or relationship_pattern at byte 65..65
- `tck.expressions.existentialsubqueries.existentialsubquery3.scenario-3`: query execution failed: Parse error: unknown variable `m` at byte 77..78; mutation execution failed: Cypher mutation binding failed: unknown variable `m` at byte 77..78; query:
MATCH (n) WHERE exists {
  MATCH (m) WHERE exists {
    MATCH (l) WHERE (l)(m) RETURN true
  }
  RETURN true
}
RETURN n
- `tck.expressions.graph.graph3.scenario-6`: query execution failed: Parse error: no such function: labels; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..59; query:
MATCH (a)
WITH [a, 1] AS list
RETURN labels(list[0]) AS l
- `tck.expressions.graph.graph3.scenario-7`: query execution failed: Parse error: no such function: labels; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..64; query:
OPTIONAL MATCH (n:DoesNotExist)
RETURN labels(n), labels(null)
- `tck.expressions.graph.graph4.scenario-1`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..33; query:
MATCH ()-[r]->()
RETURN type(r)
- `tck.expressions.graph.graph4.scenario-2`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 28..54; query:
MATCH ()-[r1]->()-[r2]->()
RETURN type(r1), type(r2)
- `tck.expressions.graph.graph4.scenario-3`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 48..75; query:
MATCH (a)
OPTIONAL MATCH (a)-[r:NOT_THERE]->()
RETURN type(r), type(null)
- `tck.expressions.graph.graph4.scenario-4`: expected [["<null>"], ["T"]], observed [["T"], ["T"]]
- `tck.expressions.graph.graph4.scenario-5`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..60; query:
MATCH (a)-[r]->()
WITH [r, 1] AS list
RETURN type(list[0])
- `tck.expressions.graph.graph5.scenario-2`: expected [["[:T1]", "0"], ["[:T2]", "1"], ["[:T3]", "0"], ["[:T4]", "0"], ["[:t2]", "0"]], observed [["[:T1]", "0"], ["[:T2]", "0"], ["[:T3]", "0"], ["[:T4]", "0"], ["[:t2]", "0"]]
- `tck.expressions.graph.graph5.scenario-5`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.graph.graph6.scenario-3`: expected [["<null>"]], observed []
- `tck.expressions.graph.graph6.scenario-4`: query execution failed: Parse error: property access requires a node or relationship at byte 41..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..100; query:
MATCH (n)
WITH [123, n] AS list
RETURN (list[1]).missing, (list[1]).missingToo, (list[1]).existing
- `tck.expressions.graph.graph6.scenario-6`: expected [["<null>", "<null>", "42"]], observed [["<null>", "<null>", "42"], ["<null>", "<null>", "<null>"]]
- `tck.expressions.graph.graph6.scenario-7`: expected [["<null>"]], observed []
- `tck.expressions.graph.graph6.scenario-8`: query execution failed: Parse error: property access requires a node or relationship at byte 48..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 40..107; query:
MATCH ()-[r]->()
WITH [123, r] AS list
RETURN (list[1]).missing, (list[1]).missingToo, (list[1]).existing
- `tck.expressions.graph.graph7.scenario-1`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 32..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 25..56; query:
MATCH (n {name: 'Apa'})
RETURN n['nam' + 'e'] AS value
- `tck.expressions.graph.graph7.scenario-2`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..26; mutation execution failed: Cypher mutation binding failed: indexing this operand/key combination is not supported in the initial graph slice at byte 33..34; query:
CREATE (n {name: 'Apa'})
RETURN n['nam' + 'e'] AS value
- `tck.expressions.graph.graph7.scenario-3`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..26; mutation execution failed: Cypher mutation binding failed: indexing this operand/key combination is not supported in the initial graph slice at byte 33..34; query:
CREATE (n {name: 'Apa'})
RETURN n[$idx] AS value
- `tck.expressions.graph.graph8.scenario-1`: expected [["name"], ["surname"]], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-2`: expected [["name"], ["otherName"], ["otherSurname"], ["surname"]], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-3`: expected [], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-4`: expected [], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-5`: expected [["status"], ["year"]], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-6`: expected [], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-7`: expected [], observed [["<null>"]]
- `tck.expressions.graph.graph8.scenario-8`: expected [["1", "0", "0"]], observed [["0", "0", "0"]]
- `tck.expressions.graph.graph9.scenario-1`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..44; query:
MATCH (p:Person)
RETURN properties(p) AS m
- `tck.expressions.graph.graph9.scenario-2`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 20..46; query:
MATCH ()-[r:R]->()
RETURN properties(r) AS m
- `tck.expressions.graph.graph9.scenario-3`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 70..124; query:
OPTIONAL MATCH (n:DoesNotExist)
OPTIONAL MATCH (n)-[r:NOT_THERE]->()
RETURN properties(n), properties(r), properties(null)
- `tck.expressions.graph.graph9.scenario-4`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..55; query:
RETURN properties({name: 'Popeye', level: 9001}) AS m
- `tck.expressions.list.list1.scenario-3`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-5`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-9.examples-1-row-1`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-9.examples-1-row-2`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-9.examples-1-row-3`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-9.examples-1-row-4`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list1.scenario-9.examples-1-row-5`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.list.list11.scenario-3`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 277..346; query:
WITH 0 AS start, [1, 2, 500, 1000, 1500] AS stopList, [-1000, -3, -2, -1, 1, 2, 3, 1000] AS stepList
UNWIND stopList AS stop
UNWIND stepList AS step
WITH start, stop, step, range(start, stop, step) AS list
WITH start, stop, step, list, sign(stop-start) <> sign(step) AS empty
RETURN ALL(ok IN collect((size(list) = 0) = empty) WHERE ok) AS okay
- `tck.expressions.list.list12.scenario-1`: query execution failed: Parse error: unknown variable `x` at byte 69..70; mutation execution failed: Cypher mutation binding failed: unknown variable `x` at byte 69..70; query:
MATCH (a:Label1)
WITH collect(a) AS nodes
WITH nodes, [x IN nodes | x.name] AS oldNames
UNWIND nodes AS n
SET n.name = 'newName'
RETURN n.name, oldNames
- `tck.expressions.list.list12.scenario-2`: query execution failed: Parse error: unknown variable `x` at byte 73..74; mutation execution failed: Cypher mutation binding failed: unknown variable `x` at byte 73..74; query:
MATCH (a:Label1)
WITH collect(a) AS nodes
WITH nodes, [x IN nodes WHERE x.name = 'original'] AS noopFiltered
UNWIND nodes AS n
SET n.name = 'newName'
RETURN n.name, size(noopFiltered)
- `tck.expressions.list.list12.scenario-3`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..92; query:
MATCH (n)
OPTIONAL MATCH (n)-[r]->(m)
RETURN size([x IN collect(r) WHERE x <> null]) AS cn
- `tck.expressions.list.list12.scenario-4`: query execution failed: Parse error: no such function: collect; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 20..67; query:
MATCH p = (n)-->()
RETURN [x IN collect(p) | head(nodes(x))] AS p
- `tck.expressions.list.list12.scenario-5`: query execution failed: Parse error: no such column: b2; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 82..94; query:
MATCH p = (n:A)-->()
WITH [x IN collect(p) | head(nodes(x))] AS p, count(n) AS c
RETURN p, c
- `tck.expressions.list.list12.scenario-6`: expected [["(:C)"]], observed []
- `tck.expressions.list.list2.scenario-9.examples-1-row-1`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-2`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-3`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-4`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-5`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list3.scenario-4`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.list.list3.scenario-7`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.list.list4.scenario-1`: expected [["[1, 10, 100, 4, 5]"]], observed [["0"]]
- `tck.expressions.list.list4.scenario-2`: expected [["[0, 1, 0]"]], observed [["0"]]
- `tck.expressions.list.list5.scenario-5`: expected [["0"]], observed [["1"]]
- `tck.expressions.list.list5.scenario-21`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.list.list5.scenario-29`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.list.list5.scenario-31`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.list.list5.scenario-34`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.list.list6.scenario-3`: expected [["3"]], observed [["1"]]
- `tck.expressions.list.list6.scenario-7`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 36..36
- `tck.expressions.list.list6.scenario-8`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 34..34
- `tck.expressions.list.list6.scenario-9`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 38..38
- `tck.expressions.list.list6.scenario-10`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 44..44
- `tck.expressions.literals.literals5.scenario-1`: expected [["1.0"]], observed [["1"]]
- `tck.expressions.literals.literals5.scenario-2`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-4`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-5`: expected [["1.2635418652381264e305"]], observed [["126354186523812637083265300874972409152306344844529887549280826072773391668220596519840562718295208487895546677575070099382084697423587658029812212044105366779846072573635712245818261999842901642900582100199277717544304856592188574754249157767617334086173213808419907747092679556434770033100573931636523008.0"]]
- `tck.expressions.literals.literals5.scenario-6`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-7`: expected [["0.0"]], observed [["0"]]
- `tck.expressions.literals.literals5.scenario-8`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-9`: expected [["0.0"]], observed [["0"]]
- `tck.expressions.literals.literals5.scenario-10`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-11`: expected [["-1.2635418652381264e305"]], observed [["-126354186523812637083265300874972409152306344844529887549280826072773391668220596519840562718295208487895546677575070099382084697423587658029812212044105366779846072573635712245818261999842901642900582100199277717544304856592188574754249157767617334086173213808419907747092679556434770033100573931636523008.0"]]
- `tck.expressions.literals.literals5.scenario-12`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-13`: expected [["1000000000.0"]], observed [["1000000000"]]
- `tck.expressions.literals.literals5.scenario-14`: expected [["1000000000.0"]], observed [["1000000000"]]
- `tck.expressions.literals.literals5.scenario-15`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-17`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-18`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-19`: expected [["-1000000000.0"]], observed [["-1000000000"]]
- `tck.expressions.literals.literals5.scenario-20`: expected [["-1000000000.0"]], observed [["-1000000000"]]
- `tck.expressions.literals.literals5.scenario-21`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-23`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-24`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals5.scenario-25`: expected [["1e308"]], observed [["100000000000000001097906362944045541740492309677311846336810682903157585404911491537163328978494688899061249669721172515611590283743140088328307009198146046031271664502933027185697489699588559043338384466165001178426897626212945177628091195786707458122783970171784415105291802893207873272974885715430223118336.0"]]
- `tck.expressions.literals.literals5.scenario-26`: expected [["1.23456789e308"]], observed [["123456789000000004810070270463755942267619224180247500956396206068102412092226236396443696899102353171304143817789487919123992323950091504541492906785860605957745633924722561053445871433980835192763796487815950728557880041793319526330986197397604709979539461737602865391693914881194183408595281924019779010560.0"]]
- `tck.expressions.literals.literals6.scenario-5`: expected [["a\\\\bcn5t'\"\\\\//\\\\\"'"]], observed [["a\\bcn5t'\"\\//\\\"'"]]
- `tck.expressions.literals.literals6.scenario-9`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals6.scenario-11`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals6.scenario-12`: expected DISTINCT or projection_items at byte 7..7
- `tck.expressions.literals.literals7.scenario-7`: expected identifier or not_expression at byte 8..8
- `tck.expressions.literals.literals7.scenario-16`: expected [["[0.2,\", as#?lßdj \",null,71034856,false]"]], observed [["[0.2,\", as#?lßdj \",null,71034856,0]"]]
- `tck.expressions.literals.literals7.scenario-18`: expected [["[{id:\"0001\",type:\"donut\",name:\"Cake\",ppu:0.55,batters:{batter:[{id:\"1001\",type:\"Regular\"},{id:\"1002\",type:\"Chocolate\"},{id:\"1003\",type:\"Blueberry\"},{id:\"1004\",type:\"Devils Food\"}]},topping:[{id:\"5001\",type:\"None\"},{id:\"5002\",type:\"Glazed\"},{id:\"5005\",type:\"Sugar\"},{id:\"5007\",type:\"Powdered Sugar\"},{id:\"5006\",type:\"Chocolate Sprinkles\"},{id:\"5003\",type:\"Chocolate\"},{id:\"5004\",type:\"Maple\"}]},{id:\"0002\",type:\"donut\",name:\"Raised\",ppu:0.55,batters:{batter:[{id:\"1001\",type:\"Regular\"}]},topping:[{id:\"5001\",type:\"None\"},{id:\"5002\",type:\"Glazed\"},{id:\"5005\",type:\"Sugar\"},{id:\"5003\",type:\"Chocolate\"},{id:\"5004\",type:\"Maple\"}]},{id:\"0003\",type:\"donut\",name:\"Old Fashioned\",ppu:0.55,batters:{batter:[{id:\"1001\",type:\"Regular\"},{id:\"1002\",type:\"Chocolate\"}]},topping:[{id:\"5001\",type:\"None\"},{id:\"5002\",type:\"Glazed\"},{id:\"5003\",type:\"Chocolate\"},{id:\"5004\",type:\"Maple\"}]}]"]], observed [["[{\"id\":\"0001\",\"type\":\"donut\",\"name\":\"Cake\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"},{\"id\":\"1003\",\"type\":\"Blueberry\"},{\"id\":\"1004\",\"type\":\"Devils Food\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5007\",\"type\":\"Powdered Sugar\"},{\"id\":\"5006\",\"type\":\"Chocolate Sprinkles\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0002\",\"type\":\"donut\",\"name\":\"Raised\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0003\",\"type\":\"donut\",\"name\":\"Old Fashioned\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]}]"]]
- `tck.expressions.literals.literals8.scenario-2`: expected [["{abc: 1}"]], observed [["{\"abc\":1}"]]
- `tck.expressions.literals.literals8.scenario-3`: expected [["{ABC: 1}"]], observed [["{\"ABC\":1}"]]
- `tck.expressions.literals.literals8.scenario-4`: expected [["{aBCdeF: 1}"]], observed [["{\"aBCdeF\":1}"]]
- `tck.expressions.literals.literals8.scenario-5`: expected [["{a1B2c3e67: 1}"]], observed [["{\"a1B2c3e67\":1}"]]
- `tck.expressions.literals.literals8.scenario-6`: expected [["{k: 0}"]], observed [["{\"k\":0}"]]
- `tck.expressions.literals.literals8.scenario-7`: expected [["{k: null}"]], observed [["{\"k\":null}"]]
- `tck.expressions.literals.literals8.scenario-8`: expected [["{k: 1}"]], observed [["{\"k\":1}"]]
- `tck.expressions.literals.literals8.scenario-9`: expected [["{F: -372036854}"]], observed [["{\"F\":-372036854}"]]
- `tck.expressions.literals.literals8.scenario-10`: expected [["{k: 372036854}"]], observed [["{\"k\":372036854}"]]
- `tck.expressions.literals.literals8.scenario-11`: expected not_expression at byte 11..11
- `tck.expressions.literals.literals8.scenario-12`: expected [["{k: 'ab: c, as#?lßdj '}"]], observed [["{\"k\":\"ab: c, as#?lßdj \"}"]]
- `tck.expressions.literals.literals8.scenario-13`: expected [["{a: {}}"]], observed [["{\"a\":{}}"]]
- `tck.expressions.literals.literals8.scenario-14`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-15`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {a7: {a8: {a9: {a10: {a11: {a12: {a13: {a14: {a15: {a16: {a17: {a18: {a19: {}}}}}}}}}}}}}}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{\"a7\":{\"a8\":{\"a9\":{\"a10\":{\"a11\":{\"a12\":{\"a13\":{\"a14\":{\"a15\":{\"a16\":{\"a17\":{\"a18\":{\"a19\":{}}}}}}}}}}}}}}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-16`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {a7: {a8: {a9: {a10: {a11: {a12: {a13: {a14: {a15: {a16: {a17: {a18: {a19: {a20: {a21: {a22: {a23: {a24: {a25: {a26: {a27: {a28: {a29: {a30: {a31: {a32: {a33: {a34: {a35: {a36: {a37: {a38: {a39: {}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{\"a7\":{\"a8\":{\"a9\":{\"a10\":{\"a11\":{\"a12\":{\"a13\":{\"a14\":{\"a15\":{\"a16\":{\"a17\":{\"a18\":{\"a19\":{\"a20\":{\"a21\":{\"a22\":{\"a23\":{\"a24\":{\"a25\":{\"a26\":{\"a27\":{\"a28\":{\"a29\":{\"a30\":{\"a31\":{\"a32\":{\"a33\":{\"a34\":{\"a35\":{\"a36\":{\"a37\":{\"a38\":{\"a39\":{}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-17`: expected [["{a: ' { b : ', c: {d: ' '}, d: ' } '}"]], observed [["{\"a\":\" { b : \",\"c\":{\"d\":\" \"},\"d\":\" } \"}"]]
- `tck.expressions.literals.literals8.scenario-18`: expected [["{data: [{batters: {batter: [{id: '1001', type: 'Regular'}, {id: '1002', type: 'Chocolate'}, {id: '1003', type: 'Blueberry'}, {id: '1004', type: 'Devils Food'}]}, id: '0001', name: 'Cake', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5005', type: 'Sugar'}, {id: '5007', type: 'Powdered Sugar'}, {id: '5006', type: 'Chocolate Sprinkles'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}, {batters: {batter: [ {id: '1001', type: 'Regular'}]}, id: '0002', name: 'Raised', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5005', type: 'Sugar'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}, {batters: {batter: [{id: '1001', type: 'Regular'}, {id: '1002', type: 'Chocolate'}]}, id: '0003', name: 'Old Fashioned', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}]}"]], observed [["{\"data\":[{\"id\":\"0001\",\"type\":\"donut\",\"name\":\"Cake\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"},{\"id\":\"1003\",\"type\":\"Blueberry\"},{\"id\":\"1004\",\"type\":\"Devils Food\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5007\",\"type\":\"Powdered Sugar\"},{\"id\":\"5006\",\"type\":\"Chocolate Sprinkles\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0002\",\"type\":\"donut\",\"name\":\"Raised\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0003\",\"type\":\"donut\",\"name\":\"Old Fashioned\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]}]}"]]
- `tck.expressions.map.map1.scenario-1`: query execution failed: Parse error: property access requires a node or relationship at byte 51..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..87; query:
WITH {existing: 42, notMissing: null} AS m
RETURN m.missing, m.notMissing, m.existing
- `tck.expressions.map.map1.scenario-2`: query execution failed: Parse error: property access requires a node or relationship at byte 23..24; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..33; query:
WITH null AS m
RETURN m.missing
- `tck.expressions.map.map1.scenario-3`: query execution failed: Parse error: property access requires a node or relationship at byte 62..69; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 54..121; query:
WITH [123, {existing: 42, notMissing: null}] AS list
RETURN (list[1]).missing, (list[1]).notMissing, (list[1]).existing
- `tck.expressions.map.map1.scenario-4.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..70; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map.name AS result
- `tck.expressions.map.map1.scenario-4.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..70; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map.name AS result
- `tck.expressions.map.map1.scenario-4.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..70; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map.Name AS result
- `tck.expressions.map.map1.scenario-4.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..70; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map.nAMe AS result
- `tck.expressions.map.map1.scenario-5.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..72; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map.`name` AS result
- `tck.expressions.map.map1.scenario-5.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..72; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map.`nome` AS result
- `tck.expressions.map.map1.scenario-5.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..72; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map.`Mats` AS result
- `tck.expressions.map.map1.scenario-5.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..72; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map.`null` AS result
- `tck.expressions.map.map1.scenario-5.examples-1-row-5`: expected identifier at byte 6..6
- `tck.expressions.map.map1.scenario-5.examples-1-row-6`: expected identifier at byte 6..6
- `tck.expressions.map.map2.scenario-1`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.map.map2.scenario-2`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.map.map2.scenario-4`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 49..53; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..68; query:
WITH {name: 'Mats'} AS expr, null AS idx
RETURN expr[idx] AS value
- `tck.expressions.map.map2.scenario-5.examples-1-row-1`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..73; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map['name'] AS result
- `tck.expressions.map.map2.scenario-5.examples-1-row-2`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..73; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map['name'] AS result
- `tck.expressions.map.map2.scenario-5.examples-1-row-3`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..73; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map['Name'] AS result
- `tck.expressions.map.map2.scenario-5.examples-1-row-4`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..73; query:
WITH {name: 'Mats', Name: 'Pontus'} AS map
RETURN map['nAMe'] AS result
- `tck.expressions.map.map2.scenario-5.examples-1-row-5`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 51..54; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..73; query:
WITH {name: 'Mats', nome: 'Pontus'} AS map
RETURN map['null'] AS result
- `tck.expressions.map.map2.scenario-5.examples-1-row-6`: expected identifier at byte 6..6
- `tck.expressions.map.map2.scenario-5.examples-1-row-7`: expected identifier at byte 6..6
- `tck.expressions.map.map2.scenario-6`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.map.map2.scenario-7`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.map.map3.scenario-2`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.map.map3.scenario-3`: expected [["<null>", "<null>"]], observed [["[null]", "[null]"]]
- `tck.expressions.map.map3.scenario-5`: expected [["1", "1", "0"]], observed [["0", "0", "0"]]
- `tck.expressions.null.null1.scenario-3`: expected [["1"]], observed []
- `tck.expressions.null.null1.scenario-5.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..79; query:
WITH {name: 'Mats', name2: 'Pontus'} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..80; query:
WITH {name: 'Mats', name2: 'Pontus'} AS map
RETURN map.name2 IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 48..51; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..75; query:
WITH {name: 'Mats', name2: null} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 48..51; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..76; query:
WITH {name: 'Mats', name2: null} AS map
RETURN map.name2 IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 33..36; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 26..60; query:
WITH {name: null} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-6`: query execution failed: Parse error: property access requires a node or relationship at byte 46..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..73; query:
WITH {name: null, name2: null} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-7`: query execution failed: Parse error: property access requires a node or relationship at byte 46..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..74; query:
WITH {name: null, name2: null} AS map
RETURN map.name2 IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-8`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..79; query:
WITH {notName: null, notName2: null} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-9`: query execution failed: Parse error: property access requires a node or relationship at byte 49..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..76; query:
WITH {notName: 0, notName2: null} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-10`: query execution failed: Parse error: property access requires a node or relationship at byte 33..36; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 26..60; query:
WITH {notName: 0} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-11`: query execution failed: Parse error: property access requires a node or relationship at byte 23..26; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..50; query:
WITH {} AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null1.scenario-5.examples-1-row-12`: query execution failed: Parse error: property access requires a node or relationship at byte 25..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..52; query:
WITH null AS map
RETURN map.name IS NULL AS result
- `tck.expressions.null.null2.scenario-3`: expected [["0"]], observed []
- `tck.expressions.null.null2.scenario-5.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..83; query:
WITH {name: 'Mats', name2: 'Pontus'} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..84; query:
WITH {name: 'Mats', name2: 'Pontus'} AS map
RETURN map.name2 IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 48..51; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..79; query:
WITH {name: 'Mats', name2: null} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 48..51; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..80; query:
WITH {name: 'Mats', name2: null} AS map
RETURN map.name2 IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 33..36; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 26..64; query:
WITH {name: null} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-6`: query execution failed: Parse error: property access requires a node or relationship at byte 46..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..77; query:
WITH {name: null, name2: null} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-7`: query execution failed: Parse error: property access requires a node or relationship at byte 46..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..78; query:
WITH {name: null, name2: null} AS map
RETURN map.name2 IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-8`: query execution failed: Parse error: property access requires a node or relationship at byte 52..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..83; query:
WITH {notName: null, notName2: null} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-9`: query execution failed: Parse error: property access requires a node or relationship at byte 49..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..80; query:
WITH {notName: 0, notName2: null} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-10`: query execution failed: Parse error: property access requires a node or relationship at byte 33..36; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 26..64; query:
WITH {notName: 0} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-11`: query execution failed: Parse error: property access requires a node or relationship at byte 23..26; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..54; query:
WITH {} AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null2.scenario-5.examples-1-row-12`: query execution failed: Parse error: property access requires a node or relationship at byte 25..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..56; query:
WITH null AS map
RETURN map.name IS NOT NULL AS result
- `tck.expressions.null.null3.scenario-4.examples-1-row-2`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.null.null3.scenario-4.examples-1-row-3`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.null.null3.scenario-4.examples-1-row-4`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.null.null3.scenario-4.examples-1-row-5`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.null.null3.scenario-4.examples-1-row-6`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.null.null3.scenario-4.examples-1-row-7`: TCK parameter value is not representable by the generic adapter
- `tck.expressions.path.path1.scenario-1`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 36..37; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 36..37; query:
WITH null AS a
OPTIONAL MATCH p = (a)-[r]->()
RETURN nodes(p), nodes(null)
- `tck.expressions.path.path2.scenario-1`: expected [["[[:REL {num: 1}], [:REL {num: 2}]]"]], observed [["[1, 2]"]]
- `tck.expressions.path.path2.scenario-2`: expected [["[[:REL {num: 1}], [:REL {num: 2}]]"]], observed [["[1, 2]"]]
- `tck.expressions.path.path2.scenario-3`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 36..37; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 36..37; query:
WITH null AS a
OPTIONAL MATCH p = (a)-[r]->()
RETURN relationships(p), relationships(null)
- `tck.expressions.path.path3.scenario-2`: expected an error but execution succeeded
- `tck.expressions.path.path3.scenario-3`: expected an error but execution succeeded
- `tck.expressions.pattern.pattern1.scenario-5`: expected [["(:A)"], ["(:B)"], ["(:D)"]], observed [["(:A)"], ["(:B)"], ["(:C)"], ["(:D)"]]
- `tck.expressions.pattern.pattern1.scenario-7`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..43; query:
MATCH (n) WHERE (n)-[:REL1*]->() RETURN n
- `tck.expressions.pattern.pattern1.scenario-8`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..42; query:
MATCH (n) WHERE (n)-[:REL1*]-() RETURN n
- `tck.expressions.pattern.pattern1.scenario-9`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..43; query:
MATCH (n) WHERE (n)<-[:REL1*]-() RETURN n
- `tck.expressions.pattern.pattern1.scenario-10`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..43; query:
MATCH (n) WHERE (n)-[:REL1*2]-() RETURN n
- `tck.expressions.pattern.pattern1.scenario-11`: expected an error but execution succeeded
- `tck.expressions.pattern.pattern1.scenario-13`: expected [["(:A)", "(:B)"], ["(:A)", "(:C)"], ["(:A)", "(:D)"], ["(:B)", "(:A)"], ["(:C)", "(:A)"], ["(:D)", "(:A)"]], observed [["(:A)", "(:B)"], ["(:A)", "(:C)"], ["(:A)", "(:D)"], ["(:B)", "(:A)"], ["(:C)", "(:A)"], ["(:C)", "(:B)"], ["(:D)", "(:A)"], ["(:D)", "(:B)"]]
- `tck.expressions.pattern.pattern1.scenario-15`: expected [["(:A)", "(:B)"], ["(:A)", "(:D)"], ["(:B)", "(:A)"], ["(:D)", "(:A)"]], observed [["(:A)", "(:B)"], ["(:A)", "(:D)"], ["(:B)", "(:A)"], ["(:C)", "(:A)"], ["(:D)", "(:A)"]]
- `tck.expressions.pattern.pattern1.scenario-16`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 40..52; query:
MATCH (n), (m) WHERE (n)-[:REL1*]->(m) RETURN n, m
- `tck.expressions.pattern.pattern1.scenario-17`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..51; query:
MATCH (n), (m) WHERE (n)-[:REL1*]-(m) RETURN n, m
- `tck.expressions.pattern.pattern1.scenario-18`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 40..52; query:
MATCH (n), (m) WHERE (n)-[:REL1*2]-(m) RETURN n, m
- `tck.expressions.pattern.pattern1.scenario-20`: expected [["(:A)"]], observed [["(:A)"], ["(:B)"], ["(:C)"], ["(:D)"]]
- `tck.expressions.pattern.pattern2.scenario-1`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 31..31
- `tck.expressions.pattern.pattern2.scenario-2`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 35..35
- `tck.expressions.pattern.pattern2.scenario-3`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 41..41
- `tck.expressions.pattern.pattern2.scenario-4`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 32..32
- `tck.expressions.pattern.pattern2.scenario-5`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 32..32
- `tck.expressions.pattern.pattern2.scenario-6`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 45..45
- `tck.expressions.pattern.pattern2.scenario-7`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 65..65
- `tck.expressions.pattern.pattern2.scenario-8`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 35..35
- `tck.expressions.pattern.pattern2.scenario-9`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 42..42
- `tck.expressions.pattern.pattern2.scenario-10`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 39..39
- `tck.expressions.pattern.pattern2.scenario-11`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, or postfix_suffix at byte 38..38
- `tck.expressions.precedence.precedence1.scenario-1`: expected [["1", "1", "0"]], observed [["0", "1", "0"]]
- `tck.expressions.precedence.precedence1.scenario-14`: expected [["1"]], observed [["0"]]
- `tck.expressions.precedence.precedence2.scenario-4`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence2.scenario-5.examples-1-row-1`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence2.scenario-5.examples-1-row-2`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence3.scenario-1`: expected [["[[1], [2, 3], [4, 5], 10]", "[[1], [2, 3], [4, 5], 10]", "5"]], observed [["10", "10", "<null>"]]
- `tck.expressions.precedence.precedence3.scenario-2`: expected [["[[1], [2, 3], [4, 5], 8, 9]", "[[1], [2, 3], [4, 5], 8, 9]", "[4, 5]"]], observed [["0", "0", "<null>"]]
- `tck.expressions.precedence.precedence3.scenario-3`: expected [["[[1], [2, 3], [4, 5], [6, 7], [8, 9]]", "[[1], [2, 3], [4, 5], [6, 7], [8, 9]]", "[[2, 3], [4, 5]]"]], observed [["0", "0", "[]"]]
- `tck.expressions.precedence.precedence3.scenario-4`: expected [["0", "0", "[1, 0, 4]"]], observed [["0", "0", "4"]]
- `tck.expressions.precedence.precedence3.scenario-5`: expected [["0", "0", "[0, 4]", "[1, 0, 4]"]], observed [["1", "1", "0", "0"]]
- `tck.expressions.precedence.precedence3.scenario-6.examples-1-row-3`: expected [["<null>", "<null>", "0"]], observed [["0", "0", "0"]]
- `tck.expressions.precedence.precedence3.scenario-6.examples-1-row-4`: expected [["<null>", "<null>", "1"]], observed [["1", "1", "1"]]
- `tck.expressions.precedence.precedence3.scenario-6.examples-1-row-5`: expected [["<null>", "<null>", "0"]], observed [["0", "0", "0"]]
- `tck.expressions.precedence.precedence3.scenario-6.examples-1-row-6`: expected [["<null>", "<null>", "1"]], observed [["1", "1", "1"]]
- `tck.expressions.precedence.precedence4.scenario-4`: query execution failed: Parse error: string predicates on non-string operands is not supported in the initial graph slice at byte 146..160; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..334; query:
RETURN ('abc' STARTS WITH null OR true) = (('abc' STARTS WITH null) OR true) AS a,
       ('abc' STARTS WITH null OR true) <> ('abc' STARTS WITH (null OR true)) AS b,
       (true OR null STARTS WITH 'abc') = (true OR (null STARTS WITH 'abc')) AS c,
       (true OR null STARTS WITH 'abc') <> ((true OR null) STARTS WITH 'abc') AS d
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-1`: query execution failed: Parse error: unknown variable `x` at byte 27..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..46; query:
RETURN none(x IN [] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-2`: query execution failed: Parse error: unknown variable `x` at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..58; query:
RETURN none(x IN [{a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-3`: query execution failed: Parse error: unknown variable `x` at byte 33..34; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..52; query:
RETURN none(x IN [{a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-4`: query execution failed: Parse error: unknown variable `x` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..66; query:
RETURN none(x IN [{a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-5`: query execution failed: Parse error: unknown variable `x` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..66; query:
RETURN none(x IN [{a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-6`: query execution failed: Parse error: unknown variable `x` at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..80; query:
RETURN none(x IN [{a: 2, b: 5}, {a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-7`: query execution failed: Parse error: unknown variable `x` at byte 55..56; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..74; query:
RETURN none(x IN [{a: 4}, {a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-8`: query execution failed: Parse error: unknown variable `x` at byte 67..68; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..86; query:
RETURN none(x IN [{a: 2, b: 5}, {a: 2, b: 5}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-7.examples-1-row-9`: query execution failed: Parse error: unknown variable `x` at byte 49..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..68; query:
RETURN none(x IN [{a: 4}, {a: 4}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier1.scenario-8`: query execution failed: Parse error: unknown variable `x` at byte 99..100; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..123; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, none(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier1.scenario-9`: query execution failed: Parse error: unknown variable `x` at byte 154..155; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..178; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, none(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier1.scenario-10.examples-1-row-1`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier1.scenario-10.examples-1-row-2`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier1.scenario-10.examples-1-row-3`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier1.scenario-10.examples-1-row-7`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier10.scenario-2`: expected [["0"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier10.scenario-4.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH single(x IN list WHERE x = 2) = (size([x IN list WHERE x = 2 | x]) = 1) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier10.scenario-4.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH single(x IN list WHERE x % 2 = 0) = (size([x IN list WHERE x % 2 = 0 | x]) = 1) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier10.scenario-4.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH single(x IN list WHERE x % 3 = 0) = (size([x IN list WHERE x % 3 = 0 | x]) = 1) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier10.scenario-4.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH single(x IN list WHERE x < 7) = (size([x IN list WHERE x < 7 | x]) = 1) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier10.scenario-4.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH single(x IN list WHERE x >= 3) = (size([x IN list WHERE x >= 3 | x]) = 1) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-3.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 152..157; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 152..157; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH list WHERE single(x IN list WHERE x = 2) OR all(x IN list WHERE x = 2)
WITH any(x IN list WHERE x = 2) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-3.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 152..157; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 152..157; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH list WHERE single(x IN list WHERE x % 2 = 0) OR all(x IN list WHERE x % 2 = 0)
WITH any(x IN list WHERE x % 2 = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-3.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 152..157; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 152..157; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH list WHERE single(x IN list WHERE x % 3 = 0) OR all(x IN list WHERE x % 3 = 0)
WITH any(x IN list WHERE x % 3 = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-3.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 152..157; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 152..157; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH list WHERE single(x IN list WHERE x < 7) OR all(x IN list WHERE x < 7)
WITH any(x IN list WHERE x < 7) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-3.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 152..157; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 152..157; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH list WHERE single(x IN list WHERE x >= 3) OR all(x IN list WHERE x >= 3)
WITH any(x IN list WHERE x >= 3) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-6.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH any(x IN list WHERE x = 2) = (size([x IN list WHERE x = 2 | x]) > 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-6.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH any(x IN list WHERE x % 2 = 0) = (size([x IN list WHERE x % 2 = 0 | x]) > 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-6.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH any(x IN list WHERE x % 3 = 0) = (size([x IN list WHERE x % 3 = 0 | x]) > 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-6.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH any(x IN list WHERE x < 7) = (size([x IN list WHERE x < 7 | x]) > 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier11.scenario-6.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH any(x IN list WHERE x >= 3) = (size([x IN list WHERE x >= 3 | x]) > 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier12.scenario-5.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH all(x IN list WHERE x = 2) = (size([x IN list WHERE x = 2 | x]) = size(list)) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier12.scenario-5.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH all(x IN list WHERE x % 2 = 0) = (size([x IN list WHERE x % 2 = 0 | x]) = size(list)) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier12.scenario-5.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH all(x IN list WHERE x % 3 = 0) = (size([x IN list WHERE x % 3 = 0 | x]) = size(list)) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier12.scenario-5.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH all(x IN list WHERE x < 7) = (size([x IN list WHERE x < 7 | x]) = size(list)) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier12.scenario-5.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH all(x IN list WHERE x >= 3) = (size([x IN list WHERE x >= 3 | x]) = size(list)) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-1`: query execution failed: Parse error: unknown variable `x` at byte 29..30; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..48; query:
RETURN single(x IN [] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-2`: query execution failed: Parse error: unknown variable `x` at byte 41..42; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..60; query:
RETURN single(x IN [{a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-3`: query execution failed: Parse error: unknown variable `x` at byte 35..36; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..54; query:
RETURN single(x IN [{a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-4`: query execution failed: Parse error: unknown variable `x` at byte 49..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..68; query:
RETURN single(x IN [{a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-5`: query execution failed: Parse error: unknown variable `x` at byte 49..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..68; query:
RETURN single(x IN [{a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-6`: query execution failed: Parse error: unknown variable `x` at byte 63..64; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..82; query:
RETURN single(x IN [{a: 2, b: 5}, {a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-7`: query execution failed: Parse error: unknown variable `x` at byte 57..58; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..76; query:
RETURN single(x IN [{a: 4}, {a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-8`: query execution failed: Parse error: unknown variable `x` at byte 69..70; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..88; query:
RETURN single(x IN [{a: 2, b: 5}, {a: 2, b: 5}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-7.examples-1-row-9`: query execution failed: Parse error: unknown variable `x` at byte 51..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..70; query:
RETURN single(x IN [{a: 4}, {a: 4}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier2.scenario-8`: query execution failed: Parse error: unknown variable `x` at byte 101..102; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..125; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, single(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier2.scenario-9`: query execution failed: Parse error: unknown variable `x` at byte 156..157; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..180; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, single(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-1`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-2`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-3`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-4`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-5`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier2.scenario-10.examples-1-row-7`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-1`: query execution failed: Parse error: unknown variable `x` at byte 26..27; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..45; query:
RETURN any(x IN [] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-2`: query execution failed: Parse error: unknown variable `x` at byte 38..39; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..57; query:
RETURN any(x IN [{a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-3`: query execution failed: Parse error: unknown variable `x` at byte 32..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..51; query:
RETURN any(x IN [{a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-4`: query execution failed: Parse error: unknown variable `x` at byte 46..47; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..65; query:
RETURN any(x IN [{a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-5`: query execution failed: Parse error: unknown variable `x` at byte 46..47; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..65; query:
RETURN any(x IN [{a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-6`: query execution failed: Parse error: unknown variable `x` at byte 60..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..79; query:
RETURN any(x IN [{a: 2, b: 5}, {a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-7`: query execution failed: Parse error: unknown variable `x` at byte 54..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..73; query:
RETURN any(x IN [{a: 4}, {a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-8`: query execution failed: Parse error: unknown variable `x` at byte 66..67; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..85; query:
RETURN any(x IN [{a: 2, b: 5}, {a: 2, b: 5}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-7.examples-1-row-9`: query execution failed: Parse error: unknown variable `x` at byte 48..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..67; query:
RETURN any(x IN [{a: 4}, {a: 4}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier3.scenario-8`: query execution failed: Parse error: unknown variable `x` at byte 98..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..122; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, any(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier3.scenario-9`: query execution failed: Parse error: unknown variable `x` at byte 153..154; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..177; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, any(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier3.scenario-10.examples-1-row-1`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier3.scenario-10.examples-1-row-2`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier3.scenario-10.examples-1-row-3`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier3.scenario-10.examples-1-row-7`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-1`: query execution failed: Parse error: unknown variable `x` at byte 26..27; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..45; query:
RETURN all(x IN [] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-2`: query execution failed: Parse error: unknown variable `x` at byte 38..39; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..57; query:
RETURN all(x IN [{a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-3`: query execution failed: Parse error: unknown variable `x` at byte 32..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..51; query:
RETURN all(x IN [{a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-4`: query execution failed: Parse error: unknown variable `x` at byte 46..47; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..65; query:
RETURN all(x IN [{a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-5`: query execution failed: Parse error: unknown variable `x` at byte 46..47; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..65; query:
RETURN all(x IN [{a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-6`: query execution failed: Parse error: unknown variable `x` at byte 60..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..79; query:
RETURN all(x IN [{a: 2, b: 5}, {a: 4}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-7`: query execution failed: Parse error: unknown variable `x` at byte 54..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..73; query:
RETURN all(x IN [{a: 4}, {a: 2, b: 5}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-8`: query execution failed: Parse error: unknown variable `x` at byte 66..67; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..85; query:
RETURN all(x IN [{a: 2, b: 5}, {a: 2, b: 5}, {a: 2, b: 5}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-7.examples-1-row-9`: query execution failed: Parse error: unknown variable `x` at byte 48..49; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..67; query:
RETURN all(x IN [{a: 4}, {a: 4}, {a: 4}] WHERE x.a = 2) AS result
- `tck.expressions.quantifier.quantifier4.scenario-8`: query execution failed: Parse error: unknown variable `x` at byte 98..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..122; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, all(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier4.scenario-9`: query execution failed: Parse error: unknown variable `x` at byte 153..154; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..177; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, all(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier4.scenario-10.examples-1-row-1`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier4.scenario-10.examples-1-row-2`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier4.scenario-10.examples-1-row-4`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier4.scenario-10.examples-1-row-5`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier4.scenario-10.examples-1-row-8`: expected [["<null>"]], observed [["1"]]
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-1`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 91..93; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..110; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE none(y IN list WHERE x <= y)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-2`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 91..93; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..109; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE none(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-3`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 97..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..120; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE single(y IN list WHERE abs(x - y) < 3)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-4`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 93..95; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..116; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE single(y IN list WHERE x + y = 15)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-5`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..112; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE any(y IN list WHERE x + y < 2)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-6`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..113; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE any(y IN list WHERE x + y <= 3)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-7`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..108; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE all(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier5.scenario-2.examples-1-row-8`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..109; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN none(x IN list WHERE all(y IN list WHERE x <= y)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-1`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 93..95; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..111; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE none(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-2`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 93..95; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..115; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE none(y IN list WHERE x % y = 0)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-3`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 95..97; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..117; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE single(y IN list WHERE x + y < 5)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-4`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 95..97; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..117; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE single(y IN list WHERE x % y = 1)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-5`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 96..98; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..119; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE any(y IN list WHERE 2 * x + y > 25)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-6`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..110; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE any(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-7`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..111; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE all(y IN list WHERE x <= y)) AS result
- `tck.expressions.quantifier.quantifier6.scenario-2.examples-1-row-8`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..115; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN single(x IN list WHERE all(y IN list WHERE x <= y + 1)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-1`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..112; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE none(y IN list WHERE x = y * y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-2`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..112; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE none(y IN list WHERE x % y = 0)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-3`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..114; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE single(y IN list WHERE x = y * y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-4`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..114; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE single(y IN list WHERE x < y * y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-5`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..107; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE any(y IN list WHERE x = y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-6`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..112; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE any(y IN list WHERE x = 10 * y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-7`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..108; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE all(y IN list WHERE x <= y)) AS result
- `tck.expressions.quantifier.quantifier7.scenario-2.examples-1-row-8`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..107; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN any(x IN list WHERE all(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-1`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..113; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE none(y IN list WHERE x = 10 * y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-2`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 90..92; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..108; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE none(y IN list WHERE x = y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-3`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..110; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE single(y IN list WHERE x = y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-4`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 92..94; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..110; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE single(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-5`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..111; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE any(y IN list WHERE x % y = 0)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-6`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..107; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE any(y IN list WHERE x < y)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-7`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 93..95; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..117; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE all(y IN list WHERE abs(x - y) < 10)) AS result
- `tck.expressions.quantifier.quantifier8.scenario-2.examples-1-row-8`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 89..91; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..111; query:
WITH [1, 2, 3, 4, 5, 6, 7, 8, 9] AS list
RETURN all(x IN list WHERE all(y IN list WHERE x < y + 7)) AS result
- `tck.expressions.quantifier.quantifier9.scenario-5.examples-1-row-1`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH none(x IN list WHERE x = 2) = (size([x IN list WHERE x = 2 | x]) = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier9.scenario-5.examples-1-row-2`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH none(x IN list WHERE x % 2 = 0) = (size([x IN list WHERE x % 2 = 0 | x]) = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier9.scenario-5.examples-1-row-3`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH none(x IN list WHERE x % 3 = 0) = (size([x IN list WHERE x % 3 = 0 | x]) = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier9.scenario-5.examples-1-row-4`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH none(x IN list WHERE x < 7) = (size([x IN list WHERE x < 7 | x]) = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.quantifier.quantifier9.scenario-5.examples-1-row-5`: query execution failed: Parse error: property access requires a node or relationship at byte 186..191; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 186..191; query:
UNWIND [{list: [2], fixed: true},
        {list: [6], fixed: true},
        {list: [7], fixed: true},
        {list: [1, 2, 3, 4, 5, 6, 7, 8, 9], fixed: false}] AS input
WITH CASE WHEN input.fixed THEN input.list ELSE null END AS fixedList,
     CASE WHEN NOT input.fixed THEN input.list ELSE [1] END AS inputList
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
UNWIND inputList AS x
WITH fixedList, inputList, x, [ y IN inputList WHERE rand() > 0.5 | y] AS list
WITH fixedList, inputList, CASE WHEN rand() < 0.5 THEN reverse(list) ELSE list END + x AS list
WITH coalesce(fixedList, list) AS list
WITH none(x IN list WHERE x >= 3) = (size([x IN list WHERE x >= 3 | x]) = 0) AS result, count(*) AS cnt
RETURN result
- `tck.expressions.string.string1.scenario-1`: expected [["123456789"]], observed [["0123456789"]]
- `tck.expressions.string.string10.scenario-8`: expected [["<null>", "36"]], observed [["0", "16"], ["1", "9"], ["<null>", "11"]]
- `tck.expressions.string.string4.scenario-1`: query execution failed: Parse error: no such function: split; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 38..65; query:
UNWIND split('one1two', '1') AS item
RETURN count(item) AS item
- `tck.expressions.string.string8.scenario-8`: expected [["<null>", "36"]], observed [["0", "18"], ["1", "7"], ["<null>", "11"]]
- `tck.expressions.string.string9.scenario-8`: expected [["<null>", "36"]], observed [["0", "18"], ["1", "7"], ["<null>", "11"]]
- `tck.expressions.temporal.temporal1.scenario-1.examples-1-row-13`: expected [["1817-01-08"]], observed [["1816-01-10"]]
- `tck.expressions.temporal.temporal1.scenario-1.examples-1-row-14`: expected [["1817-01-07"]], observed [["1816-01-08"]]
- `tck.expressions.temporal.temporal1.scenario-1.examples-1-row-15`: expected [["1817-01-07"]], observed [["1817-01-06"]]
- `tck.expressions.temporal.temporal1.scenario-2.examples-1-row-13`: expected [["1817-01-08T00:00"]], observed [["1816-01-10T00:00"]]
- `tck.expressions.temporal.temporal1.scenario-2.examples-1-row-14`: expected [["1817-01-07T00:00"]], observed [["1816-01-08T00:00"]]
- `tck.expressions.temporal.temporal1.scenario-2.examples-1-row-15`: expected [["1817-01-07T00:00"]], observed [["1817-01-06T00:00"]]
- `tck.expressions.temporal.temporal1.scenario-3.examples-1-row-13`: expected [["1817-01-08T00:00Z"]], observed [["1816-01-10T00:00Z"]]
- `tck.expressions.temporal.temporal1.scenario-3.examples-1-row-14`: expected [["1817-01-07T00:00Z"]], observed [["1816-01-08T00:00Z"]]
- `tck.expressions.temporal.temporal1.scenario-3.examples-1-row-15`: expected [["1817-01-07T00:00Z"]], observed [["1817-01-06T00:00Z"]]
- `tck.expressions.temporal.temporal1.scenario-11`: query execution failed: Parse error: invalid resolved function or parameter name: datetime.fromepoch; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..105; query:
RETURN datetime.fromepoch(416779, 999999999) AS d1,
       datetime.fromepochmillis(237821673987) AS d2
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-2`: expected [["P5M1DT12H"]], observed [["P5M1D"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-3`: expected [["P22DT19H51M49.5S"]], observed [["PT0S"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-4`: expected [["P17DT12H"]], observed [["P17D"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-6`: expected [["P14DT1M10.001S"]], observed [["P14DT1M10.001000000S"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-7`: expected [["P14DT1M10.000001S"]], observed [["P14DT1M10.000001000S"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-1`: expected [["12:34:56+02:05"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-2`: expected [["12:34:56+02:05:59"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-3`: expected [["12:34:56-02:05:07"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-4`: expected [["1984-10-11T12:34:56+02:05:59"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-1.examples-1-row-3`: expected [["PT23H59M59.9S", "0", "86399", "900000000"]], observed [["PT23H59M59.900000000S", "0", "86399", "900000000"]]
- `tck.expressions.temporal.temporal10.scenario-1.examples-1-row-4`: expected [["PT-23H-59M-59.9S", "0", "-86400", "100000000"]], observed [["PT-24H0.100000000S", "0", "-86400", "100000000"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-2`: expected [["P31Y9M10DT21H45M22.142S"]], observed [["P31Y9M10DT21H45M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-3`: expected [["P30Y9M10DT21H40M32.142S"]], observed [["P30Y9M10DT21H40M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-4`: expected [["PT16H30M"]], observed [["P-14Y-9M-9DT-7H-30M"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-5`: expected [["PT16H30M"]], observed [["P-14Y-9M-9DT-7H-30M"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-6`: expected [["PT-14H-30M"]], observed [["P45Y5M22DT9H30M"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-7`: expected [["PT7H15M22.142S"]], observed [["P46Y6M20DT7H15M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-8`: expected [["PT7H10M32.142S"]], observed [["P45Y6M20DT7H10M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-11`: expected [["PT-14H-30M"]], observed [["P45Y5M22DT9H30M"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-12`: expected [["PT7H15M22.142S"]], observed [["P46Y6M20DT7H15M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-13`: expected [["PT6H10M32.142S"]], observed [["P45Y6M20DT7H10M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-15`: expected [["PT1H"]], observed [["PT2H"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-16`: expected [["P-27DT-21H-40M-32.142S"]], observed [["P-27DT-21H-40M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-19`: expected [["PT-5H-10M-32.142S"]], observed [["P-45Y-6M-20DT-5H-10M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-20`: expected [["PT-5H-10M-32.142S"]], observed [["P-45Y-6M-20DT-5H-10M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-21`: expected [["P11M2DT2H19M23.857S"]], observed [["P11M2DT2H19M23.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-22`: expected [["P2YT4M45.999S"]], observed [["P2YT4M45.999000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-23`: expected [["P1YT59M55.999S"]], observed [["P11M29DT23H59M55.999000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-24`: expected [["PT-5H-10M-36.143S"]], observed [["P-44Y-6M-20DT-5H-10M-37.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-25`: expected [["PT-4H-10M-36.143S"]], observed [["P-44Y-6M-20DT-5H-10M-37.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-4`: expected [["PT0S"]], observed [["P-14Y-9M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-5`: expected [["PT0S"]], observed [["P-14Y-9M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-6`: expected [["PT0S"]], observed [["P45Y5M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-7`: expected [["PT0S"]], observed [["P46Y6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-8`: expected [["PT0S"]], observed [["P45Y6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-9`: expected [["PT0S"]], observed [["P45Y5M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-10`: expected [["PT0S"]], observed [["P46Y6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-11`: expected [["PT0S"]], observed [["P45Y6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-15`: expected [["PT0S"]], observed [["P-45Y-6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-16`: expected [["PT0S"]], observed [["P-45Y-6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-19`: expected [["P1Y"]], observed [["P11M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-20`: expected [["PT0S"]], observed [["P-44Y-6M"]]
- `tck.expressions.temporal.temporal10.scenario-3.examples-1-row-21`: expected [["PT0S"]], observed [["P-44Y-6M"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-4`: expected [["PT0S"]], observed [["P-5396D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-5`: expected [["PT0S"]], observed [["P-5396D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-6`: expected [["PT0S"]], observed [["P16609D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-7`: expected [["PT0S"]], observed [["P17003D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-8`: expected [["PT0S"]], observed [["P16637D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-9`: expected [["PT0S"]], observed [["P16609D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-10`: expected [["PT0S"]], observed [["P17003D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-11`: expected [["PT0S"]], observed [["P16637D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-15`: expected [["PT0S"]], observed [["P-16637D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-16`: expected [["PT0S"]], observed [["P-16637D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-19`: expected [["P365D"]], observed [["P364D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-20`: expected [["PT0S"]], observed [["P-16272D"]]
- `tck.expressions.temporal.temporal10.scenario-4.examples-1-row-21`: expected [["PT0S"]], observed [["P-16272D"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-2`: expected [["PT278565H45M22.142S"]], observed [["PT278565H45M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-3`: expected [["PT269781H40M32.142S"]], observed [["PT269781H40M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-4`: expected [["PT16H30M"]], observed [["PT-129511H-30M"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-5`: expected [["PT16H30M"]], observed [["PT-129511H-30M"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-6`: expected [["PT-14H-30M"]], observed [["PT398625H30M"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-7`: expected [["PT7H15M22.142S"]], observed [["PT408079H15M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-8`: expected [["PT7H10M32.142S"]], observed [["PT399295H10M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-11`: expected [["PT-14H-30M"]], observed [["PT398625H30M"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-12`: expected [["PT7H15M22.142S"]], observed [["PT408079H15M22.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-13`: expected [["PT6H10M32.142S"]], observed [["PT399295H10M32.142000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-15`: expected [["PT1H"]], observed [["PT2H"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-16`: expected [["PT-669H-40M-32.142S"]], observed [["PT-669H-40M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-19`: expected [["PT-5H-10M-32.142S"]], observed [["PT-399293H-10M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-20`: expected [["PT-5H-10M-32.142S"]], observed [["PT-399293H-10M-33.858000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-21`: expected [["PT8090H19M23.857S"]], observed [["PT8090H19M23.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-22`: expected [["PT17544H4M45.999S"]], observed [["PT17544H4M45.999000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-23`: expected [["PT8760H59M55.999S"]], observed [["PT8759H59M55.999000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-24`: expected [["PT-5H-10M-36.143S"]], observed [["PT-390533H-10M-37.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-25`: expected [["PT-4H-10M-36.143S"]], observed [["PT-390533H-10M-37.857000000S"]]
- `tck.expressions.temporal.temporal10.scenario-6`: expected [["PT-0.001S"]], observed [["PT-1.999000000S"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-1`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-2`: expected [["PT5H"]], observed [["PT-419228H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-3`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-4`: expected [["PT5H"]], observed [["PT419236H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-5`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-6`: expected [["PT25H"]], observed [["PT24H"]]
- `tck.expressions.temporal.temporal10.scenario-9`: expected [["P1999999998Y11M30D"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-10`: expected [["PT17531639991215H59M59S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-1`: expected [["PT-0.4S"]], observed [["PT-1.600000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-2`: expected [["PT0.4S"]], observed [["PT0.400000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-3`: expected [["PT0.6S"]], observed [["PT0.600000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-4`: expected [["PT10M0.6S"]], observed [["PT10M0.600000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-5`: expected [["PT-9M-59.4S"]], observed [["PT-10M0.600000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-6`: expected [["PT-0.3S"]], observed [["PT-1.700000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-7`: expected [["PT9M59.7S"]], observed [["PT9M59.700000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-8`: expected [["PT-10M-0.3S"]], observed [["PT-10M-1.700000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-9`: expected [["PT-1.6S"]], observed [["PT-2.400000000S"]]
- `tck.expressions.temporal.temporal10.scenario-11.examples-1-row-10`: expected [["PT1.6S"]], observed [["PT1.600000000S"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-1`: expected [["PT0S"]], observed [["PT0.000006000S"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2`: expected [["PT0S"]], observed [["PT0.000007000S"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-4`: expected [["PT0S"]], observed [["PT0.000006000S"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-5`: expected [["PT0S"]], observed [["PT0.000006000S"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-2`: expected [["2015-07-21"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-3`: expected [["2015-07-01"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-4`: expected [["2015-07-01"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-5`: expected [["2015-07-21"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-6`: expected [["2015-07-21"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-7`: expected [["2015-07-20"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-8`: expected [["2015-07-20"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-9`: expected [["2015-07-21"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-10`: expected [["2015-07-21"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-1.examples-1-row-11`: expected [["2015-01-01"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-3.examples-1-row-1`: expected [["21:40:32.142+01:00"]], observed [["21:40:32.142Z"]]
- `tck.expressions.temporal.temporal2.scenario-3.examples-1-row-4`: expected [["21:40:32-01:00"]], observed [["21:40:32Z"]]
- `tck.expressions.temporal.temporal2.scenario-3.examples-1-row-7`: expected [["21:40-02:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-2`: expected [["2015-07-21T21:40:32.142"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-3`: expected [["2015-07-21T21:40:32"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-4`: expected [["2015-01-01T21:40:32"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-5`: expected [["2015-07-21T21:40"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-6`: expected [["2015-07-20T21:40"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-4.examples-1-row-7`: expected [["2015-07-21T21:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-1`: expected [["2015-07-21T21:40:32.142+01:00"]], observed [["2015-07-21T21:40:32.142Z"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-2`: expected [["2015-07-21T21:40:32.142Z"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-3`: expected [["2015-07-21T21:40:32+01:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-4`: expected [["2015-01-01T21:40:32-01:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-5`: expected [["2015-07-21T21:40-01:30"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-6`: expected [["2015-07-20T21:40Z"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-7`: expected [["2015-07-20T21:40-02:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-5.examples-1-row-8`: expected [["2015-07-21T21:00+18:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-6.examples-1-row-5`: expected [["1818-07-21T21:40:32.142+00:53:28[Europe/Stockholm]"]], observed [["1818-07-21T21:40:32.142+00:53[Europe/Stockholm]"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-2`: expected [["P5M1DT12H"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-3`: expected [["P22DT19H51M49.5S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-4`: expected [["PT45S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-5`: expected [["P17DT12H"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-7`: expected [["P2012Y2M2DT14H37M21.545S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-5`: expected [["1984-01-08"]], observed [["1984-01-02"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-7`: expected [["1984-08-11"]], observed [["1984-07-01"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-12`: expected [["1984-01-08"]], observed [["1984-01-02"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-14`: expected [["1984-08-11"]], observed [["1984-07-01"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-19`: expected [["1984-01-08"]], observed [["1984-01-02"]]
- `tck.expressions.temporal.temporal3.scenario-1.examples-1-row-21`: expected [["1984-08-11"]], observed [["1984-07-01"]]
- `tck.expressions.temporal.temporal3.scenario-2.examples-1-row-1`: query execution failed: Parse error: no such function: localtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 84..118; query:
WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 645876123}) AS other
RETURN localtime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-2.examples-1-row-4`: query execution failed: Parse error: no such function: localtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 97..131; query:
WITH time({hour: 12, minute: 31, second: 14, microsecond: 645876, timezone: '+01:00'}) AS other
RETURN localtime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-2.examples-1-row-7`: query execution failed: Parse error: no such function: localtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 119..153; query:
WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, millisecond: 645}) AS other
RETURN localtime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-2.examples-1-row-10`: query execution failed: Parse error: no such function: localtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..122; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other
RETURN localtime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-1`: expected [["12:31:14.645876123Z"]], observed [["12:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-6`: expected [["12:31:14.645876+01:00"]], observed [["11:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-8`: expected [["16:31:14.645876+05:00"]], observed [["12:31:14.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-10`: expected [["16:31:42.645876+05:00"]], observed [["12:31:42.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-11`: expected [["12:31:14.645Z"]], observed [["12:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-16`: expected [["12:00+01:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-17`: expected [["12:00+01:00"]], observed [["12:00+02:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-18`: expected [["16:00+05:00"]], observed [["12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-19`: expected [["12:00:42+01:00"]], observed [["12:00:42+02:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-20`: expected [["16:00:42+05:00"]], observed [["12:00:42+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-7.examples-1-row-1`: query execution failed: Parse error: no such function: localdatetime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 119..157; query:
WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, millisecond: 645}) AS other
RETURN localdatetime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-7.examples-1-row-4`: query execution failed: Parse error: no such function: localdatetime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..126; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other
RETURN localdatetime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-9.examples-1-row-6`: expected [["1984-10-11T16:31:14.645876+05:00"]], observed [["1984-10-11T12:31:14.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-9.examples-1-row-8`: expected [["1984-10-11T01:31:42.645876-10:00[Pacific/Honolulu]"]], observed [["1984-10-11T12:31:42.645876-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-9.examples-1-row-14`: expected [["1984-10-11T16:00+05:00"]], observed [["1984-10-11T12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-9.examples-1-row-16`: expected [["1984-10-11T01:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-10-11T12:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-6`: expected [["1984-10-11T16:31:14.645876+05:00"]], observed [["1984-10-11T12:31:14.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-8`: expected [["1984-10-28T01:31:42.645876-10:00[Pacific/Honolulu]"]], observed [["1984-10-28T12:31:42.645876-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-14`: expected [["1984-10-11T16:00+05:00"]], observed [["1984-10-11T12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-16`: expected [["1984-10-28T01:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-10-28T12:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-22`: expected [["1984-03-07T16:31:14.645876+05:00"]], observed [["1984-03-07T12:31:14.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-24`: expected [["1984-03-28T01:31:42.645876-10:00[Pacific/Honolulu]"]], observed [["1984-03-28T12:31:42.645876-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-30`: expected [["1984-03-07T16:00+05:00"]], observed [["1984-03-07T12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-32`: expected [["1984-03-28T00:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-03-28T12:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-38`: expected [["1984-10-11T16:31:14.645876+05:00"]], observed [["1984-10-11T12:31:14.645876+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-40`: expected [["1984-10-28T01:31:42.645876-10:00[Pacific/Honolulu]"]], observed [["1984-10-28T12:31:42.645876-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-46`: expected [["1984-10-11T16:00+05:00"]], observed [["1984-10-11T12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-48`: expected [["1984-10-28T01:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-10-28T12:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-1`: expected [["1984-03-07T12:31:14.645Z"]], observed [["1984-03-07 12:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-6`: expected [["1984-10-11T12:00+01:00[Europe/Stockholm]"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-8`: expected [["1984-10-11T16:00+05:00"]], observed [["1984-10-11T12:00+05:00"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-10`: expected [["1984-10-28T01:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-10-28T12:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-2`: query execution failed: Parse error: invalid resolved function or parameter name: date.transaction; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..36; query:
RETURN date.transaction(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-3`: query execution failed: Parse error: invalid resolved function or parameter name: date.statement; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..34; query:
RETURN date.statement(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-4`: query execution failed: Parse error: invalid resolved function or parameter name: date.realtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..33; query:
RETURN date.realtime(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-6`: query execution failed: Parse error: invalid resolved function or parameter name: localtime.transaction; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..41; query:
RETURN localtime.transaction(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-7`: query execution failed: Parse error: invalid resolved function or parameter name: localtime.statement; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..39; query:
RETURN localtime.statement(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-8`: query execution failed: Parse error: invalid resolved function or parameter name: localtime.realtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..38; query:
RETURN localtime.realtime(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-10`: query execution failed: Parse error: invalid resolved function or parameter name: time.transaction; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..36; query:
RETURN time.transaction(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-11`: query execution failed: Parse error: invalid resolved function or parameter name: time.statement; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..34; query:
RETURN time.statement(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-12`: query execution failed: Parse error: invalid resolved function or parameter name: time.realtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..33; query:
RETURN time.realtime(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-14`: query execution failed: Parse error: invalid resolved function or parameter name: localdatetime.transaction; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..45; query:
RETURN localdatetime.transaction(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-15`: query execution failed: Parse error: invalid resolved function or parameter name: localdatetime.statement; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..43; query:
RETURN localdatetime.statement(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-16`: query execution failed: Parse error: invalid resolved function or parameter name: localdatetime.realtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..42; query:
RETURN localdatetime.realtime(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-18`: query execution failed: Parse error: invalid resolved function or parameter name: datetime.transaction; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..40; query:
RETURN datetime.transaction(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-19`: query execution failed: Parse error: invalid resolved function or parameter name: datetime.statement; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..38; query:
RETURN datetime.statement(null) AS t
- `tck.expressions.temporal.temporal4.scenario-13.examples-1-row-20`: query execution failed: Parse error: invalid resolved function or parameter name: datetime.realtime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..37; query:
RETURN datetime.realtime(null) AS t
- `tck.expressions.temporal.temporal5.scenario-1`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..134; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.year, d.quarter, d.month, d.week, d.weekYear, d.day, d.ordinalDay, d.weekDay, d.dayOfQuarter
- `tck.expressions.temporal.temporal5.scenario-2`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..77; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.year, d.weekYear, d.week, d.weekDay
- `tck.expressions.temporal.temporal5.scenario-3`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..110; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.hour, d.minute, d.second, d.millisecond, d.microsecond, d.nanosecond
- `tck.expressions.temporal.temporal5.scenario-4`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..166; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.hour, d.minute, d.second, d.millisecond, d.microsecond, d.nanosecond, d.timezone, d.offset, d.offsetMinutes, d.offsetSeconds
- `tck.expressions.temporal.temporal5.scenario-5`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..213; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.year, d.quarter, d.month, d.week, d.weekYear, d.day, d.ordinalDay, d.weekDay, d.dayOfQuarter,
       d.hour, d.minute, d.second, d.millisecond, d.microsecond, d.nanosecond
- `tck.expressions.temporal.temporal5.scenario-6`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..307; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.year, d.quarter, d.month, d.week, d.weekYear, d.day, d.ordinalDay, d.weekDay, d.dayOfQuarter,
       d.hour, d.minute, d.second, d.millisecond, d.microsecond, d.nanosecond,
       d.timezone, d.offset, d.offsetMinutes, d.offsetSeconds, d.epochSeconds, d.epochMillis
- `tck.expressions.temporal.temporal5.scenario-7`: query execution failed: Parse error: property access requires a node or relationship at byte 39..40; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 32..352; query:
MATCH (v:Val)
WITH v.date AS d
RETURN d.years, d.quarters, d.months, d.weeks, d.days,
       d.hours, d.minutes, d.seconds, d.milliseconds, d.microseconds, d.nanoseconds,
       d.quartersOfYear, d.monthsOfQuarter, d.monthsOfYear, d.daysOfWeek, d.minutesOfHour, d.secondsOfMinute, d.millisecondsOfSecond, d.microsecondsOfSecond, d.nanosecondsOfSecond
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-4`: expected [["PT1.999S", "1"]], observed [["PT1.999000000S", "1"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-5`: expected [["PT-1.999S", "1"]], observed [["PT-2.001000000S", "0"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-6`: expected [["PT-2.001S", "1"]], observed [["PT-3.999000000S", "0"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-7`: expected [["P1DT0.001S", "1"]], observed [["P1DT0.001000000S", "1"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-8`: expected [["P1DT-0.001S", "1"]], observed [["P1DT-1.999000000S", "0"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-9`: expected [["PT59.999S", "1"]], observed [["PT59.999000000S", "1"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-10`: expected [["PT-59.999S", "1"]], observed [["PT-1M0.001000000S", "1"]]
- `tck.expressions.temporal.temporal6.scenario-6.examples-1-row-11`: expected [["PT-1M-0.001S", "1"]], observed [["PT-1M-1.999000000S", "0"]]
- `tck.expressions.temporal.temporal7.scenario-3.examples-1-row-1`: expected [["0", "1", "0", "1", "0"]], observed [["1", "0", "1", "0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-1.examples-1-row-1`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 98..104; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 69..112; query:
WITH date({year: 1984, month: 10, day: 11}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-1.examples-1-row-2`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 98..104; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 69..112; query:
WITH date({year: 1984, month: 10, day: 11}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-1.examples-1-row-3`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 98..104; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 69..112; query:
WITH date({year: 1984, month: 10, day: 11}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-2.examples-1-row-1`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 120..126; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 91..134; query:
WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-2.examples-1-row-2`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 120..126; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 91..134; query:
WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-2.examples-1-row-3`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 120..126; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 91..134; query:
WITH localtime({hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-3.examples-1-row-1`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 135..141; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 106..149; query:
WITH time({hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-3.examples-1-row-2`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 135..141; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 106..149; query:
WITH time({hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-3.examples-1-row-3`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 135..141; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 106..149; query:
WITH time({hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-4.examples-1-row-1`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 156..162; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 127..170; query:
WITH localdatetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-4.examples-1-row-2`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 156..162; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 127..170; query:
WITH localdatetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-4.examples-1-row-3`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 156..162; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 127..170; query:
WITH localdatetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-5.examples-1-row-1`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 171..177; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 142..185; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-5.examples-1-row-2`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 171..177; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 142..185; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-5.examples-1-row-3`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 171..177; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 142..185; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, minute: 31, second: 14, nanosecond: 1, timezone: '+01:00'}) AS x
MATCH (d:Duration)
RETURN x + d.dur AS sum, x - d.dur AS diff
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-1`: expected [["P24Y10M28DT32H26M20.000000002S", "PT0S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-2`: expected [["P12Y6MT32H2M20.000000001S", "P12Y4M28DT24M0.000000001S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-3`: expected [["P25Y4M43DT50H11M23.500000004S", "P-6M-15DT-17H-45M-3.500000002S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-4`: expected [["P12Y6MT32H2M20.000000001S", "P-12Y-4M-28DT-24M-0.000000001S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-5`: expected [["P2M-28DT31H38M20S", "PT0S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-6`: expected [["P13Y15DT49H47M23.500000003S", "P-12Y-10M-43DT-18H-9M-3.500000003S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-7`: expected [["P25Y4M43DT50H11M23.500000004S", "P6M15DT17H45M3.500000002S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-8`: expected [["P13Y15DT49H47M23.500000003S", "P12Y10M43DT18H9M3.500000003S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-9`: expected [["P25Y10M58DT67H56M27.000000006S", "PT0S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-1`: expected [["P12Y5M14DT16H13M10.000000001S", "P12Y5M14DT16H13M10.000000001S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-2`: expected [["P24Y10M28DT32H26M20.000000002S", "P6Y2M22DT13H21M8S"]], observed [["0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-3`: expected [["P6Y2M22DT13H21M8S", "P24Y10M28DT32H26M20.000000002S"]], observed [["0.0", "0.0"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-43`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-45`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-47`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-64`: expected [["1984-10-09T00:00Z"]], observed [["1984-10-08T00:00Z"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-67`: expected [["1984-10-09T00:00+01:00"]], observed [["1984-10-08T00:00+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-70`: expected [["1984-10-09T00:00Z"]], observed [["1984-10-08T00:00Z"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-98`: expected [["1984-10-11T12:31:14.645000002+01:00"]], observed [["1984-10-11T12:31:14.000000002+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-100`: expected [["1984-10-11T12:31:14.645000002Z"]], observed [["1984-10-11T12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-102`: expected [["1984-10-11T12:31:14.645876002+01:00"]], observed [["1984-10-11T12:31:14.000000002+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-104`: expected [["1984-10-11T12:31:14.645876002Z"]], observed [["1984-10-11T12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-43`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-45`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-47`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-67`: expected [["1984-10-11T12:31:14.645000002"]], observed [["1984-10-11T12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-69`: expected [["1984-10-11T12:31:14.645000002"]], observed [["1984-10-11T12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-71`: expected [["1984-10-11T12:31:14.645876002"]], observed [["1984-10-11T12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-73`: expected [["1984-10-11T12:31:14.645876002"]], observed [["1984-10-11T12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-29`: expected [["12:31:14.645000002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-31`: expected [["12:31:14.645000002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-33`: expected [["12:31:14.645000002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-35`: expected [["12:31:14.645000002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-37`: expected [["12:31:14.645876002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-39`: expected [["12:31:14.645876002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-41`: expected [["12:31:14.645876002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-4.examples-1-row-43`: expected [["12:31:14.645876002"]], observed [["12:31:14.000000002"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-33`: expected [["12:31:14.645000002+01:00"]], observed [["12:31:14.000000002+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-35`: expected [["12:31:14.645000002Z"]], observed [["12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-37`: expected [["12:31:14.645000002Z"]], observed [["12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-39`: expected [["12:31:14.645000002+01:00"]], observed [["12:31:14.000000002+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-41`: expected [["12:31:14.645876002+01:00"]], observed [["12:31:14.000000002+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-43`: expected [["12:31:14.645876002Z"]], observed [["12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-45`: expected [["12:31:14.645876002Z"]], observed [["12:31:14.000000002Z"]]
- `tck.expressions.temporal.temporal9.scenario-5.examples-1-row-47`: expected [["12:31:14.645876002+01:00"]], observed [["12:31:14.000000002+01:00"]]
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-1`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-2`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-3`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-4`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-5`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion1.scenario-5.examples-1-row-6`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion2.scenario-2`: expected [["<null>", "<null>"]], observed [["0", "0"]]
- `tck.expressions.typeconversion.typeconversion2.scenario-5`: expected [["[2, 2, null]"]], observed [["[2, 2, 0]"]]
- `tck.expressions.typeconversion.typeconversion2.scenario-8.examples-1-row-1`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion2.scenario-8.examples-1-row-2`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion2.scenario-8.examples-1-row-3`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion2.scenario-8.examples-1-row-4`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion2.scenario-8.examples-1-row-5`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-2`: expected [["<null>", "<null>"]], observed [["0.0", "0.0"]]
- `tck.expressions.typeconversion.typeconversion3.scenario-4`: expected [["[1.0, 2.0, null]"]], observed [["[1.0, 2.0, 0.0]"]]
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-1`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-2`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-3`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-4`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-5`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion3.scenario-6.examples-1-row-6`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion4.scenario-5`: expected [["[\"1\",\"2.3\",\"true\",\"apa\"]"]], observed [["[\"1\",\"2.3\",\"1\",\"apa\"]"]]
- `tck.expressions.typeconversion.typeconversion4.scenario-10.examples-1-row-1`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion4.scenario-10.examples-1-row-2`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion4.scenario-10.examples-1-row-3`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion4.scenario-10.examples-1-row-4`: expected an error but execution succeeded
- `tck.expressions.typeconversion.typeconversion4.scenario-10.examples-1-row-5`: expected an error but execution succeeded
- `tck.usecases.countingsubgraphmatches.countingsubgraphmatches1.scenario-10`: expected [["2"]], observed [["3"]]
- `tck.usecases.countingsubgraphmatches.countingsubgraphmatches1.scenario-11`: expected [["6"]], observed [["11"]]
- `tck.usecases.triadicselection.triadicselection1.scenario-1`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-2`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-3`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-4`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-5`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-6`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-7`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-8`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-9`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-10`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-11`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-12`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-13`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-14`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-15`: TCK named graph `binary-tree-1` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:X {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:X {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:X {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:X {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-16`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-17`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-18`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);
- `tck.usecases.triadicselection.triadicselection1.scenario-19`: TCK named graph `binary-tree-2` setup failed: query execution failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
       (b1:X {name: 'b1'}),
       (b2:X {name: 'b2'}),
       (b3:X {name: 'b3'}),
       (b4:X {name: 'b4'}),
       (c11:X {name: 'c11'}),
       (c12:Y {name: 'c12'}),
       (c21:X {name: 'c21'}),
       (c22:Y {name: 'c22'}),
       (c31:X {name: 'c31'}),
       (c32:Y {name: 'c32'}),
       (c41:X {name: 'c41'}),
       (c42:Y {name: 'c42'})
CREATE (a)-[:KNOWS]->(b1),
       (a)-[:KNOWS]->(b2),
       (a)-[:FOLLOWS]->(b3),
       (a)-[:FOLLOWS]->(b4)
CREATE (b1)-[:FRIEND]->(c11),
       (b1)-[:FRIEND]->(c12),
       (b2)-[:FRIEND]->(c21),
       (b2)-[:FRIEND]->(c22),
       (b3)-[:FRIEND]->(c31),
       (b3)-[:FRIEND]->(c32),
       (b4)-[:FRIEND]->(c41),
       (b4)-[:FRIEND]->(c42)
CREATE (b1)-[:FRIEND]->(b2),
       (b2)-[:FRIEND]->(b3),
       (b3)-[:FRIEND]->(b4),
       (b4)-[:FRIEND]->(b1);

## Longitudinal inventory

- Runs: 304
- Result records: 623585
- Unique test identities: 10441
