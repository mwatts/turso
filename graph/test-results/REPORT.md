# Graph test history

Generated from `graph/test-results/history.jsonl`. Results are grouped by stable test identity; performance comparisons are meaningful only for matching environment and workload dimensions.

## Latest complete corpus run

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Semantics: v3
- Records: 10242
- Passed: 8928
- Unsupported: 53
- Failed: 1261

### Failure-reason histogram

| Failure family | Count |
|---|---:|
| `execution`: other | 489 |
| `execution`: mutation projection unsupported | 252 |
| `execution`: runtime scalar function missing | 176 |
| `parser`: other grammar | 112 |
| `parser`: expression/operator continuation grammar | 43 |
| `execution`: expected-error mismatch | 38 |
| `setup-execution`: other | 34 |
| `execution`: mutation operation unsupported | 32 |
| `side-effect-comparison`: other | 23 |
| `fixture-execution`: other | 19 |
| `execution`: parameter binding/declaration | 18 |
| `parser`: graph-pattern grammar | 12 |
| `parser`: map-literal grammar | 5 |
| `dataset-execution`: other | 4 |
| `parser`: projection/expression item grammar | 2 |
| `setup-execution`: runtime scalar function missing | 2 |

## Latest `age-deep` run

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Commit: `8e296519275bbca3b006b64032b9776d76de5037` (dirty)
- Package: `0.8.0-pre.1`
- Semantics: v3
- Environment: `macos/aarch64` (`release`)
- Records: 3595
- Passed: 3042
- Unsupported: 53
- Failed or changed: 500

### Outcome changes from `20260726T225523.619934Z-f5c009b2f8e2-corpus-deep`

- No outcome changes.

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| age_global_graph | `failed` | 1 |
| age_global_graph | `passed` | 29 |
| age_global_graph | `unsupported` | 27 |
| age_load | `failed` | 1 |
| age_load | `passed` | 11 |
| age_load | `unsupported` | 1 |
| age_reduce | `failed` | 3 |
| age_reduce | `passed` | 71 |
| age_shortest_path | `failed` | 11 |
| age_shortest_path | `passed` | 175 |
| age_shortest_path | `unsupported` | 8 |
| agtype | `failed` | 1 |
| agtype | `passed` | 17 |
| agtype_jsonb_cast | `passed` | 3 |
| analyze | `passed` | 2 |
| catalog | `passed` | 7 |
| cypher | `passed` | 20 |
| cypher_call | `failed` | 30 |
| cypher_call | `passed` | 12 |
| cypher_create | `failed` | 13 |
| cypher_create | `passed` | 80 |
| cypher_delete | `failed` | 11 |
| cypher_delete | `passed` | 104 |
| cypher_match | `failed` | 37 |
| cypher_match | `passed` | 341 |
| cypher_merge | `failed` | 18 |
| cypher_merge | `passed` | 255 |
| cypher_remove | `failed` | 1 |
| cypher_remove | `passed` | 41 |
| cypher_set | `failed` | 10 |
| cypher_set | `passed` | 107 |
| cypher_subquery | `failed` | 8 |
| cypher_subquery | `passed` | 45 |
| cypher_union | `failed` | 14 |
| cypher_union | `passed` | 5 |
| cypher_unwind | `failed` | 2 |
| cypher_unwind | `passed` | 15 |
| cypher_vle | `failed` | 39 |
| cypher_vle | `passed` | 73 |
| cypher_with | `failed` | 8 |
| cypher_with | `passed` | 33 |
| direct_field_access | `failed` | 4 |
| direct_field_access | `passed` | 37 |
| expr | `failed` | 174 |
| expr | `passed` | 902 |
| expr | `unsupported` | 11 |
| generated_columns | `passed` | 10 |
| graph_generation | `passed` | 2 |
| index | `passed` | 65 |
| issue_369 | `passed` | 4 |
| jsonb_operators | `failed` | 35 |
| jsonb_operators | `passed` | 124 |
| list_comprehension | `failed` | 12 |
| list_comprehension | `passed` | 110 |
| map_projection | `failed` | 14 |
| map_projection | `passed` | 4 |
| name_validation | `passed` | 4 |
| name_validation | `unsupported` | 6 |
| pattern_expression | `failed` | 4 |
| pattern_expression | `passed` | 28 |
| pgvector | `failed` | 18 |
| pgvector | `passed` | 31 |
| predicate_functions | `failed` | 9 |
| predicate_functions | `passed` | 53 |
| reserved_keyword_alias | `failed` | 6 |
| reserved_keyword_alias | `passed` | 25 |
| scan | `failed` | 14 |
| scan | `passed` | 42 |
| security | `failed` | 2 |
| security | `passed` | 131 |
| subgraph | `passed` | 24 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 417 |
| `execution` | `passed` | 3001 |
| `execution` | `unsupported` | 53 |
| `parser` | `failed` | 83 |
| `parser` | `passed` | 41 |

### Failures (500)

- `age.age.global.graph.query-51`: query execution failed: Parse error: property access requires a node or relationship at byte 132..133; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 89..140
- `age.age.load.query-8`: expected primary_expression at byte 56..56
- `age.age.reduce.query-39`: query execution failed: Parse error: circular reference: r0_0; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..78
- `age.age.reduce.query-40`: query execution failed: Parse error: circular reference: r0_0; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..79
- `age.age.reduce.query-74`: query execution failed: Parse error: unknown parameter `$p` at byte 46..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..49
- `age.age.shortest.path.query-25`: query succeeded but AGE expects an error
- `age.age.shortest.path.query-26`: query succeeded but AGE expects an error
- `age.age.shortest.path.query-67`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..71
- `age.age.shortest.path.query-68`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..76
- `age.age.shortest.path.query-69`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..80
- `age.age.shortest.path.query-135`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..80
- `age.age.shortest.path.query-136`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..85
- `age.age.shortest.path.query-137`: query execution failed: Parse error: no such function: shortest_path; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..91
- `age.age.shortest.path.query-138`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..96
- `age.age.shortest.path.query-139`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..96
- `age.age.shortest.path.query-140`: query execution failed: Parse error: no such function: all_shortest_paths; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 45..87
- `age.agtype.query-9`: query execution failed: Parse error: invalid resolved function or parameter name: ag_catalog.agtype_build_map; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..73
- `age.cypher.call.query-3`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.cypher.call.query-5`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.cypher.call.query-7`: query execution failed: Parse error: unsupported graph procedure `call_stmt_test.add_agtype` at byte 5..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..35
- `age.cypher.call.query-10`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-12`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-13`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-14`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-15`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-17`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 15..19; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 10..35
- `age.cypher.call.query-18`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-19`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-20`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-21`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-22`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-23`: expected EOI, UNION, or clause at byte 35..35
- `age.cypher.call.query-24`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-25`: expected identifier at byte 58..58
- `age.cypher.call.query-26`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-27`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-28`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-29`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-30`: query execution failed: Parse error: unsupported graph procedure `sqrt` at byte 5..9; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..25
- `age.cypher.call.query-32`: expected EOI, UNION, or clause at byte 50..50
- `age.cypher.call.query-33`: expected EOI, UNION, or clause at byte 25..25
- `age.cypher.call.query-36`: query execution failed: Parse error: unsupported graph procedure `ag_catalog.age_sqrt` at byte 5..24; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..44
- `age.cypher.call.query-37`: query execution failed: Parse error: unsupported graph procedure `myfunc` at byte 5..11; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.cypher.call.query-38`: query execution failed: Parse error: unsupported graph procedure `ag_catalog.myfunc` at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.cypher.call.query-40`: query execution failed: Parse error: unsupported graph procedure `myfunz` at byte 5..11; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.cypher.call.query-41`: query execution failed: Parse error: unsupported graph procedure `ag_catalog.myfunc` at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..38
- `age.cypher.call.query-42`: query execution failed: Parse error: unsupported graph procedure `ag_catalog.myfunz` at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.cypher.create.query-36`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..39; mutation execution failed: Cypher mutation binding failed: unknown parameter `$var_name` at byte 27..36
- `age.cypher.create.query-65`: query succeeded but AGE expects an error
- `age.cypher.create.query-74`: expected identifier at byte 10..10
- `age.cypher.create.query-75`: expected identifier at byte 10..10
- `age.cypher.create.query-76`: expected identifier at byte 10..10
- `age.cypher.create.query-77`: query succeeded but AGE expects an error
- `age.cypher.create.query-78`: query succeeded but AGE expects an error
- `age.cypher.create.query-80`: query succeeded but AGE expects an error
- `age.cypher.create.query-81`: query succeeded but AGE expects an error
- `age.cypher.create.query-82`: query succeeded but AGE expects an error
- `age.cypher.create.query-83`: query succeeded but AGE expects an error
- `age.cypher.create.query-88`: query succeeded but AGE expects an error
- `age.cypher.create.query-90`: query succeeded but AGE expects an error
- `age.cypher.delete.query-7`: query succeeded but AGE expects an error
- `age.cypher.delete.query-8`: query succeeded but AGE expects an error
- `age.cypher.delete.query-29`: query succeeded but AGE expects an error
- `age.cypher.delete.query-45`: query succeeded but AGE expects an error
- `age.cypher.delete.query-49`: query succeeded but AGE expects an error
- `age.cypher.delete.query-54`: query succeeded but AGE expects an error
- `age.cypher.delete.query-70`: query succeeded but AGE expects an error
- `age.cypher.delete.query-74`: expected identifier at byte 17..17
- `age.cypher.delete.query-77`: expected identifier at byte 17..17
- `age.cypher.delete.query-105`: expected identifier at byte 27..27
- `age.cypher.delete.query-106`: query succeeded but AGE expects an error
- `age.cypher.match.query-65`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-66`: query succeeded but AGE expects an error
- `age.cypher.match.query-131`: query execution failed: Parse error: duplicate variable `e` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 19..55
- `age.cypher.match.query-133`: query succeeded but AGE expects an error
- `age.cypher.match.query-136`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 29..37
- `age.cypher.match.query-137`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..41
- `age.cypher.match.query-138`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..57
- `age.cypher.match.query-139`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..61
- `age.cypher.match.query-140`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..44
- `age.cypher.match.query-141`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..46
- `age.cypher.match.query-156`: query succeeded but AGE expects an error
- `age.cypher.match.query-157`: query succeeded but AGE expects an error
- `age.cypher.match.query-171`: query execution failed: Parse error: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..215; mutation execution failed: Cypher mutation binding failed: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..95
- `age.cypher.match.query-172`: query execution failed: Parse error: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 0..234; mutation execution failed: Cypher mutation binding failed: multiple OPTIONAL MATCH paths is not supported in the initial graph slice at byte 68..114
- `age.cypher.match.query-173`: query succeeded but AGE expects an error
- `age.cypher.match.query-183`: expected identifier at byte 24..24
- `age.cypher.match.query-194`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..39
- `age.cypher.match.query-195`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 31..45; mutation execution failed: graph mutation database operation failed: near "q": syntax error
- `age.cypher.match.query-208`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..57
- `age.cypher.match.query-209`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..61
- `age.cypher.match.query-227`: query execution failed: Parse error: duplicate variable `b` at byte 21..22; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 21..22
- `age.cypher.match.query-228`: query execution failed: Parse error: duplicate variable `b` at byte 27..28; mutation execution failed: Cypher mutation binding failed: duplicate variable `b` at byte 27..28
- `age.cypher.match.query-231`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 9..10
- `age.cypher.match.query-232`: query execution failed: Parse error: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 10..16; mutation execution failed: Cypher mutation binding failed: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 10..16
- `age.cypher.match.query-233`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19
- `age.cypher.match.query-234`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 18..19
- `age.cypher.match.query-235`: query execution failed: Parse error: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 19..25; mutation execution failed: Cypher mutation binding failed: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 19..25
- `age.cypher.match.query-236`: query execution failed: Parse error: duplicate variable `p` at byte 16..17; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 16..17
- `age.cypher.match.query-237`: query execution failed: Parse error: duplicate variable `p` at byte 23..24; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 23..24
- `age.cypher.match.query-238`: query execution failed: Parse error: duplicate variable `p` at byte 19..20; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 19..20
- `age.cypher.match.query-239`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..11; mutation execution failed: Cypher mutation binding failed: named paths in MATCH after a mutation clause is not supported in the initial graph slice at byte 18..29
- `age.cypher.match.query-242`: query succeeded but AGE expects an error
- `age.cypher.match.query-348`: query succeeded but AGE expects an error
- `age.cypher.match.query-388`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..44; mutation execution failed: Cypher mutation binding failed: named paths in MATCH after a mutation clause is not supported in the initial graph slice at byte 61..88
- `age.cypher.match.query-396`: expected node_labels or map_literal at byte 9..9
- `age.cypher.match.query-397`: expected relationship_types, range_literal, or map_literal at byte 12..12
- `age.cypher.match.query-398`: expected node_labels or map_literal at byte 9..9
- `age.cypher.merge.query-81`: query succeeded but AGE expects an error
- `age.cypher.merge.query-106`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 16..32; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 24..30
- `age.cypher.merge.query-122`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..45; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 27..32
- `age.cypher.merge.query-123`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..46; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 27..32
- `age.cypher.merge.query-124`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..47; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-125`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..48; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-126`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..48; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 29..34
- `age.cypher.merge.query-129`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-134`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..23; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 18..23
- `age.cypher.merge.query-144`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..26; mutation execution failed: Cypher mutation binding failed: labels or properties on an already-bound CREATE node is not supported in the initial graph slice at byte 20..25
- `age.cypher.merge.query-148`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..31; mutation execution failed: Cypher mutation binding failed: relationship creation without exactly one type is not supported in the initial graph slice at byte 22..28
- `age.cypher.merge.query-164`: query succeeded but AGE expects an error
- `age.cypher.merge.query-166`: query succeeded but AGE expects an error
- `age.cypher.merge.query-169`: query succeeded but AGE expects an error
- `age.cypher.merge.query-170`: query succeeded but AGE expects an error
- `age.cypher.merge.query-182`: expected identifier at byte 16..16
- `age.cypher.merge.query-184`: expected identifier at byte 16..16
- `age.cypher.merge.query-187`: expected identifier at byte 16..16
- `age.cypher.remove.query-42`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..28; mutation execution failed: Cypher mutation binding failed: unknown variable `wrong_var` at byte 17..26
- `age.cypher.set.query-28`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 89..110; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 93..94
- `age.cypher.set.query-29`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 97..118; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 101..102
- `age.cypher.set.query-34`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..30; mutation execution failed: Cypher mutation binding failed: unknown parameter `$var_name` at byte 20..30
- `age.cypher.set.query-40`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..19; mutation execution failed: Cypher mutation binding failed: unknown variable `i` at byte 14..15
- `age.cypher.set.query-80`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 52..68; mutation execution failed: Cypher mutation binding failed: SET of a whole entity from a non-map value is not supported in the initial graph slice at byte 61..68
- `age.cypher.set.query-84`: query succeeded but AGE expects an error
- `age.cypher.set.query-94`: expected identifier at byte 17..17
- `age.cypher.set.query-96`: expected identifier at byte 17..17
- `age.cypher.set.query-98`: expected identifier at byte 17..17
- `age.cypher.set.query-112`: query execution failed: Parse error: property access requires a node or relationship at byte 34..38; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..38
- `age.cypher.subquery.query-9`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 159..159
- `age.cypher.subquery.query-10`: expected ORDER, SKIP, LIMIT, AS, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 159..159
- `age.cypher.subquery.query-11`: expected RETURN, AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 137..137
- `age.cypher.subquery.query-16`: query execution failed: Parse error: unknown variable `b` at byte 142..144; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 142..144
- `age.cypher.subquery.query-17`: query execution failed: Parse error: unknown variable `b` at byte 170..171; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 170..171
- `age.cypher.subquery.query-19`: query execution failed: Parse error: unknown variable `b` at byte 160..178; mutation execution failed: Cypher mutation binding failed: unknown variable `b` at byte 160..178
- `age.cypher.subquery.query-45`: query execution failed: Parse error: coalesce function with less than 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 241..249
- `age.cypher.subquery.query-46`: query execution failed: Parse error: coalesce function with less than 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 34..248
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
- `age.cypher.unwind.query-13`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 58..82; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 62..66
- `age.cypher.vle.query-19`: relationship range is outside the supported u32 range at byte 22..24
- `age.cypher.vle.query-20`: relationship range is outside the supported u32 range at byte 17..19
- `age.cypher.vle.query-21`: query execution failed: Parse error: variable-length relationship properties is not supported in the initial graph slice at byte 8..38; mutation execution failed: Cypher mutation binding failed: variable-length relationship properties is not supported in the initial graph slice at byte 8..38
- `age.cypher.vle.query-22`: query execution failed: Parse error: variable-length relationship properties is not supported in the initial graph slice at byte 9..39; mutation execution failed: Cypher mutation binding failed: variable-length relationship properties is not supported in the initial graph slice at byte 9..39
- `age.cypher.vle.query-23`: query execution failed: Parse error: variable-length relationship properties is not supported in the initial graph slice at byte 8..38; mutation execution failed: Cypher mutation binding failed: variable-length relationship properties is not supported in the initial graph slice at byte 8..38
- `age.cypher.vle.query-28`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 53..55; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..58
- `age.cypher.vle.query-35`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 54..56; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..59
- `age.cypher.vle.query-47`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 59..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 41..64
- `age.cypher.vle.query-52`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 48..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..53
- `age.cypher.vle.query-53`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 48..50; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 30..53
- `age.cypher.vle.query-60`: query execution failed: Parse error: duplicate variable `p` at byte 19..20; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 19..20
- `age.cypher.vle.query-61`: query execution failed: Parse error: duplicate variable `p` at byte 11..12; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 11..12
- `age.cypher.vle.query-62`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 15..18; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 15..18
- `age.cypher.vle.query-63`: query execution failed: Parse error: duplicate variable `p` at byte 12..13; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 12..13
- `age.cypher.vle.query-65`: query execution failed: Parse error: duplicate variable `p` at byte 26..27; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 26..27
- `age.cypher.vle.query-66`: query execution failed: Parse error: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 26..31; mutation execution failed: Cypher mutation binding failed: reusing a non-relationship variable in a relationship pattern is not supported in the initial graph slice at byte 26..31
- `age.cypher.vle.query-67`: query succeeded but AGE expects an error
- `age.cypher.vle.query-69`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 25..26; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 25..26
- `age.cypher.vle.query-70`: query execution failed: Parse error: duplicate variable `p` at byte 21..22; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 21..22
- `age.cypher.vle.query-71`: query execution failed: Parse error: duplicate variable `p` at byte 24..25; mutation execution failed: Cypher mutation binding failed: duplicate variable `p` at byte 24..25
- `age.cypher.vle.query-78`: query execution failed: Parse error: unsupported relational graph operator: properties() with a non-entity argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-79`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..58
- `age.cypher.vle.query-80`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..63
- `age.cypher.vle.query-81`: query execution failed: Parse error: unsupported relational graph operator: properties() with a non-entity argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-82`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..58
- `age.cypher.vle.query-83`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..63
- `age.cypher.vle.query-84`: query execution failed: Parse error: property access requires a node or relationship at byte 29..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..69
- `age.cypher.vle.query-85`: query execution failed: Parse error: unsupported relational graph operator: properties() with a non-entity argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..67
- `age.cypher.vle.query-86`: query execution failed: Parse error: unsupported relational graph operator: properties() with a non-entity argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..95
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
- `age.cypher.with.query-9`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 91..165
- `age.cypher.with.query-13`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 161..199
- `age.cypher.with.query-18`: query execution failed: Parse error: unknown variable `b` at byte 44..45; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..45
- `age.cypher.with.query-19`: query execution failed: Parse error: unknown variable `end_node` at byte 177..185; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 155..190
- `age.cypher.with.query-21`: query execution failed: Parse error: unknown variable `d` at byte 156..157; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 147..157
- `age.cypher.with.query-23`: query execution failed: Parse error: unknown variable `v` at byte 74..75; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 67..87
- `age.direct.field.access.query-30`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 73..98
- `age.direct.field.access.query-31`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 73..96
- `age.direct.field.access.query-33`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..51
- `age.direct.field.access.query-36`: query execution failed: Parse error: duplicate variable `name` at byte 204..222; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..222
- `age.expr.query-7`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-8`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..20
- `age.expr.query-9`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..22
- `age.expr.query-10`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-11`: query execution failed: Parse error: unknown parameter `$var` at byte 7..11; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..11
- `age.expr.query-88`: expected identifier, node_labels, not_expression, or map_literal at byte 8..8
- `age.expr.query-167`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 90..97; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..97
- `age.expr.query-168`: query execution failed: Parse error: arithmetic on non-numeric operands is not supported in the initial graph slice at byte 90..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..99
- `age.expr.query-169`: query execution failed: Extension error: TypeError: invalid operand types for -; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 79..97
- `age.expr.query-183`: expected primary_expression at byte 31..31
- `age.expr.query-184`: expected primary_expression at byte 31..31
- `age.expr.query-185`: expected primary_expression at byte 31..31
- `age.expr.query-186`: expected primary_expression at byte 31..31
- `age.expr.query-201`: query succeeded but AGE expects an error
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
- `age.expr.query-344`: query execution failed: Parse error: startNode()/endNode() require a relationship argument is not supported in the initial graph slice at byte 17..29; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..29
- `age.expr.query-345`: query execution failed: Parse error: startNode()/endNode() require exactly one relationship argument is not supported in the initial graph slice at byte 7..18; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-346`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..51
- `age.expr.query-348`: query execution failed: Parse error: startNode()/endNode() require a relationship argument is not supported in the initial graph slice at byte 17..27; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..27
- `age.expr.query-349`: query execution failed: Parse error: startNode()/endNode() require exactly one relationship argument is not supported in the initial graph slice at byte 7..16; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-352`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..24
- `age.expr.query-353`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-355`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..32
- `age.expr.query-356`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..64
- `age.expr.query-358`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 18..33
- `age.expr.query-359`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-360`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..27
- `age.expr.query-361`: query execution failed: Parse error: no such function: label; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..54
- `age.expr.query-368`: query execution failed: Parse error: no such function: size; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-377`: query execution failed: Parse error: no such function: head; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-386`: query execution failed: Parse error: no such function: last; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-392`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..23
- `age.expr.query-393`: query execution failed: Parse error: no such function: properties; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `age.expr.query-400`: query execution failed: Parse error: coalesce function with less than 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..21
- `age.expr.query-401`: query execution failed: Parse error: coalesce function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-410`: query execution failed: Parse error: no such function: toBoolean; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-419`: query execution failed: Parse error: unknown variable `fail` at byte 21..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..26
- `age.expr.query-420`: query execution failed: Parse error: malformed JSON; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-429`: query execution failed: Parse error: conversion from this value type is not supported in the initial graph slice at byte 7..20; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..20
- `age.expr.query-430`: query execution failed: Parse error: no such function: toFloat; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-440`: query execution failed: Parse error: unknown variable `failed` at byte 20..26; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-441`: query execution failed: Parse error: malformed JSON; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-451`: query execution failed: Parse error: no such function: toInteger; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-459`: query execution failed: Parse error: no such function: toIntegerList; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..32
- `age.expr.query-465`: query execution failed: Parse error: length function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-467`: query execution failed: Parse error: no such function: toString; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..17
- `age.expr.query-474`: query execution failed: Parse error: unknown variable `b` at byte 27..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-475`: query execution failed: Parse error: unknown variable `test` at byte 21..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-489`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..53
- `age.expr.query-492`: query execution failed: Internal error: expected 1 argument(s), got 0; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-504`: query execution failed: Parse error: no such function: toUpper; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-506`: query execution failed: Parse error: no such function: toLower; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-522`: query execution failed: Parse error: ltrim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-523`: query execution failed: Parse error: rtrim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-524`: query execution failed: Parse error: trim function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-532`: query execution failed: Parse error: no such function: left; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-543`: query execution failed: Parse error: no such function: right; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-563`: query execution failed: Parse error: wrong number of arguments to function substring(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.expr.query-573`: query execution failed: Parse error: split() over non-text arguments is not supported in the initial graph slice at byte 7..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-574`: query execution failed: Parse error: split() over non-text arguments is not supported in the initial graph slice at byte 7..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-575`: query execution failed: Extension error: split() requires exactly two arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-576`: query execution failed: Extension error: split() requires exactly two arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-588`: query execution failed: Parse error: wrong number of arguments to function replace(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-589`: query execution failed: Parse error: wrong number of arguments to function replace(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..23
- `age.expr.query-590`: query execution failed: Parse error: wrong number of arguments to function replace(); mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.expr.query-596`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-600`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-604`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-605`: query execution failed: Parse error: sin function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-606`: query execution failed: Parse error: cos function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-607`: query execution failed: Parse error: tan function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-608`: query execution failed: Parse error: no such function: cot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-628`: query execution failed: Parse error: asin function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-629`: query execution failed: Parse error: acos function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-630`: query execution failed: Parse error: atan function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-631`: query execution failed: Parse error: atan2 function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-632`: query execution failed: Parse error: atan2 function called with not exactly 2 arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-639`: query execution failed: Parse error: pi function with arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-640`: query execution failed: Parse error: pi function with arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-651`: query execution failed: Parse error: radians function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-652`: query execution failed: Parse error: degrees function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-688`: query execution failed: Parse error: abs function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-689`: query execution failed: Parse error: ceil function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-690`: query execution failed: Parse error: floor function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-691`: query execution failed: Parse error: round function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-692`: query execution failed: Parse error: sign function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-709`: query execution failed: Parse error: log function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-710`: query execution failed: Parse error: log10 function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..14
- `age.expr.query-711`: query execution failed: Parse error: no such function: e; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.expr.query-712`: query execution failed: Parse error: no such function: e; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..15
- `age.expr.query-716`: query execution failed: Parse error: exp function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..12
- `age.expr.query-723`: query execution failed: Parse error: sqrt function with no arguments; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-725`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..37
- `age.expr.query-726`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..39
- `age.expr.query-727`: query execution failed: Parse error: invalid resolved function or parameter name: ag_catalog.age_sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..30
- `age.expr.query-728`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..39
- `age.expr.query-729`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..24
- `age.expr.query-730`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..37
- `age.expr.query-731`: query execution failed: Parse error: invalid resolved function or parameter name: something.pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..48
- `age.expr.query-732`: expected projection_items at byte 15..15
- `age.expr.query-733`: query execution failed: Parse error: invalid resolved function or parameter name: contains.age_sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
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
- `age.expr.query-766`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..125
- `age.expr.query-767`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..43
- `age.expr.query-768`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..43
- `age.expr.query-769`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-770`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-771`: query execution failed: Parse error: no such function: percentileCont; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-772`: query execution failed: Parse error: no such function: percentileDisc; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.expr.query-778`: expected primary_expression at byte 24..24
- `age.expr.query-779`: query execution failed: Parse error: aggregate calls without exactly one argument is not supported in the initial graph slice at byte 7..16; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-793`: query execution failed: Parse error: unknown variable `x` at byte 34..35; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..71
- `age.expr.query-794`: query execution failed: Parse error: unknown variable `x` at byte 36..37; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..73
- `age.expr.query-798`: query execution failed: Parse error: unknown variable `x` at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..61
- `age.expr.query-799`: query execution failed: Parse error: unknown variable `x` at byte 19..20; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..56
- `age.expr.query-816`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..80
- `age.expr.query-817`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..84
- `age.expr.query-818`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..85
- `age.expr.query-819`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..100
- `age.expr.query-820`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 31..100
- `age.expr.query-830`: query execution failed: Parse error: boolean operators on non-boolean operands is not supported in the initial graph slice at byte 38..39; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..45
- `age.expr.query-852`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..120
- `age.expr.query-853`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..113
- `age.expr.query-854`: query execution failed: Parse error: generated relational SQL did not parse: near "q": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..123
- `age.expr.query-855`: query execution failed: Parse error: star arguments outside aggregating projections is not supported in the initial graph slice at byte 47..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 12..83
- `age.expr.query-863`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 10..33; mutation execution failed: Cypher mutation binding failed: RETURN * after mutation clauses is not supported in the initial graph slice at byte 33..41
- `age.expr.query-867`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..72
- `age.expr.query-868`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.sqrt; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..72
- `age.expr.query-889`: query execution failed: Parse error: malformed JSON; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..21
- `age.expr.query-896`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `age.expr.query-897`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.expr.query-898`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..32
- `age.expr.query-899`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..25
- `age.expr.query-900`: query execution failed: Parse error: no such function: nodes; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..32
- `age.expr.query-904`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..26
- `age.expr.query-905`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.expr.query-906`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..40
- `age.expr.query-907`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..33
- `age.expr.query-908`: query execution failed: Parse error: no such function: relationships; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..40
- `age.expr.query-920`: query execution failed: Extension error: Invalid Argument; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..27
- `age.expr.query-922`: query execution failed: Parse error: range over non-integer arguments is not supported in the initial graph slice at byte 7..28; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..28
- `age.expr.query-928`: query execution failed: Parse error: unknown variable `abc` at byte 12..15; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..16
- `age.expr.query-929`: query execution failed: Parse error: no such function: tail; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..13
- `age.expr.query-936`: query execution failed: Parse error: no such function: labels; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..23
- `age.expr.query-951`: query execution failed: Parse error: invalid resolved function or parameter name: pg_catalog.pg_typeof; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..53
- `age.expr.query-1016`: expected MATCH or pattern_element at byte 35..35
- `age.expr.query-1017`: expected MATCH or pattern_element at byte 29..29
- `age.expr.query-1018`: expected MATCH or pattern_element at byte 35..35
- `age.expr.query-1019`: expected MATCH or pattern_element at byte 26..26
- `age.expr.query-1020`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 0..93; mutation execution failed: Cypher mutation binding failed: unknown variable `x` at byte 65..67
- `age.expr.query-1029`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.expr.query-1030`: query execution failed: Parse error: no such function: end_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.expr.query-1034`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.expr.query-1037`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.expr.query-1049`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.expr.query-1054`: query execution failed: Parse error: no such function: start_id; mutation execution failed: Cypher parse failed: expected clause at byte 0..0
- `age.jsonb.operators.query-5`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-6`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-10`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-11`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-19`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..34
- `age.jsonb.operators.query-20`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.jsonb.operators.query-21`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..34
- `age.jsonb.operators.query-22`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.jsonb.operators.query-28`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-29`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..34
- `age.jsonb.operators.query-34`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-35`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-36`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-37`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-38`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-39`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-40`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-42`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.jsonb.operators.query-43`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..34
- `age.jsonb.operators.query-48`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..31
- `age.jsonb.operators.query-49`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..44
- `age.jsonb.operators.query-50`: query execution failed: Parse error: property access requires a node or relationship at byte 24..25; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 10..39
- `age.jsonb.operators.query-52`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-53`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-54`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-55`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-56`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-57`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-58`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-120`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-121`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-122`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-123`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-124`: query succeeded but AGE expects an error
- `age.jsonb.operators.query-142`: query succeeded but AGE expects an error
- `age.list.comprehension.query-69`: query succeeded but AGE expects an error
- `age.list.comprehension.query-76`: query execution failed: Parse error: property access requires a node or relationship at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 23..54
- `age.list.comprehension.query-77`: query execution failed: Parse error: property access requires a node or relationship at byte 60..61; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 42..67
- `age.list.comprehension.query-88`: query execution failed: Parse error: unknown variable `i` at byte 30..31; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..31
- `age.list.comprehension.query-89`: query execution failed: Parse error: unknown variable `i` at byte 47..48; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..48
- `age.list.comprehension.query-91`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 29..29
- `age.list.comprehension.query-92`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-93`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-96`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 23..23
- `age.list.comprehension.query-97`: expected EOI, WHERE, ORDER, SKIP, LIMIT, UNION, or clause at byte 16..16
- `age.list.comprehension.query-109`: query succeeded but AGE expects an error
- `age.list.comprehension.query-121`: expected not_expression at byte 28..28
- `age.map.projection.query-2`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-3`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-4`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-5`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 59..59
- `age.map.projection.query-6`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-7`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-8`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-9`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-10`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 59..59
- `age.map.projection.query-11`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `age.map.projection.query-12`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 28..28
- `age.map.projection.query-13`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 25..25
- `age.map.projection.query-14`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 35..35
- `age.map.projection.query-18`: expected AND, OR, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 61..61
- `age.pattern.expression.query-15`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 36..60; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..96
- `age.pattern.expression.query-16`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 36..59; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 21..109
- `age.pattern.expression.query-19`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 21..67; mutation execution failed: Cypher mutation binding failed: patterns in a SET value expression is not supported in the initial graph slice at byte 39..67
- `age.pattern.expression.query-20`: query execution failed: Parse error: pattern expressions in projections is not supported in the initial graph slice at byte 42..66; mutation execution failed: Cypher mutation binding failed: pattern expressions in projections is not supported in the initial graph slice at byte 42..66
- `age.pgvector.query-12`: query succeeded but AGE expects an error
- `age.pgvector.query-13`: query succeeded but AGE expects an error
- `age.pgvector.query-14`: query succeeded but AGE expects an error
- `age.pgvector.query-21`: query execution failed: Parse error: no such function: l1_distance; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.pgvector.query-22`: query execution failed: Parse error: no such function: vector_dims; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.pgvector.query-23`: query execution failed: Parse error: no such function: vector_norm; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..29
- `age.pgvector.query-24`: query execution failed: Parse error: no such function: l2_normalize; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..30
- `age.pgvector.query-25`: query execution failed: Parse error: no such function: l2_normalize; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..36
- `age.pgvector.query-26`: query execution failed: Parse error: no such function: subvector; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..39
- `age.pgvector.query-27`: query execution failed: Parse error: no such function: subvector; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..45
- `age.pgvector.query-28`: query execution failed: Parse error: no such function: binary_quantize; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..33
- `age.pgvector.query-30`: query execution failed: Extension error: TypeError: invalid operand types for -; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..40
- `age.pgvector.query-55`: query execution failed: Parse error: no such function: vector_dims; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 16..56
- `age.pgvector.query-65`: call target is not a plain or namespaced function name at byte 111..133
- `age.pgvector.query-66`: call target is not a plain or namespaced function name at byte 56..78
- `age.pgvector.query-67`: call target is not a plain or namespaced function name at byte 56..78
- `age.pgvector.query-70`: call target is not a plain or namespaced function name at byte 56..78
- `age.pgvector.query-71`: call target is not a plain or namespaced function name at byte 56..78
- `age.predicate.functions.query-51`: query execution failed: Parse error: property access requires a node or relationship at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..77
- `age.predicate.functions.query-52`: query execution failed: Parse error: property access requires a node or relationship at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..80
- `age.predicate.functions.query-53`: query execution failed: Parse error: property access requires a node or relationship at byte 66..67; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..78
- `age.predicate.functions.query-54`: query execution failed: Parse error: property access requires a node or relationship at byte 61..62; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..77
- `age.predicate.functions.query-55`: query execution failed: Parse error: property access requires a node or relationship at byte 62..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..81
- `age.predicate.functions.query-56`: query execution failed: Parse error: property access requires a node or relationship at byte 62..63; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..78
- `age.predicate.functions.query-57`: query execution failed: Parse error: property access requires a node or relationship at byte 64..65; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..79
- `age.predicate.functions.query-58`: query execution failed: Parse error: property access requires a node or relationship at byte 69..70; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 36..81
- `age.predicate.functions.query-61`: query execution failed: Parse error: outer list-scope variables inside nested list scopes is not supported in the initial graph slice at byte 58..59; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..61
- `age.reserved.keyword.alias.query-23`: query succeeded but AGE expects an error
- `age.reserved.keyword.alias.query-24`: query succeeded but AGE expects an error
- `age.reserved.keyword.alias.query-25`: query succeeded but AGE expects an error
- `age.reserved.keyword.alias.query-26`: query succeeded but AGE expects an error
- `age.reserved.keyword.alias.query-27`: query succeeded but AGE expects an error
- `age.reserved.keyword.alias.query-31`: query succeeded but AGE expects an error
- `age.scan.query-10`: integer literal is outside the supported i64 range at byte 7..28
- `age.scan.query-11`: query succeeded but AGE expects an error
- `age.scan.query-12`: query succeeded but AGE expects an error
- `age.scan.query-18`: integer literal is outside the supported i64 range at byte 26..44
- `age.scan.query-22`: expected identifier at byte 10..10
- `age.scan.query-29`: unsupported escape `\/` in string literal at byte 39..65
- `age.scan.query-36`: unsupported escape `\U` in string literal at byte 65..128
- `age.scan.query-42`: unsupported escape `\U` in string literal at byte 7..19
- `age.scan.query-49`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 8..8
- `age.scan.query-50`: query execution failed: Parse error: unknown variable `A` at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..8
- `age.scan.query-51`: query execution failed: Parse error: unknown variable `z` at byte 7..8; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..8
- `age.scan.query-52`: query execution failed: Parse error: unknown variable `$` at byte 7..10; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.scan.query-53`: query execution failed: Parse error: unknown variable `0` at byte 7..10; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..10
- `age.scan.query-54`: expected EOI, ORDER, SKIP, LIMIT, AS, AND, OR, UNION, clause, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 9..9
- `age.security.query-47`: query execution failed: Parse error: property access requires a node or relationship at byte 34..46; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 27..68
- `age.security.query-133`: query execution failed: Parse error: property access requires a node or relationship at byte 84..96; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 77..118

## Latest `cqlite-deep` run

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Commit: `8e296519275bbca3b006b64032b9776d76de5037` (dirty)
- Package: `0.8.0-pre.1`
- Semantics: v3
- Environment: `macos/aarch64` (`release`)
- Records: 124
- Passed: 113
- Unsupported: 0
- Failed or changed: 11

### Outcome changes from `20260726T225523.619934Z-f5c009b2f8e2-corpus-deep`

- No outcome changes.

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `cqlite.basic-queries.run-a-to-b.query-1` | `Conformance` | basic_queries | `Passed` | 1.867 ms |
| `cqlite.basic-queries.run-a-to-b.query-2` | `Conformance` | basic_queries | `Passed` | 1.676 ms |
| `cqlite.basic-queries.run-a-to-b.query-3` | `Conformance` | basic_queries | `Passed` | 1.755 ms |
| `cqlite.basic-queries.run-a-edge-b.query-1` | `Conformance` | basic_queries | `Passed` | 2.276 ms |
| `cqlite.basic-queries.run-a-edge-b.query-2` | `Conformance` | basic_queries | `Passed` | 1.687 ms |
| `cqlite.basic-queries.run-a-to-a.query-1` | `Conformance` | basic_queries | `Passed` | 2.289 ms |
| `cqlite.basic-queries.run-a-to-a.query-2` | `Conformance` | basic_queries | `Passed` | 1.522 ms |
| `cqlite.basic-queries.run-a-edge-a.query-1` | `Conformance` | basic_queries | `Passed` | 2.358 ms |
| `cqlite.basic-queries.run-a-edge-a.query-2` | `Conformance` | basic_queries | `Passed` | 1.519 ms |
| `cqlite.basic-queries.run-a-knows-b.query-1` | `Conformance` | basic_queries | `Passed` | 2.316 ms |
| `cqlite.basic-queries.run-a-knows-b.query-2` | `Conformance` | basic_queries | `Passed` | 1.596 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-1` | `Conformance` | basic_queries | `Passed` | 2.277 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-2` | `Conformance` | basic_queries | `Passed` | 1.729 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-1` | `Conformance` | basic_queries | `Passed` | 2.146 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-2` | `Conformance` | basic_queries | `Passed` | 1.721 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-1` | `Conformance` | basic_queries | `Passed` | 1.896 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-2` | `Conformance` | basic_queries | `Passed` | 1.500 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-1` | `Conformance` | basic_queries | `Passed` | 2.105 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-2` | `Conformance` | basic_queries | `Failed` | 1.509 ms |
| `cqlite.basic-queries.run-set.query-1` | `Conformance` | basic_queries | `Passed` | 1.685 ms |
| `cqlite.basic-queries.run-set.query-2` | `Conformance` | basic_queries | `Failed` | 1.558 ms |
| `cqlite.basic-queries.run-set.query-3` | `Conformance` | basic_queries | `Passed` | 1.582 ms |
| `cqlite.basic-queries.return-from-set.query-1` | `Conformance` | basic_queries | `Passed` | 1.708 ms |
| `cqlite.basic-queries.return-from-set.query-2` | `Conformance` | basic_queries | `Passed` | 1.551 ms |
| `cqlite.basic-queries.return-from-set.query-3` | `Conformance` | basic_queries | `Passed` | 1.583 ms |
| `cqlite.basic-queries.run-delete-node.query-1` | `Conformance` | basic_queries | `Passed` | 1.687 ms |
| `cqlite.basic-queries.run-delete-node.query-2` | `Conformance` | basic_queries | `Passed` | 1.527 ms |
| `cqlite.basic-queries.run-delete-node.query-3` | `Conformance` | basic_queries | `Passed` | 1.494 ms |
| `cqlite.basic-queries.run-delete-edge.query-1` | `Conformance` | basic_queries | `Passed` | 1.901 ms |
| `cqlite.basic-queries.run-delete-edge.query-2` | `Conformance` | basic_queries | `Passed` | 1.529 ms |
| `cqlite.basic-queries.run-delete-edge.query-3` | `Conformance` | basic_queries | `Passed` | 1.488 ms |
| `cqlite.basic-queries.run-bad-delete.query-1` | `Conformance` | basic_queries | `Passed` | 1.862 ms |
| `cqlite.basic-queries.run-bad-delete.query-2` | `Conformance` | basic_queries | `Passed` | 1.492 ms |
| `cqlite.basic-queries.run-return-label.query-1` | `Conformance` | basic_queries | `Passed` | 2.026 ms |
| `cqlite.basic-queries.run-return-label.query-2` | `Conformance` | basic_queries | `Failed` | 1.589 ms |
| `cqlite.basic-queries.match-return-count.query-1` | `Conformance` | basic_queries | `Passed` | 1.509 ms |
| `cqlite.basic-queries.match-return-count.query-2` | `Conformance` | basic_queries | `Passed` | 1.543 ms |
| `cqlite.basic-queries.match-return-count.query-3` | `Conformance` | basic_queries | `Passed` | 1.490 ms |
| `cqlite.basic-queries.match-multiple-edges.query-1` | `Conformance` | basic_queries | `Passed` | 2.491 ms |
| `cqlite.create-queries.create-label-only.query-1` | `Conformance` | create_queries | `Passed` | 1.752 ms |
| `cqlite.create-queries.create-label-only.query-2` | `Conformance` | create_queries | `Passed` | 1.484 ms |
| `cqlite.create-queries.create-with-properties.query-1` | `Conformance` | create_queries | `Passed` | 1.808 ms |
| `cqlite.create-queries.create-with-properties.query-2` | `Conformance` | create_queries | `Passed` | 1.569 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-1` | `Conformance` | create_queries | `Failed` | 1.592 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-2` | `Conformance` | create_queries | `Passed` | 1.622 ms |
| `cqlite.create-queries.create-edges-with-label.query-1` | `Conformance` | create_queries | `Passed` | 2.259 ms |
| `cqlite.create-queries.create-edges-with-label.query-2` | `Conformance` | create_queries | `Failed` | 1.723 ms |
| `cqlite.delete-queries.delete-node.query-1` | `Conformance` | delete_queries | `Passed` | 1.767 ms |
| `cqlite.delete-queries.delete-node.query-2` | `Conformance` | delete_queries | `Passed` | 1.513 ms |
| `cqlite.delete-queries.delete-node.query-3` | `Conformance` | delete_queries | `Passed` | 1.525 ms |
| `cqlite.delete-queries.delete-node.query-4` | `Conformance` | delete_queries | `Passed` | 1.572 ms |
| `cqlite.delete-queries.double-delete-node.query-1` | `Conformance` | delete_queries | `Passed` | 1.692 ms |
| `cqlite.delete-queries.double-delete-node.query-2` | `Conformance` | delete_queries | `Passed` | 1.488 ms |
| `cqlite.delete-queries.double-delete-node.query-3` | `Conformance` | delete_queries | `Passed` | 1.512 ms |
| `cqlite.delete-queries.double-delete-node.query-4` | `Conformance` | delete_queries | `Passed` | 1.509 ms |
| `cqlite.delete-queries.delete-edge.query-1` | `Conformance` | delete_queries | `Passed` | 2.150 ms |
| `cqlite.delete-queries.delete-edge.query-3` | `Conformance` | delete_queries | `Passed` | 1.672 ms |
| `cqlite.delete-queries.connected-delete-fails.query-1` | `Conformance` | delete_queries | `Passed` | 2.132 ms |
| `cqlite.delete-queries.connected-delete-fails.query-2` | `Conformance` | delete_queries | `Passed` | 1.484 ms |
| `cqlite.delete-queries.connected-delete-fails.query-3` | `Conformance` | delete_queries | `Passed` | 1.502 ms |
| `cqlite.delete-queries.connected-delete-fails.query-4` | `Conformance` | delete_queries | `Passed` | 1.412 ms |
| `cqlite.match-queries.create-test-graph.query-1` | `Conformance` | match_queries | `Passed` | 3.205 ms |
| `cqlite.match-queries.match-all-nodes.query-1` | `Conformance` | match_queries | `Passed` | 1.536 ms |
| `cqlite.match-queries.match-multiple-nodes.query-1` | `Conformance` | match_queries | `Passed` | 1.558 ms |
| `cqlite.match-queries.match-multiple-nodes.query-2` | `Conformance` | match_queries | `Passed` | 1.579 ms |
| `cqlite.match-queries.match-single-directed-edge.query-1` | `Conformance` | match_queries | `Passed` | 1.655 ms |
| `cqlite.match-queries.match-single-undirected-edge.query-1` | `Conformance` | match_queries | `Passed` | 1.572 ms |
| `cqlite.match-queries.match-labeled-nodes.query-1` | `Conformance` | match_queries | `Passed` | 1.500 ms |
| `cqlite.match-queries.match-labeled-nodes.query-2` | `Conformance` | match_queries | `Passed` | 1.540 ms |
| `cqlite.match-queries.match-labeled-nodes.query-3` | `Conformance` | match_queries | `Passed` | 1.495 ms |
| `cqlite.match-queries.match-labeled-edges.query-1` | `Conformance` | match_queries | `Passed` | 1.682 ms |
| `cqlite.match-queries.match-labeled-edges.query-2` | `Conformance` | match_queries | `Passed` | 1.607 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-1` | `Conformance` | match_queries | `Passed` | 1.508 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-2` | `Conformance` | match_queries | `Passed` | 1.542 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-3` | `Conformance` | match_queries | `Passed` | 1.603 ms |
| `cqlite.match-queries.match-edges-with-properties.query-1` | `Conformance` | match_queries | `Passed` | 1.639 ms |
| `cqlite.match-queries.match-nodes-with-label.query-1` | `Conformance` | match_queries | `Passed` | 1.498 ms |
| `cqlite.match-queries-where.create-test-graph.query-1` | `Conformance` | match_queries_where | `Passed` | 3.976 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 1.592 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-2` | `Conformance` | match_queries_where | `Failed` | 1.498 ms |
| `cqlite.match-queries-where.match-where-node-id-eq-non-id.query-1` | `Conformance` | match_queries_where | `Passed` | 1.529 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-1` | `Conformance` | match_queries_where | `Passed` | 1.534 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-2` | `Conformance` | match_queries_where | `Passed` | 1.540 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 1.511 ms |
| `cqlite.match-queries-where.match-where-node-prop.query-1` | `Conformance` | match_queries_where | `Passed` | 1.540 ms |
| `cqlite.match-queries-where.match-where-not-node-prop.query-1` | `Conformance` | match_queries_where | `Passed` | 1.461 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-1` | `Conformance` | match_queries_where | `Passed` | 1.554 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-2` | `Conformance` | match_queries_where | `Passed` | 1.673 ms |
| `cqlite.match-queries-where.match-where-node-prop-ne-null.query-1` | `Conformance` | match_queries_where | `Passed` | 1.546 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-1` | `Conformance` | match_queries_where | `Passed` | 1.625 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-2` | `Conformance` | match_queries_where | `Passed` | 1.618 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-3` | `Conformance` | match_queries_where | `Passed` | 1.562 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-1` | `Conformance` | match_queries_where | `Failed` | 1.549 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-2` | `Conformance` | match_queries_where | `Failed` | 1.560 ms |
| `cqlite.match-queries-where.match-where-edge-prop-eq.query-1` | `Conformance` | match_queries_where | `Passed` | 1.760 ms |
| `cqlite.match-queries-where.match-where-edge-prop-gt.query-1` | `Conformance` | match_queries_where | `Passed` | 1.758 ms |
| `cqlite.match-queries-where.match-where-a-or-b.query-1` | `Conformance` | match_queries_where | `Passed` | 1.679 ms |
| `cqlite.return-queries.return-parameter.query-1` | `Conformance` | return_queries | `Failed` | 1.521 ms |
| `cqlite.return-queries.return-id-of.query-1` | `Conformance` | return_queries | `Passed` | 2.071 ms |
| `cqlite.return-queries.return-id-of.query-2` | `Conformance` | return_queries | `Passed` | 1.478 ms |
| `cqlite.return-queries.return-label-of.query-1` | `Conformance` | return_queries | `Passed` | 2.453 ms |
| `cqlite.return-queries.return-label-of.query-2` | `Conformance` | return_queries | `Passed` | 1.526 ms |
| `cqlite.return-queries.create-and-return.query-1` | `Conformance` | return_queries | `Passed` | 1.893 ms |
| `cqlite.return-queries.create-and-return.query-2` | `Conformance` | return_queries | `Passed` | 1.696 ms |
| `cqlite.return-queries.set-and-return.query-1` | `Conformance` | return_queries | `Passed` | 1.828 ms |
| `cqlite.return-queries.set-and-return.query-2` | `Conformance` | return_queries | `Passed` | 1.539 ms |
| `cqlite.return-queries.delete-and-return.query-1` | `Conformance` | return_queries | `Passed` | 1.717 ms |
| `cqlite.return-queries.delete-and-return.query-2` | `Conformance` | return_queries | `Passed` | 1.518 ms |
| `cqlite.return-queries.return-out-of-bounds.query-1` | `Conformance` | return_queries | `Passed` | 1.536 ms |
| `cqlite.set-queries.set-once.query-1` | `Conformance` | set_queries | `Passed` | 1.739 ms |
| `cqlite.set-queries.set-once.query-2` | `Conformance` | set_queries | `Passed` | 1.552 ms |
| `cqlite.set-queries.set-once.query-3` | `Conformance` | set_queries | `Passed` | 1.415 ms |
| `cqlite.set-queries.set-after-create.query-1` | `Conformance` | set_queries | `Passed` | 1.839 ms |
| `cqlite.set-queries.set-after-create.query-2` | `Conformance` | set_queries | `Passed` | 1.464 ms |
| `cqlite.set-queries.set-multiple-times.query-1` | `Conformance` | set_queries | `Passed` | 2.022 ms |
| `cqlite.set-queries.set-multiple-times.query-2` | `Conformance` | set_queries | `Passed` | 1.546 ms |
| `cqlite.set-queries.delete-property.query-1` | `Conformance` | set_queries | `Passed` | 1.755 ms |
| `cqlite.set-queries.delete-property.query-2` | `Conformance` | set_queries | `Passed` | 1.484 ms |
| `cqlite.set-queries.delete-property.query-3` | `Conformance` | set_queries | `Passed` | 1.487 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-1` | `Conformance` | txn_semantics | `Passed` | 1.707 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-2` | `Conformance` | txn_semantics | `Passed` | 1.568 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-3` | `Conformance` | txn_semantics | `Passed` | 1.754 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-4` | `Conformance` | txn_semantics | `Passed` | 1.516 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-5` | `Conformance` | txn_semantics | `Passed` | 1.588 ms |

## Latest `deep` run

- Run: `20260718T013941.952713Z-e1d73880b749-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Semantics: v0
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

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Commit: `8e296519275bbca3b006b64032b9776d76de5037` (dirty)
- Package: `0.8.0-pre.1`
- Semantics: v3
- Environment: `macos/aarch64` (`release`)
- Records: 372
- Passed: 277
- Unsupported: 0
- Failed or changed: 95

### Outcome changes from `20260726T225523.619934Z-f5c009b2f8e2-corpus-deep`

- No outcome changes.

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `grafeo.spec.common.null.semantics.negative.limit.returns.empty.cypher.cypher-variant` | `Conformance` | common | `Failed` | 1.520 ms |
| `grafeo.spec.common.numeric.edge.cases.min.int64.cypher.cypher-variant` | `Conformance` | common | `Failed` | 1.485 ms |
| `grafeo.spec.common.numeric.edge.cases.nan.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Failed` | 1.469 ms |
| `grafeo.spec.common.numeric.edge.cases.inf.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Failed` | 1.460 ms |
| `grafeo.spec.lpg.cypher.admin.explain.match` | `Conformance` | lpg | `Failed` | 1.846 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.basic` | `Conformance` | lpg | `Failed` | 0.014 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.filter` | `Conformance` | lpg | `Failed` | 0.009 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.size` | `Conformance` | lpg | `Failed` | 0.015 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.property.extraction` | `Conformance` | lpg | `Failed` | 0.008 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.basic` | `Conformance` | lpg | `Passed` | 14.587 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.transform` | `Conformance` | lpg | `Passed` | 15.634 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.filter.and.transform` | `Conformance` | lpg | `Failed` | 14.115 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.nested` | `Conformance` | lpg | `Failed` | 14.888 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.exists.subquery.actors.with.action.movies` | `Conformance` | lpg | `Passed` | 14.456 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.not.exists.subquery` | `Conformance` | lpg | `Failed` | 15.027 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.movies.per.actor` | `Conformance` | lpg | `Passed` | 14.300 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.prolific.directors` | `Conformance` | lpg | `Passed` | 14.712 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.basic` | `Conformance` | lpg | `Failed` | 14.065 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.with.aggregation` | `Conformance` | lpg | `Failed` | 13.929 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.set.property` | `Conformance` | lpg | `Failed` | 15.817 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.create.relationships` | `Conformance` | lpg | `Failed` | 0.027 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.actor.collaboration.via.comprehension` | `Conformance` | lpg | `Failed` | 0.012 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.genre.diversity.per.actor` | `Conformance` | lpg | `Failed` | 0.015 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.allows.distinct.values` | `Conformance` | lpg | `Failed` | 1.673 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.violation` | `Conformance` | lpg | `Failed` | 1.662 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.null.allowed` | `Conformance` | lpg | `Failed` | 1.621 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.satisfied` | `Conformance` | lpg | `Failed` | 1.524 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation` | `Conformance` | lpg | `Failed` | 1.624 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation.on.set` | `Conformance` | lpg | `Failed` | 1.500 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.allows.different.combinations` | `Conformance` | lpg | `Failed` | 1.519 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.duplicate` | `Conformance` | lpg | `Failed` | 1.641 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.missing.property` | `Conformance` | lpg | `Failed` | 1.542 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.nonexistent.constraint` | `Conformance` | lpg | `Passed` | 0.002 ms |
| `grafeo.spec.lpg.cypher.expressions.addition` | `Conformance` | lpg | `Passed` | 1.864 ms |
| `grafeo.spec.lpg.cypher.expressions.subtraction` | `Conformance` | lpg | `Passed` | 1.930 ms |
| `grafeo.spec.lpg.cypher.expressions.multiplication` | `Conformance` | lpg | `Passed` | 1.894 ms |
| `grafeo.spec.lpg.cypher.expressions.division` | `Conformance` | lpg | `Passed` | 1.838 ms |
| `grafeo.spec.lpg.cypher.expressions.modulo` | `Conformance` | lpg | `Passed` | 1.909 ms |
| `grafeo.spec.lpg.cypher.expressions.power` | `Conformance` | lpg | `Passed` | 1.892 ms |
| `grafeo.spec.lpg.cypher.expressions.unary.minus` | `Conformance` | lpg | `Failed` | 0.006 ms |
| `grafeo.spec.lpg.cypher.expressions.string.concat` | `Conformance` | lpg | `Passed` | 1.863 ms |
| `grafeo.spec.lpg.cypher.expressions.equals` | `Conformance` | lpg | `Passed` | 1.815 ms |
| `grafeo.spec.lpg.cypher.expressions.not.equals` | `Conformance` | lpg | `Passed` | 2.014 ms |
| `grafeo.spec.lpg.cypher.expressions.less.than` | `Conformance` | lpg | `Passed` | 2.061 ms |
| `grafeo.spec.lpg.cypher.expressions.greater.equal` | `Conformance` | lpg | `Passed` | 2.075 ms |
| `grafeo.spec.lpg.cypher.expressions.starts.with` | `Conformance` | lpg | `Passed` | 2.114 ms |
| `grafeo.spec.lpg.cypher.expressions.ends.with` | `Conformance` | lpg | `Passed` | 2.061 ms |
| `grafeo.spec.lpg.cypher.expressions.contains` | `Conformance` | lpg | `Passed` | 2.127 ms |
| `grafeo.spec.lpg.cypher.expressions.in.list` | `Conformance` | lpg | `Passed` | 2.398 ms |
| `grafeo.spec.lpg.cypher.expressions.regex.match` | `Conformance` | lpg | `Failed` | 0.008 ms |
| `grafeo.spec.lpg.cypher.expressions.is.null` | `Conformance` | lpg | `Passed` | 2.105 ms |
| `grafeo.spec.lpg.cypher.expressions.is.not.null` | `Conformance` | lpg | `Passed` | 2.180 ms |
| `grafeo.spec.lpg.cypher.expressions.case.simple` | `Conformance` | lpg | `Passed` | 1.920 ms |
| `grafeo.spec.lpg.cypher.expressions.case.searched` | `Conformance` | lpg | `Passed` | 2.012 ms |
| `grafeo.spec.lpg.cypher.expressions.list.literal` | `Conformance` | lpg | `Failed` | 1.854 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension` | `Conformance` | lpg | `Failed` | 1.870 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension.filter.only` | `Conformance` | lpg | `Failed` | 1.839 ms |
| `grafeo.spec.lpg.cypher.expressions.list.slice` | `Conformance` | lpg | `Failed` | 1.897 ms |
| `grafeo.spec.lpg.cypher.expressions.index.access` | `Conformance` | lpg | `Passed` | 1.870 ms |
| `grafeo.spec.lpg.cypher.expressions.coalesce` | `Conformance` | lpg | `Passed` | 1.873 ms |
| `grafeo.spec.lpg.cypher.expressions.reduce` | `Conformance` | lpg | `Passed` | 2.267 ms |
| `grafeo.spec.lpg.cypher.expressions.all.predicate` | `Conformance` | lpg | `Passed` | 1.931 ms |
| `grafeo.spec.lpg.cypher.expressions.any.predicate` | `Conformance` | lpg | `Passed` | 1.891 ms |
| `grafeo.spec.lpg.cypher.expressions.none.predicate` | `Conformance` | lpg | `Passed` | 1.920 ms |
| `grafeo.spec.lpg.cypher.expressions.single.predicate` | `Conformance` | lpg | `Passed` | 1.916 ms |
| `grafeo.spec.lpg.cypher.expressions.any.with.labels.in.where` | `Conformance` | lpg | `Passed` | 2.440 ms |
| `grafeo.spec.lpg.cypher.expressions.comparison.in.return` | `Conformance` | lpg | `Passed` | 1.899 ms |
| `grafeo.spec.lpg.cypher.expressions.aggregate.comparison.in.return` | `Conformance` | lpg | `Passed` | 1.858 ms |
| `grafeo.spec.lpg.cypher.functions.id.of.node` | `Conformance` | lpg | `Passed` | 1.819 ms |
| `grafeo.spec.lpg.cypher.functions.labels.single` | `Conformance` | lpg | `Failed` | 1.867 ms |
| `grafeo.spec.lpg.cypher.functions.labels.multiple` | `Conformance` | lpg | `Passed` | 2.068 ms |
| `grafeo.spec.lpg.cypher.functions.type.of.relationship` | `Conformance` | lpg | `Passed` | 2.292 ms |
| `grafeo.spec.lpg.cypher.functions.keys.of.node` | `Conformance` | lpg | `Passed` | 2.122 ms |
| `grafeo.spec.lpg.cypher.functions.properties.of.node` | `Conformance` | lpg | `Passed` | 1.891 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.true` | `Conformance` | lpg | `Failed` | 1.800 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.false` | `Conformance` | lpg | `Failed` | 1.753 ms |
| `grafeo.spec.lpg.cypher.functions.head.of.list` | `Conformance` | lpg | `Passed` | 1.921 ms |
| `grafeo.spec.lpg.cypher.functions.last.of.list` | `Conformance` | lpg | `Passed` | 1.867 ms |
| `grafeo.spec.lpg.cypher.functions.tail.of.list` | `Conformance` | lpg | `Failed` | 1.891 ms |
| `grafeo.spec.lpg.cypher.functions.range.default.step` | `Conformance` | lpg | `Failed` | 1.875 ms |
| `grafeo.spec.lpg.cypher.functions.range.with.step` | `Conformance` | lpg | `Failed` | 1.887 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.list` | `Conformance` | lpg | `Passed` | 1.873 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.string` | `Conformance` | lpg | `Passed` | 1.865 ms |
| `grafeo.spec.lpg.cypher.functions.to.lower` | `Conformance` | lpg | `Passed` | 1.869 ms |
| `grafeo.spec.lpg.cypher.functions.to.upper` | `Conformance` | lpg | `Passed` | 1.810 ms |
| `grafeo.spec.lpg.cypher.functions.trim.whitespace` | `Conformance` | lpg | `Passed` | 1.836 ms |
| `grafeo.spec.lpg.cypher.functions.replace.substring` | `Conformance` | lpg | `Passed` | 1.853 ms |
| `grafeo.spec.lpg.cypher.functions.substring.from.start` | `Conformance` | lpg | `Failed` | 1.813 ms |
| `grafeo.spec.lpg.cypher.functions.substring.to.end` | `Conformance` | lpg | `Failed` | 1.872 ms |
| `grafeo.spec.lpg.cypher.functions.split.string` | `Conformance` | lpg | `Failed` | 1.849 ms |
| `grafeo.spec.lpg.cypher.functions.left.string` | `Conformance` | lpg | `Passed` | 1.856 ms |
| `grafeo.spec.lpg.cypher.functions.right.string` | `Conformance` | lpg | `Passed` | 1.869 ms |
| `grafeo.spec.lpg.cypher.functions.reverse.string` | `Conformance` | lpg | `Passed` | 1.848 ms |
| `grafeo.spec.lpg.cypher.functions.abs.positive` | `Conformance` | lpg | `Passed` | 1.858 ms |
| `grafeo.spec.lpg.cypher.functions.ceil.float` | `Conformance` | lpg | `Passed` | 1.925 ms |
| `grafeo.spec.lpg.cypher.functions.floor.float` | `Conformance` | lpg | `Passed` | 1.900 ms |
| `grafeo.spec.lpg.cypher.functions.round.float` | `Conformance` | lpg | `Passed` | 1.806 ms |
| `grafeo.spec.lpg.cypher.functions.sign.positive` | `Conformance` | lpg | `Passed` | 1.918 ms |
| `grafeo.spec.lpg.cypher.functions.sign.negative` | `Conformance` | lpg | `Passed` | 1.864 ms |
| `grafeo.spec.lpg.cypher.functions.sign.zero` | `Conformance` | lpg | `Passed` | 1.786 ms |
| `grafeo.spec.lpg.cypher.functions.sqrt.perfect.square` | `Conformance` | lpg | `Passed` | 1.793 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.string` | `Conformance` | lpg | `Passed` | 1.824 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.float` | `Conformance` | lpg | `Passed` | 1.821 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.string` | `Conformance` | lpg | `Passed` | 1.837 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.integer` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.functions.to.string.from.integer` | `Conformance` | lpg | `Passed` | 1.879 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.true` | `Conformance` | lpg | `Passed` | 1.865 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.false` | `Conformance` | lpg | `Passed` | 1.838 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.string` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.map` | `Conformance` | lpg | `Passed` | 1.843 ms |
| `grafeo.spec.lpg.cypher.functions.datetime.from.string` | `Conformance` | lpg | `Passed` | 1.802 ms |
| `grafeo.spec.lpg.cypher.functions.duration.from.string` | `Conformance` | lpg | `Passed` | 1.800 ms |
| `grafeo.spec.lpg.cypher.functions.path.length` | `Conformance` | lpg | `Passed` | 3.196 ms |
| `grafeo.spec.lpg.cypher.functions.path.length.single.hop` | `Conformance` | lpg | `Passed` | 2.309 ms |
| `grafeo.spec.lpg.cypher.functions.collect.names` | `Conformance` | lpg | `Passed` | 2.024 ms |
| `grafeo.spec.lpg.cypher.functions.collect.distinct` | `Conformance` | lpg | `Passed` | 2.288 ms |
| `grafeo.spec.lpg.cypher.functions.count.with.distinct` | `Conformance` | lpg | `Passed` | 2.229 ms |
| `grafeo.spec.lpg.cypher.functions.sum.values` | `Conformance` | lpg | `Passed` | 2.222 ms |
| `grafeo.spec.lpg.cypher.functions.avg.values` | `Conformance` | lpg | `Passed` | 2.233 ms |
| `grafeo.spec.lpg.cypher.functions.min.values` | `Conformance` | lpg | `Passed` | 2.230 ms |
| `grafeo.spec.lpg.cypher.functions.max.values` | `Conformance` | lpg | `Passed` | 2.247 ms |
| `grafeo.spec.lpg.cypher.functions.chained.string.functions` | `Conformance` | lpg | `Passed` | 1.871 ms |
| `grafeo.spec.lpg.cypher.functions.nested.list.functions` | `Conformance` | lpg | `Passed` | 1.892 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log.of.e` | `Conformance` | lpg | `Failed` | 1.837 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log10.of.100` | `Conformance` | lpg | `Passed` | 1.816 ms |
| `grafeo.spec.lpg.cypher.functions.extended.exp.of.zero` | `Conformance` | lpg | `Passed` | 1.793 ms |
| `grafeo.spec.lpg.cypher.functions.extended.e.constant` | `Conformance` | lpg | `Failed` | 1.815 ms |
| `grafeo.spec.lpg.cypher.functions.extended.pi.constant` | `Conformance` | lpg | `Passed` | 1.847 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rand.in.range` | `Conformance` | lpg | `Passed` | 1.823 ms |
| `grafeo.spec.lpg.cypher.functions.extended.sin.of.zero` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.functions.extended.cos.of.zero` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.functions.extended.tan.of.zero` | `Conformance` | lpg | `Passed` | 1.791 ms |
| `grafeo.spec.lpg.cypher.functions.extended.asin.of.one` | `Conformance` | lpg | `Passed` | 1.929 ms |
| `grafeo.spec.lpg.cypher.functions.extended.acos.of.one` | `Conformance` | lpg | `Passed` | 1.874 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan.of.one` | `Conformance` | lpg | `Passed` | 1.840 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan2.unit` | `Conformance` | lpg | `Passed` | 1.984 ms |
| `grafeo.spec.lpg.cypher.functions.extended.degrees.from.pi` | `Conformance` | lpg | `Passed` | 1.865 ms |
| `grafeo.spec.lpg.cypher.functions.extended.radians.from.180` | `Conformance` | lpg | `Passed` | 1.820 ms |
| `grafeo.spec.lpg.cypher.functions.extended.ltrim.whitespace` | `Conformance` | lpg | `Passed` | 1.827 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rtrim.whitespace` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.functions.extended.char.length.string` | `Conformance` | lpg | `Passed` | 1.831 ms |
| `grafeo.spec.lpg.cypher.functions.extended.length.of.string` | `Conformance` | lpg | `Passed` | 1.801 ms |
| `grafeo.spec.lpg.cypher.functions.extended.reverse.list` | `Conformance` | lpg | `Failed` | 1.843 ms |
| `grafeo.spec.lpg.cypher.functions.extended.keys.of.map` | `Conformance` | lpg | `Passed` | 1.923 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdev.sample` | `Conformance` | lpg | `Failed` | 3.156 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdevp.population` | `Conformance` | lpg | `Failed` | 3.192 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.cont.median` | `Conformance` | lpg | `Failed` | 2.647 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.disc.median` | `Conformance` | lpg | `Failed` | 2.567 ms |
| `grafeo.spec.lpg.cypher.functions.extended.element.id.not.null` | `Conformance` | lpg | `Failed` | 1.787 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.star` | `Conformance` | lpg | `Passed` | 1.955 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.expr` | `Conformance` | lpg | `Passed` | 2.035 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.path` | `Conformance` | lpg | `Passed` | 2.460 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.path` | `Conformance` | lpg | `Passed` | 2.454 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.multi.hop.path` | `Conformance` | lpg | `Passed` | 3.226 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.multi.hop.path` | `Conformance` | lpg | `Passed` | 3.241 ms |
| `grafeo.spec.lpg.cypher.functions.extended.date.no.args` | `Conformance` | lpg | `Passed` | 1.843 ms |
| `grafeo.spec.lpg.cypher.functions.extended.now.returns.value` | `Conformance` | lpg | `Failed` | 1.766 ms |
| `grafeo.spec.lpg.cypher.functions.extended.year.accessor` | `Conformance` | lpg | `Failed` | 1.800 ms |
| `grafeo.spec.lpg.cypher.functions.extended.month.accessor` | `Conformance` | lpg | `Failed` | 1.789 ms |
| `grafeo.spec.lpg.cypher.functions.extended.day.accessor` | `Conformance` | lpg | `Failed` | 1.809 ms |
| `grafeo.spec.lpg.cypher.functions.extended.time.from.string` | `Conformance` | lpg | `Passed` | 1.808 ms |
| `grafeo.spec.lpg.cypher.functions.extended.duration.from.map` | `Conformance` | lpg | `Passed` | 1.820 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.node` | `Conformance` | lpg | `Passed` | 1.820 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.binding` | `Conformance` | lpg | `Passed` | 1.823 ms |
| `grafeo.spec.lpg.cypher.patterns.single.label` | `Conformance` | lpg | `Passed` | 1.927 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.labels` | `Conformance` | lpg | `Passed` | 2.220 ms |
| `grafeo.spec.lpg.cypher.patterns.property.filter` | `Conformance` | lpg | `Passed` | 2.217 ms |
| `grafeo.spec.lpg.cypher.patterns.outgoing.relationship` | `Conformance` | lpg | `Passed` | 2.181 ms |
| `grafeo.spec.lpg.cypher.patterns.incoming.relationship` | `Conformance` | lpg | `Passed` | 2.221 ms |
| `grafeo.spec.lpg.cypher.patterns.undirected.relationship` | `Conformance` | lpg | `Passed` | 2.381 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.relationship.types` | `Conformance` | lpg | `Passed` | 2.855 ms |
| `grafeo.spec.lpg.cypher.patterns.relationship.properties` | `Conformance` | lpg | `Passed` | 2.337 ms |
| `grafeo.spec.lpg.cypher.patterns.untyped.relationship` | `Conformance` | lpg | `Passed` | 2.297 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.relationship` | `Conformance` | lpg | `Passed` | 2.211 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.unbounded` | `Conformance` | lpg | `Passed` | 2.926 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.exact` | `Conformance` | lpg | `Passed` | 3.428 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.range` | `Conformance` | lpg | `Passed` | 3.065 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.max.only` | `Conformance` | lpg | `Passed` | 3.060 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.min.only` | `Conformance` | lpg | `Passed` | 3.070 ms |
| `grafeo.spec.lpg.cypher.patterns.path.alias` | `Conformance` | lpg | `Passed` | 2.259 ms |
| `grafeo.spec.lpg.cypher.patterns.shortest.path` | `Conformance` | lpg | `Failed` | 0.004 ms |
| `grafeo.spec.lpg.cypher.patterns.all.shortest.paths` | `Conformance` | lpg | `Failed` | 0.002 ms |
| `grafeo.spec.lpg.cypher.patterns.pattern.comprehension` | `Conformance` | lpg | `Failed` | 0.011 ms |
| `grafeo.spec.lpg.cypher.patterns.exists.subquery` | `Conformance` | lpg | `Passed` | 2.586 ms |
| `grafeo.spec.lpg.cypher.patterns.not.exists` | `Conformance` | lpg | `Passed` | 2.250 ms |
| `grafeo.spec.lpg.cypher.patterns.count.subquery` | `Conformance` | lpg | `Passed` | 2.721 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.single.node` | `Conformance` | lpg | `Passed` | 2.023 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.label` | `Conformance` | lpg | `Passed` | 2.006 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.property` | `Conformance` | lpg | `Passed` | 2.093 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multi.label` | `Conformance` | lpg | `Passed` | 2.090 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.comma.patterns` | `Conformance` | lpg | `Passed` | 2.021 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multiple.clauses` | `Conformance` | lpg | `Passed` | 2.099 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.outgoing` | `Conformance` | lpg | `Passed` | 2.515 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.incoming` | `Conformance` | lpg | `Passed` | 2.288 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.undirected` | `Conformance` | lpg | `Passed` | 2.329 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.with.result` | `Conformance` | lpg | `Passed` | 2.310 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.null` | `Conformance` | lpg | `Passed` | 2.082 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.comparison` | `Conformance` | lpg | `Passed` | 2.056 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.and` | `Conformance` | lpg | `Passed` | 2.135 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.or` | `Conformance` | lpg | `Passed` | 2.227 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.not` | `Conformance` | lpg | `Passed` | 2.094 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.xor` | `Conformance` | lpg | `Passed` | 2.325 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.projection` | `Conformance` | lpg | `Passed` | 1.851 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.distinct` | `Conformance` | lpg | `Passed` | 2.069 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.where` | `Conformance` | lpg | `Passed` | 2.029 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.star` | `Conformance` | lpg | `Passed` | 1.754 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.list` | `Conformance` | lpg | `Passed` | 1.607 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.with.match` | `Conformance` | lpg | `Passed` | 2.075 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union` | `Conformance` | lpg | `Passed` | 2.154 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union.all` | `Conformance` | lpg | `Passed` | 1.858 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.labels` | `Conformance` | lpg | `Passed` | 1.943 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.relationship.types` | `Conformance` | lpg | `Passed` | 2.170 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.property.keys` | `Conformance` | lpg | `Passed` | 1.805 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.basic` | `Conformance` | lpg | `Passed` | 2.030 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.with.outer.scope` | `Conformance` | lpg | `Failed` | 2.153 ms |
| `grafeo.spec.lpg.cypher.regression.not.exists.with.type.filter` | `Conformance` | lpg | `Passed` | 2.746 ms |
| `grafeo.spec.lpg.cypher.regression.sum.case.when` | `Conformance` | lpg | `Passed` | 3.632 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.matches` | `Conformance` | lpg | `Passed` | 2.319 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.no.match` | `Conformance` | lpg | `Passed` | 2.168 ms |
| `grafeo.spec.lpg.cypher.regression.any.with.single.match` | `Conformance` | lpg | `Passed` | 2.117 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.max` | `Conformance` | lpg | `Passed` | 2.523 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.min` | `Conformance` | lpg | `Passed` | 2.444 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.conditional.sum` | `Conformance` | lpg | `Passed` | 2.522 ms |
| `grafeo.spec.lpg.cypher.regression.outgoing.target.property.filter` | `Conformance` | lpg | `Passed` | 3.258 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.count` | `Conformance` | lpg | `Passed` | 2.930 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.no.match` | `Conformance` | lpg | `Passed` | 2.325 ms |
| `grafeo.spec.lpg.cypher.regression.edge.property.filter` | `Conformance` | lpg | `Passed` | 2.867 ms |
| `grafeo.spec.lpg.cypher.regression.optional.match.count.preserves.all.rows` | `Conformance` | lpg | `Passed` | 2.591 ms |
| `grafeo.spec.lpg.cypher.regression.union.deduplicates` | `Conformance` | lpg | `Passed` | 1.659 ms |
| `grafeo.spec.lpg.cypher.regression.union.all.preserves` | `Conformance` | lpg | `Passed` | 1.558 ms |
| `grafeo.spec.lpg.cypher.regression.two.hop.equivalence` | `Conformance` | lpg | `Passed` | 3.096 ms |
| `grafeo.spec.lpg.cypher.regression.merge.creates.new.after.delete` | `Conformance` | lpg | `Passed` | 2.183 ms |
| `grafeo.spec.lpg.cypher.regression.replace.edge` | `Conformance` | lpg | `Passed` | 2.981 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.forward` | `Conformance` | lpg | `Passed` | 2.359 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.reverse` | `Conformance` | lpg | `Passed` | 2.355 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.wrong.direction` | `Conformance` | lpg | `Passed` | 2.335 ms |
| `grafeo.spec.lpg.cypher.regression.null.equals.null.is.unknown` | `Conformance` | lpg | `Passed` | 1.812 ms |
| `grafeo.spec.lpg.cypher.regression.null.is.null.is.true` | `Conformance` | lpg | `Passed` | 1.805 ms |
| `grafeo.spec.lpg.cypher.regression.bool.to.string` | `Conformance` | lpg | `Passed` | 1.958 ms |
| `grafeo.spec.lpg.cypher.regression.int.to.string` | `Conformance` | lpg | `Passed` | 1.931 ms |
| `grafeo.spec.lpg.cypher.regression.string.false.ne.bool.false` | `Conformance` | lpg | `Failed` | 2.017 ms |
| `grafeo.spec.lpg.cypher.regression.neq.excludes.null` | `Conformance` | lpg | `Passed` | 2.197 ms |
| `grafeo.spec.lpg.cypher.regression.skip.plus.limit` | `Conformance` | lpg | `Passed` | 3.815 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.values` | `Conformance` | lpg | `Passed` | 2.334 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.collapses.nulls` | `Conformance` | lpg | `Passed` | 2.345 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.property.matching.return.alias.with.edge` | `Conformance` | lpg | `Passed` | 3.054 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.desc.with.relationship.traversal` | `Conformance` | lpg | `Passed` | 3.169 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.expression` | `Conformance` | lpg | `Passed` | 1.817 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.alias` | `Conformance` | lpg | `Passed` | 1.725 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.distinct` | `Conformance` | lpg | `Passed` | 2.018 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.star` | `Conformance` | lpg | `Passed` | 1.745 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.count.star` | `Conformance` | lpg | `Passed` | 1.958 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.arithmetic` | `Conformance` | lpg | `Passed` | 1.814 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.boolean.expression` | `Conformance` | lpg | `Passed` | 2.043 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.asc` | `Conformance` | lpg | `Passed` | 2.099 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.desc` | `Conformance` | lpg | `Passed` | 2.014 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.multiple.keys` | `Conformance` | lpg | `Passed` | 2.287 ms |
| `grafeo.spec.lpg.cypher.return.ordering.limit` | `Conformance` | lpg | `Passed` | 2.615 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip` | `Conformance` | lpg | `Passed` | 2.648 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip.and.limit` | `Conformance` | lpg | `Passed` | 2.619 ms |
| `grafeo.spec.lpg.cypher.types.integer.decimal` | `Conformance` | lpg | `Passed` | 1.872 ms |
| `grafeo.spec.lpg.cypher.types.integer.negative` | `Conformance` | lpg | `Passed` | 1.773 ms |
| `grafeo.spec.lpg.cypher.types.integer.zero` | `Conformance` | lpg | `Passed` | 1.820 ms |
| `grafeo.spec.lpg.cypher.types.integer.hex` | `Conformance` | lpg | `Passed` | 1.765 ms |
| `grafeo.spec.lpg.cypher.types.integer.octal` | `Conformance` | lpg | `Passed` | 1.788 ms |
| `grafeo.spec.lpg.cypher.types.float.decimal` | `Conformance` | lpg | `Passed` | 1.868 ms |
| `grafeo.spec.lpg.cypher.types.float.scientific` | `Conformance` | lpg | `Passed` | 1.890 ms |
| `grafeo.spec.lpg.cypher.types.float.negative` | `Conformance` | lpg | `Passed` | 1.896 ms |
| `grafeo.spec.lpg.cypher.types.string.single.quoted` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.types.string.double.quoted` | `Conformance` | lpg | `Passed` | 1.816 ms |
| `grafeo.spec.lpg.cypher.types.string.empty` | `Conformance` | lpg | `Passed` | 1.799 ms |
| `grafeo.spec.lpg.cypher.types.boolean.true` | `Conformance` | lpg | `Passed` | 1.793 ms |
| `grafeo.spec.lpg.cypher.types.boolean.false` | `Conformance` | lpg | `Passed` | 1.800 ms |
| `grafeo.spec.lpg.cypher.types.null.literal` | `Conformance` | lpg | `Passed` | 1.784 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.null` | `Conformance` | lpg | `Passed` | 1.827 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.not.null` | `Conformance` | lpg | `Passed` | 1.829 ms |
| `grafeo.spec.lpg.cypher.types.null.equality.returns.null` | `Conformance` | lpg | `Failed` | 1.779 ms |
| `grafeo.spec.lpg.cypher.types.missing.property.is.null` | `Conformance` | lpg | `Passed` | 1.829 ms |
| `grafeo.spec.lpg.cypher.types.list.of.integers` | `Conformance` | lpg | `Failed` | 1.806 ms |
| `grafeo.spec.lpg.cypher.types.list.empty` | `Conformance` | lpg | `Passed` | 1.774 ms |
| `grafeo.spec.lpg.cypher.types.list.nested` | `Conformance` | lpg | `Passed` | 1.799 ms |
| `grafeo.spec.lpg.cypher.types.list.size` | `Conformance` | lpg | `Passed` | 1.821 ms |
| `grafeo.spec.lpg.cypher.types.map.literal` | `Conformance` | lpg | `Passed` | 1.904 ms |
| `grafeo.spec.lpg.cypher.types.map.key.count` | `Conformance` | lpg | `Passed` | 1.973 ms |
| `grafeo.spec.lpg.cypher.types.node.return` | `Conformance` | lpg | `Passed` | 1.731 ms |
| `grafeo.spec.lpg.cypher.types.relationship.return` | `Conformance` | lpg | `Passed` | 2.272 ms |
| `grafeo.spec.lpg.cypher.types.path.return` | `Conformance` | lpg | `Passed` | 2.266 ms |
| `grafeo.spec.lpg.cypher.types.date.from.string` | `Conformance` | lpg | `Passed` | 1.781 ms |
| `grafeo.spec.lpg.cypher.types.time.from.string` | `Conformance` | lpg | `Passed` | 1.743 ms |
| `grafeo.spec.lpg.cypher.types.datetime.from.string` | `Conformance` | lpg | `Passed` | 1.777 ms |
| `grafeo.spec.lpg.cypher.types.duration.from.string` | `Conformance` | lpg | `Passed` | 1.769 ms |
| `grafeo.spec.lpg.cypher.types.date.stored.as.property` | `Conformance` | lpg | `Passed` | 1.811 ms |
| `grafeo.spec.lpg.cypher.types.integer.to.float.arithmetic` | `Conformance` | lpg | `Passed` | 1.798 ms |
| `grafeo.spec.lpg.cypher.types.to.integer.truncation` | `Conformance` | lpg | `Passed` | 1.768 ms |
| `grafeo.spec.lpg.cypher.types.to.float.from.integer` | `Conformance` | lpg | `Passed` | 1.780 ms |
| `grafeo.spec.lpg.cypher.types.to.string.from.boolean` | `Conformance` | lpg | `Failed` | 1.803 ms |
| `grafeo.spec.lpg.cypher.types.to.boolean.from.string.false` | `Conformance` | lpg | `Passed` | 1.773 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node` | `Conformance` | lpg | `Passed` | 1.718 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node.multi.label` | `Conformance` | lpg | `Passed` | 1.880 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship` | `Conformance` | lpg | `Passed` | 2.217 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship.with.properties` | `Conformance` | lpg | `Passed` | 2.245 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.path.pattern` | `Conformance` | lpg | `Passed` | 2.458 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.node` | `Conformance` | lpg | `Passed` | 1.908 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.multiple` | `Conformance` | lpg | `Passed` | 2.126 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete` | `Conformance` | lpg | `Passed` | 2.253 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete.with.return` | `Conformance` | lpg | `Passed` | 2.216 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.property` | `Conformance` | lpg | `Passed` | 2.029 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.properties` | `Conformance` | lpg | `Passed` | 2.049 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.replace.all` | `Conformance` | lpg | `Passed` | 2.034 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.merge.map` | `Conformance` | lpg | `Passed` | 1.971 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label` | `Conformance` | lpg | `Passed` | 1.908 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.labels` | `Conformance` | lpg | `Passed` | 2.122 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label.preserves.variable.binding` | `Conformance` | lpg | `Passed` | 1.906 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.star.after.set.label` | `Conformance` | lpg | `Passed` | 2.467 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.var.after.set.label` | `Conformance` | lpg | `Passed` | 2.236 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.property` | `Conformance` | lpg | `Passed` | 1.957 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label` | `Conformance` | lpg | `Failed` | 0.011 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label.preserves.variable.binding` | `Conformance` | lpg | `Failed` | 0.004 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.no.phantoms` | `Conformance` | lpg | `Passed` | 2.273 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.correct.endpoints` | `Conformance` | lpg | `Passed` | 2.564 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.create` | `Conformance` | lpg | `Passed` | 1.817 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.match` | `Conformance` | lpg | `Passed` | 1.913 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set` | `Conformance` | lpg | `Failed` | 1.958 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set` | `Conformance` | lpg | `Passed` | 2.054 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set.self.reference.increment` | `Conformance` | lpg | `Passed` | 2.060 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set.self.reference.coalesce` | `Conformance` | lpg | `Passed` | 1.954 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship` | `Conformance` | lpg | `Passed` | 2.365 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship.set` | `Conformance` | lpg | `Passed` | 2.564 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.foreach.create` | `Conformance` | lpg | `Passed` | 2.478 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.same.source.and.target.variable.cypher-variant` | `Conformance` | regression | `Failed` | 1.538 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.no.cycle.returns.empty.cypher-variant` | `Conformance` | regression | `Failed` | 1.496 ms |
| `grafeo.spec.rosetta.aggregation.count.products.cypher-variant` | `Conformance` | rosetta | `Failed` | 5.502 ms |
| `grafeo.spec.rosetta.aggregation.sum.order.totals.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.003 ms |
| `grafeo.spec.rosetta.aggregation.avg.product.price.cypher-variant` | `Conformance` | rosetta | `Failed` | 5.475 ms |
| `grafeo.spec.rosetta.aggregation.min.max.price.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 5.454 ms |
| `grafeo.spec.rosetta.aggregation.count.by.status.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.004 ms |
| `grafeo.spec.rosetta.aggregation.orders.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.004 ms |
| `grafeo.spec.rosetta.aggregation.total.spend.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.003 ms |
| `grafeo.spec.rosetta.aggregation.customers.with.multiple.orders.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 0.003 ms |
| `grafeo.spec.rosetta.aggregation.avg.review.rating.cypher-variant` | `Conformance` | rosetta | `Failed` | 5.653 ms |
| `grafeo.spec.rosetta.basic.queries.count.all.nodes.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.667 ms |
| `grafeo.spec.rosetta.basic.queries.match.by.label.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.634 ms |
| `grafeo.spec.rosetta.basic.queries.filter.by.age.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.650 ms |
| `grafeo.spec.rosetta.basic.queries.edge.traversal.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.637 ms |
| `grafeo.spec.rosetta.basic.queries.two.hop.path.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.799 ms |
| `grafeo.spec.rosetta.basic.queries.aggregation.group.by.cypher-variant` | `Conformance` | rosetta | `Passed` | 3.749 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.and.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.577 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.read.properties.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.556 ms |
| `grafeo.spec.rosetta.crud.operations.create.edge.and.traverse.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.526 ms |
| `grafeo.spec.rosetta.crud.operations.match.count.multiple.nodes.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.482 ms |
| `grafeo.spec.rosetta.crud.operations.set.property.and.read.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.524 ms |
| `grafeo.spec.rosetta.crud.operations.delete.node.and.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.477 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.sum.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.499 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.505 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.avg.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.514 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.name.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.519 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.503 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.edge.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.481 ms |
| `grafeo.spec.rosetta.data.fidelity.int.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.570 ms |
| `grafeo.spec.rosetta.data.fidelity.bool.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.482 ms |
| `grafeo.spec.rosetta.data.fidelity.string.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.464 ms |
| `grafeo.spec.rosetta.data.fidelity.missing.property.null.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.506 ms |
| `grafeo.spec.rosetta.data.fidelity.multi.label.visible.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.477 ms |
| `grafeo.spec.rosetta.data.fidelity.edge.type.in.cypher.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.511 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.all.read.count.cypher-variant` | `Conformance` | rosetta | `Failed` | 1.505 ms |
| `grafeo.spec.rosetta.pattern.matching.count.actors.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.808 ms |
| `grafeo.spec.rosetta.pattern.matching.find.actor.by.name.cypher-variant` | `Conformance` | rosetta | `Passed` | 13.912 ms |
| `grafeo.spec.rosetta.pattern.matching.actors.in.heist.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.418 ms |
| `grafeo.spec.rosetta.pattern.matching.genres.of.vincent.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.799 ms |
| `grafeo.spec.rosetta.pattern.matching.movies.per.director.cypher.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.261 ms |
| `grafeo.spec.rosetta.pattern.matching.actor.roles.in.movie.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.912 ms |
| `grafeo.spec.rosetta.pattern.matching.high.rated.movies.cypher-variant` | `Conformance` | rosetta | `Passed` | 14.004 ms |

## Latest `performance-deep` run

- Run: `20260718T013944.410388Z-e1d73880b749-performance-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Semantics: v0
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
- Semantics: v0
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
- Semantics: v0
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

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Commit: `8e296519275bbca3b006b64032b9776d76de5037` (dirty)
- Package: `0.8.0-pre.1`
- Semantics: v3
- Environment: `macos/aarch64` (`release`)
- Records: 2225
- Passed: 2164
- Unsupported: 0
- Failed or changed: 61

### Outcome changes from `20260726T225523.619934Z-f5c009b2f8e2-corpus-deep`

- No outcome changes.

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| acceptance | `failed` | 3 |
| acceptance | `passed` | 33 |
| call_subquery | `failed` | 2 |
| call_subquery | `passed` | 16 |
| cypher_range_function_test | `passed` | 3 |
| debug_case_when | `passed` | 4 |
| debug_so_subclass | `passed` | 2 |
| delete_edge | `passed` | 7 |
| export_import | `passed` | 19 |
| fts_index | `failed` | 8 |
| gap_10_parameterized_queries | `passed` | 19 |
| hybrid_search | `failed` | 5 |
| match_after_create | `passed` | 6 |
| match_property_index | `passed` | 13 |
| mcp_cypher_templates | `failed` | 2 |
| mcp_cypher_templates | `passed` | 17 |
| merge_node | `passed` | 4 |
| path_semantics | `passed` | 5 |
| property_range_index | `passed` | 6 |
| readtx_query | `passed` | 22 |
| regression_355 | `passed` | 9 |
| regression_363 | `passed` | 13 |
| regression_364 | `passed` | 3 |
| regression_366 | `passed` | 4 |
| regression_367 | `passed` | 10 |
| regression_368 | `passed` | 9 |
| regression_369 | `passed` | 4 |
| regression_372 | `passed` | 11 |
| regression_373 | `passed` | 18 |
| regression_379 | `passed` | 30 |
| regression_380 | `passed` | 15 |
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
| spa_151_kms_query_validation | `failed` | 3 |
| spa_151_kms_query_validation | `passed` | 85 |
| spa_155_unwind_param | `passed` | 3 |
| spa_156_161 | `passed` | 16 |
| spa_165_col_prefix_property | `passed` | 9 |
| spa_168_degree_cache_wiring | `passed` | 16 |
| spa_168_match_create | `passed` | 9 |
| spa_169_string_props | `passed` | 19 |
| spa_172_count_distinct | `passed` | 17 |
| spa_178_edge_properties | `passed` | 27 |
| spa_182_create_path_rhs | `passed` | 5 |
| spa_183_match_create_bindings | `passed` | 16 |
| spa_185_rel_table_id | `passed` | 24 |
| spa_186_csr_nodeid | `passed` | 13 |
| spa_187_column_slot_alignment | `passed` | 17 |
| spa_188_two_hop_where | `passed` | 32 |
| spa_189_checkpoint_optimize | `passed` | 6 |
| spa_192_match_no_label | `passed` | 17 |
| spa_193_undirected_pattern | `passed` | 12 |
| spa_194_count_node_var | `passed` | 12 |
| spa_195_type_function | `passed` | 14 |
| spa_196_id_function | `passed` | 14 |
| spa_197_count_label_fastpath | `passed` | 16 |
| spa_197_missing_prop_null | `passed` | 7 |
| spa_198_limit_pushdown | `passed` | 8 |
| spa_198_unlabeled_rel_endpoint | `passed` | 8 |
| spa_199_bfs_early_exit | `passed` | 6 |
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
| spa_235_234_create_index_constraint | `passed` | 14 |
| spa_236_labels_predicate | `passed` | 19 |
| spa_237_unwind_match | `passed` | 19 |
| spa_240_coalesce | `passed` | 11 |
| spa_241_multihop_props | `passed` | 15 |
| spa_242_count_rel_var | `passed` | 16 |
| spa_243_create_entity | `passed` | 9 |
| spa_244_mcp_errors | `passed` | 3 |
| spa_245_unknown_label_returns_empty | `passed` | 10 |
| spa_249_property_index | `passed` | 37 |
| spa_250_batch_write | `passed` | 2 |
| spa_251_text_search_index | `passed` | 30 |
| spa_252_three_hop_binding | `passed` | 15 |
| spa_254_query_timeout | `passed` | 2 |
| spa_259_inline_prop_filter | `passed` | 10 |
| spa_261_edge_props_perf | `passed` | 11 |
| spa_263_two_hop_agg | `passed` | 23 |
| spa_263_two_hop_null | `passed` | 25 |
| spa_264_boolean_props | `passed` | 14 |
| spa_265_backtick_escaping | `failed` | 4 |
| spa_265_backtick_escaping | `passed` | 22 |
| spa_266_265_bugs | `passed` | 6 |
| spa_267_float_codec | `passed` | 17 |
| spa_268_bfs_bugs | `passed` | 21 |
| spa_272_degree_cache | `passed` | 11 |
| spa_272_q7_count_fastpath | `passed` | 24 |
| spa_272_q7_cypher_wiring | `failed` | 4 |
| spa_272_q7_cypher_wiring | `passed` | 20 |
| spa_273_planner_stats | `passed` | 22 |
| spa_289_multi_label | `passed` | 28 |
| spa_296_bulk_loader | `passed` | 1 |
| spa_299_chunked_pipeline | `passed` | 31 |
| spa_299_phase2_parity | `passed` | 35 |
| spa_299_phase3_parity | `passed` | 66 |
| spa_299_phase4_parity | `passed` | 66 |
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
| spa_list_predicates | `passed` | 38 |
| spa_type_labels | `failed` | 1 |
| spa_type_labels | `passed` | 17 |
| spa_variable_paths | `passed` | 37 |
| test_pole | `passed` | 8 |
| test_reactome | `passed` | 5 |
| uc1_social_graph | `passed` | 2 |
| uc7_unwind | `failed` | 2 |
| uc7_unwind | `passed` | 5 |
| uc_tracing | `passed` | 1 |
| vector_index | `failed` | 4 |
| vector_index | `passed` | 7 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 55 |
| `execution` | `passed` | 2164 |
| `parser` | `failed` | 6 |

### Failures (61)

- `sparrowdb.acceptance.check-14-fulltext-search.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..74
- `sparrowdb.acceptance.check-14-fulltext-search.query-2`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..72
- `sparrowdb.acceptance.check-14-fulltext-search.query-3`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..72
- `sparrowdb.call-subquery.correlated-subquery-counts-friends.query-4`: query execution failed: Parse error: CALL subqueries after other clauses is not supported in the initial graph slice at byte 17..85; mutation execution failed: Cypher mutation binding failed: CALL subqueries after other clauses is not supported in the initial graph slice at byte 17..85
- `sparrowdb.call-subquery.correlated-subquery-collects-friend-names.query-6`: query execution failed: Parse error: CALL subqueries after other clauses is not supported in the initial graph slice at byte 33..102; mutation execution failed: Cypher mutation binding failed: CALL subqueries after other clauses is not supported in the initial graph slice at byte 33..102
- `sparrowdb.fts-index.test-auto-index-on-create.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 76..92
- `sparrowdb.fts-index.test-full-text-search-predicate.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 68..81
- `sparrowdb.fts-index.test-full-text-search-predicate.query-2`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 70..83
- `sparrowdb.fts-index.test-full-text-search-predicate.query-3`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 75..88
- `sparrowdb.fts-index.test-bm25-score-order-by.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 86..188
- `sparrowdb.fts-index.test-multiword-query-union-scoring.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 66..79
- `sparrowdb.fts-index.test-fts-index-survives-restart.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 68..81
- `sparrowdb.fts-index.test-bm25-ranking-50-nodes.query-1`: query execution failed: Parse error: no such function: full_text_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..183
- `sparrowdb.hybrid-search.hybrid-search-20-nodes-rrf.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..98
- `sparrowdb.hybrid-search.hybrid-search-weighted-fusion-alpha.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..89
- `sparrowdb.hybrid-search.hybrid-search-weighted-fusion-alpha.query-2`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..89
- `sparrowdb.hybrid-search.hybrid-search-missing-fts-falls-back.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..76
- `sparrowdb.hybrid-search.hybrid-search-k-zero-returns-null.query-1`: query execution failed: Parse error: no such function: hybrid_search; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..63
- `sparrowdb.mcp-cypher-templates.mcp-template-merge-node-count-query-integer-parses-and-executes.query-1`: expected identifier at byte 9..9
- `sparrowdb.mcp-cypher-templates.mcp-template-merge-node-count-query-integer-parses-and-executes.query-2`: expected identifier at byte 9..9
- `sparrowdb.spa-136-shortest-path.shortest-path-direct.query-4`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-136-shortest-path.shortest-path-2-hops.query-6`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-136-shortest-path.shortest-path-no-path.query-3`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 59..98
- `sparrowdb.spa-139-phase9-path-acceptance.shortest-path-prefers-minimum-hops.query-7`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 49..87
- `sparrowdb.spa-139-phase9-path-acceptance.shortest-path-null-when-disconnected.query-3`: query execution failed: Parse error: no such function: shortestPath; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 53..93
- `sparrowdb.spa-140-143-functions.spa143-isnull-true.query-1`: query execution failed: Parse error: generated relational SQL did not parse: near "isNull": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `sparrowdb.spa-140-143-functions.spa143-isnotnull-true.query-1`: query execution failed: Parse error: no such function: isNotNull; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..25
- `sparrowdb.spa-140-143-functions.spa143-id-function-in-match-return.query-1`: query execution failed: Parse error: generated relational SQL did not parse: near "isNull": syntax error; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..19
- `sparrowdb.spa-151-kms-query-validation.kms-q18-fulltext-search-call-procedure.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..80
- `sparrowdb.spa-151-kms-query-validation.kms-q18b-fulltext-search-yield-node-only.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..74
- `sparrowdb.spa-151-kms-query-validation.kms-q19-fulltext-search-no-results.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..80
- `sparrowdb.spa-209-schema-introspection.schema-result-has-named-columns.query-2`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-contains-node-labels-and-properties.query-4`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-contains-relationship-types.query-3`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-empty-db-returns-no-rows.query-1`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.schema-label-with-no-properties.query-1`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-209-schema-introspection.query-result-row-as-map.query-2`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-265-backtick-escaping.bare-keyword-label-order.query-1`: expected identifier at byte 10..10
- `sparrowdb.spa-265-backtick-escaping.bare-keyword-label-order.query-2`: expected identifier at byte 9..9
- `sparrowdb.spa-265-backtick-escaping.backtick-and-bare-keyword-same-case-are-interchangeable.query-1`: expected identifier at byte 10..10
- `sparrowdb.spa-265-backtick-escaping.backtick-and-bare-keyword-same-case-are-interchangeable.query-4`: expected identifier at byte 9..9
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-out-degree-returns-top-k.query-11`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..73
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-alias-returns-top-k.query-7`: query execution failed: Parse error: no such function: degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 17..65
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-unknown-label-returns-empty.query-1`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..79
- `sparrowdb.spa-272-q7-cypher-wiring.cypher-order-by-degree-desc-ordering-correct.query-2`: query execution failed: Parse error: no such function: out_degree; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 15..71
- `sparrowdb.spa-98-wal-encryption.spa-98-wrong-key-fails.query-1`: query execution failed: Parse error: unsupported graph procedure `db.schema` at byte 5..14; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..16
- `sparrowdb.spa-datetime-fns.timestamp-alias.query-1`: query execution failed: Parse error: no such function: timestamp; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..18
- `sparrowdb.spa-fulltext.create-index-and-search.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.search-partial-match.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..77
- `sparrowdb.spa-fulltext.search-partial-match.query-2`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..66
- `sparrowdb.spa-fulltext.call-yield-node-usable-in-return.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.call-yield-node-usable-in-return.query-2`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..71
- `sparrowdb.spa-fulltext.unknown-procedure-returns-error.query-1`: query execution failed: Parse error: unsupported graph procedure `unknown.procedure` at byte 5..22; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..40
- `sparrowdb.spa-fulltext.call-missing-index-returns-empty.query-1`: query execution failed: Parse error: unsupported graph procedure `db.index.fulltext.queryNodes` at byte 5..33; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 0..73
- `sparrowdb.spa-type-labels.type-fn-variable-path.query-6`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 44..58
- `sparrowdb.uc7-unwind.unwind-param-returns-empty-without-binding.query-1`: query execution failed: Parse error: unknown parameter `$items` at byte 7..14; mutation execution failed: Cypher mutation binding failed: unknown parameter `$items` at byte 7..14
- `sparrowdb.uc7-unwind.unwind-return-wrong-variable-yields-null.query-1`: query execution failed: Parse error: unknown variable `y` at byte 29..30; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 22..30
- `sparrowdb.vector-index.vector-similarity-function.query-3`: query execution failed: Parse error: no such function: vector_similarity; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..65
- `sparrowdb.vector-index.vector-similarity-orthogonal-is-zero.query-1`: query execution failed: Parse error: no such function: vector_similarity; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..55
- `sparrowdb.vector-index.vector-distance-function.query-1`: query execution failed: Parse error: no such function: vector_distance; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..51
- `sparrowdb.vector-index.vector-dot-function.query-1`: query execution failed: Parse error: no such function: vector_dot; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 0..47

## Latest `tck-deep` run

- Run: `20260726T235836.378258Z-8e296519275b-corpus-deep`
- Commit: `8e296519275bbca3b006b64032b9776d76de5037` (dirty)
- Package: `0.8.0-pre.1`
- Semantics: v3
- Environment: `macos/aarch64` (`release`)
- Records: 3926
- Passed: 3332
- Unsupported: 0
- Failed or changed: 594

### Outcome changes from `20260726T225523.619934Z-f5c009b2f8e2-corpus-deep`

- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-1`: Passed
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2`: Passed

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| clauses | `failed` | 232 |
| clauses | `passed` | 1019 |
| expressions | `failed` | 343 |
| expressions | `passed` | 2302 |
| useCases | `failed` | 19 |
| useCases | `passed` | 11 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `execution` | `failed` | 482 |
| `execution` | `passed` | 3291 |
| `fixture-execution` | `failed` | 19 |
| `parser` | `failed` | 66 |
| `parser` | `passed` | 41 |
| `setup-execution` | `failed` | 4 |
| `side-effect-comparison` | `failed` | 23 |

### Failures (594)

- `tck.clauses.call.call1.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.doNothing` at byte 6..20; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.doNothing()
- `tck.clauses.call.call1.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.doNothing` at byte 6..20; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..21; query:
CALL test.doNothing
- `tck.clauses.call.call1.scenario-3`: query execution failed: Parse error: unsupported graph procedure `test.doNothing` at byte 16..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 11..33; query:
MATCH (n)
CALL test.doNothing()
RETURN n
- `tck.clauses.call.call1.scenario-4`: query execution failed: Parse error: unsupported graph procedure `test.doNothing` at byte 16..30; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 11..33; query:
MATCH (n)
CALL test.doNothing()
RETURN n.name AS `name`
- `tck.clauses.call.call1.scenario-5`: query execution failed: Parse error: unsupported graph procedure `test.labels` at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..20; query:
CALL test.labels()
- `tck.clauses.call.call1.scenario-6`: query execution failed: Parse error: unsupported graph procedure `test.labels` at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.labels() YIELD label
RETURN label
- `tck.clauses.call.call2.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..56; query:
CALL test.my.proc('Stefan', 1) YIELD city, country_code
RETURN city, country_code
- `tck.clauses.call.call2.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.my.proc('Stefan', 1)
- `tck.clauses.call.call2.scenario-3`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..19; query:
CALL test.my.proc
- `tck.clauses.call.call3.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.my.proc(42)
- `tck.clauses.call.call3.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..33; query:
CALL test.my.proc(42) YIELD out
RETURN out
- `tck.clauses.call.call3.scenario-3`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..25; query:
CALL test.my.proc(42.3)
- `tck.clauses.call.call3.scenario-4`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(42.3) YIELD out
RETURN out
- `tck.clauses.call.call3.scenario-5`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..23; query:
CALL test.my.proc(42)
- `tck.clauses.call.call3.scenario-6`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..33; query:
CALL test.my.proc(42) YIELD out
RETURN out
- `tck.clauses.call.call4.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..25; query:
CALL test.my.proc(null)
- `tck.clauses.call.call4.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN out
- `tck.clauses.call.call5.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN out
- `tck.clauses.call.call5.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
RETURN *
- `tck.clauses.call.call5.scenario-3.examples-1-row-1`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD a, b
RETURN a, b
- `tck.clauses.call.call5.scenario-3.examples-1-row-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
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
- `tck.clauses.call.call6.scenario-1`: query execution failed: Parse error: unsupported graph procedure `test.labels` at byte 6..17; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..32; query:
CALL test.labels() YIELD label
WITH count(*) AS c
CALL test.labels() YIELD label
RETURN *
- `tck.clauses.call.call6.scenario-2`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
WITH out RETURN out
- `tck.clauses.call.call6.scenario-3`: query execution failed: Parse error: unsupported graph procedure `test.my.proc` at byte 6..18; mutation execution failed: Cypher mutation binding failed: procedures in mutation queries is not supported in the initial graph slice at byte 1..35; query:
CALL test.my.proc(null) YIELD out
WITH out AS a RETURN a
- `tck.clauses.create.create5.scenario-4`: expected EOI, UNION, clause, or relationship_pattern at byte 19..19
- `tck.clauses.create.create5.scenario-5`: expected EOI, UNION, clause, or relationship_pattern at byte 11..11
- `tck.clauses.delete.delete1.scenario-5`: expected [["<null>"]], observed []
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
- `tck.clauses.match.match3.scenario-7`: expected [["(:A:B:C:D:E:F:G:H:I:J:K:L:M)", "(:Z:Y:X:W:V:U)"]], observed [["(:A:B:C:D:E:F:G:H:I:J:K:L:M)", "(:U:V:W:X:Y:Z)"]]
- `tck.clauses.match.match3.scenario-19`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 54..54
- `tck.clauses.match.match4.scenario-1`: expected [["[[:T]]"]], observed [["[1]"]]
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
- `tck.clauses.match.match4.scenario-6`: expected [["[[:X], [:Y]]"]], observed [["[1, 2]"]]
- `tck.clauses.match.match4.scenario-7`: query execution failed: Parse error: variable-length path values is not supported in the initial graph slice at byte 79..80; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 66..87; query:
MATCH ()-[r:EDGE]-()
MATCH p = (n)-[*0..1]-()-[r]-()-[*0..1]-(m)
RETURN count(p) AS c
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
- `tck.clauses.match.match5.scenario-26`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 21..21
- `tck.clauses.match.match5.scenario-27`: expected EOI, WHERE, UNION, clause, or relationship_pattern at byte 34..34
- `tck.clauses.match.match6.scenario-1`: expected [[""]], observed [["<()>"]]
- `tck.clauses.match.match6.scenario-5`: expected [[""]], observed [["<(:Label1)<-[:TYPE]-(:Label2)>"]]
- `tck.clauses.match.match6.scenario-6`: expected [[""]], observed [["<(:B)<-[:T]-(:A)>"]]
- `tck.clauses.match.match6.scenario-9`: TCK setup query failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 17..17; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 17..17; query:
CREATE (:Label1)(:Label3)
; query:
CREATE (:Label1)(:Label3)
- `tck.clauses.match.match6.scenario-10`: expected [["(:B)-[:T]->(:A)>"]], observed [["<(:C)-[:T]->(:B)-[:T]->(:A)>"]]
- `tck.clauses.match.match6.scenario-11`: expected [["(:C)-[:T]->(:B)-[:T]->(:A)>"]], observed [["<(:D)-[:T]->(:C)-[:T]->(:B)-[:T]->(:A)>"]]
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
- `tck.clauses.match.match7.scenario-3`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["(:B {num: 46})"], ["(:C)"], ["(:Single)"]]
- `tck.clauses.match.match7.scenario-5`: expected [["(:A)", "[:T]", "(:B)"]], observed [["(:A)", "[:T]", "(:B)"], ["(:B)", "[:T]", "<null>"]]
- `tck.clauses.match.match7.scenario-8`: expected [["<null>"]], observed [["(:C)"]]
- `tck.clauses.match.match7.scenario-9`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["(:B {num: 46})"]]
- `tck.clauses.match.match7.scenario-10`: expected [["<null>"]], observed []
- `tck.clauses.match.match7.scenario-11`: expected [["(:A {num: 1})", "(:B {num: 2})", "(:C {num: 3})"], ["(:B {num: 2})", "(:A {num: 1})", "<null>"]], observed [["(:A {num: 1})", "(:B {num: 2})", "(:C {num: 3})"], ["(:A {num: 1})", "(:B {num: 2})", "<null>"], ["(:B {num: 2})", "(:A {num: 1})", "<null>"]]
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
- `tck.clauses.match.match7.scenario-27`: expected [["<null>", "(:B {num: 46})", "<null>"]], observed []
- `tck.clauses.match.match7.scenario-28`: expected [["<null>"]], observed [["<null>"], ["<null>"]]
- `tck.clauses.match.match8.scenario-2`: expected [["6"]], observed [["3"]]
- `tck.clauses.match.match9.scenario-2`: expected [["[[:REL {num: 1}], [:REL {num: 2}]]"]], observed [["[1, 2]"]]
- `tck.clauses.match.match9.scenario-3`: expected [["[[:REL {num: 1}], [:REL {num: 2}]]"], ["[[:REL {num: 2}], [:REL {num: 1}]]"]], observed [["[1, 2]"], ["[2, 1]"]]
- `tck.clauses.match.match9.scenario-4`: expected [["[[:REL {num: 1}], [:REL {num: 2}]]"]], observed [["[1, 2]"]]
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
- `tck.clauses.match-where.matchwhere1.scenario-5`: TCK setup query failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 27..27; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 27..27; query:
CREATE ({name: 'Someone'})({name: 'Andres'})
; query:
CREATE ({name: 'Someone'})({name: 'Andres'})
- `tck.clauses.match-where.matchwhere1.scenario-6`: expected [["[:T{name:\"bar\"}]"]], observed [["[:T {name: 'bar'}]"]]
- `tck.clauses.match-where.matchwhere4.scenario-2`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 93..111; query:
MATCH (a), (b)
WHERE a.id = 0
  AND (a)-[:T]->(b:TheLabel)
  OR (a)-[:T*]->(b:MissingLabel)
RETURN DISTINCT b
- `tck.clauses.match-where.matchwhere6.scenario-1`: expected [["A"]], observed [["A"], ["A"], ["A"]]
- `tck.clauses.match-where.matchwhere6.scenario-2`: expected [["<null>"]], observed [["<null>"], ["<null>"]]
- `tck.clauses.match-where.matchwhere6.scenario-3`: expected [["(:A {num: 42})"]], observed [["(:A {num: 42})"], ["<null>"]]
- `tck.clauses.match-where.matchwhere6.scenario-5`: expected [["(:A)", "[:T]", "<null>", "<null>"]], observed [["(:A)", "[:T]", "<null>", "(:A)"], ["(:A)", "[:T]", "<null>", "(:B)"]]
- `tck.clauses.match-where.matchwhere6.scenario-7`: expected [["(:X {val: 1})", "(:Y {val: 2})", "(:Z {val: 3})"], ["(:X {val: 4})", "<null>", "<null>"], ["(:X {val: 6})", "<null>", "<null>"]], observed [["(:X {val: 1})", "(:Y {val: 2})", "(:Z {val: 3})"], ["(:X {val: 4})", "(:Y {val: 5})", "<null>"], ["(:X {val: 6})", "<null>", "<null>"]]
- `tck.clauses.merge.merge1.scenario-14`: side effect +nodes expected 1, observed 0
- `tck.clauses.merge.merge5.scenario-3`: expected [["2"]], observed [["1"]]
- `tck.clauses.merge.merge5.scenario-11`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..33; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 64..76; query:
CREATE (a {id: 2}), (b {id: 1})
MERGE (a)-[r:KNOWS]-(b)
RETURN startNode(r).id AS s, endNode(r).id AS e
- `tck.clauses.merge.merge5.scenario-13`: expected [["[:KNOWS{name:\"ab\"}]"], ["[:KNOWS{name:\"cd\"}]"]], observed [["[:KNOWS {name: 'ab'}]"], ["[:KNOWS]"]]
- `tck.clauses.merge.merge5.scenario-20`: side effect +nodes expected 2, observed 0
- `tck.clauses.merge.merge5.scenario-21`: side effect +relationships expected 1, observed 0
- `tck.clauses.merge.merge6.scenario-6`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 40..86; mutation execution failed: Cypher mutation binding failed: SET of a whole entity from a non-map value is not supported in the initial graph slice at byte 84..86; query:
MATCH (a {name: 'A'}), (b {name: 'B'})
MERGE (a)-[r:TYPE]->(b)
  ON CREATE SET r = a
- `tck.clauses.merge.merge7.scenario-4`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 40..85; mutation execution failed: Cypher mutation binding failed: SET of a whole entity from a non-map value is not supported in the initial graph slice at byte 83..85; query:
MATCH (a {name: 'A'}), (b {name: 'B'})
MERGE (a)-[r:TYPE]->(b)
  ON MATCH SET r = a
- `tck.clauses.merge.merge7.scenario-5`: side effect +properties expected 2, observed 1
- `tck.clauses.remove.remove1.scenario-5`: expected [["<null>"]], observed []
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
- `tck.clauses.return.return6.scenario-6`: query execution failed: Parse error: unknown variable `a` at byte 68..69; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 47..112; query:
MATCH (a {name: 'Andres'})<-[:FATHER]-(child)
RETURN a.name, {foo: a.name='Andres', kids: collect(child.name)}
- `tck.clauses.return.return6.scenario-10`: expected [["1", "[()]"]], observed [["1", "[1]"]]
- `tck.clauses.return.return6.scenario-13`: expected [["a", "[\"c\",\"b\"]", "1"]], observed [["a", "[\"b\",\"c\"]", "1"]]
- `tck.clauses.return.return6.scenario-19`: query execution failed: Parse error: unknown variable `me` at byte 50..52; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 35..74; query:
MATCH (me: Person)--(you: Person)
RETURN me.age, me.age + count(you.age)
- `tck.clauses.return.return7.scenario-1`: expected [["(:Start)", "()", "()>"]], observed [["<(:Start)-[:T]->()>", "(:Start)", "[:T]", "()"]]
- `tck.clauses.return.return7.scenario-2`: expected an error but execution succeeded
- `tck.clauses.return-orderby.returnorderby1.scenario-9`: expected [["[]"], ["[\"a\"]"], ["[\"a\",1]"], ["[1]"], ["[1,\"a\"]"], ["[1, null]"], ["[null, 1]"], ["[null, 2]"]], observed [["[\"a\",1]"], ["[\"a\"]"], ["[1,\"a\"]"], ["[1, null]"], ["[1]"], ["[]"], ["[null, 1]"], ["[null, 2]"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-10`: expected [["[null, 2]"], ["[null, 1]"], ["[1, null]"], ["[1,\"a\"]"], ["[1]"], ["[\"a\",1]"], ["[\"a\"]"], ["[]"]], observed [["[null, 2]"], ["[null, 1]"], ["[]"], ["[1]"], ["[1, null]"], ["[1,\"a\"]"], ["[\"a\"]"], ["[\"a\",1]"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-11`: expected [["{a: 'map'}"], ["(:N)"], ["[:REL]"], ["[\"list\"]"], ["()>"], ["text"], ["0"], ["1.5"], ["NaN"], ["<null>"]], observed [["0"], ["1"], ["1"], ["1.5"], ["[\"list\"]"], ["text"], ["{\"a\":\"map\"}"], ["{\"nodes\":[1,2],\"relationships\":[1]}"], ["<null>"], ["<null>"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-12`: expected [["<null>"], ["NaN"], ["1.5"], ["0"], ["text"], ["()>"], ["[\"list\"]"], ["[:REL]"], ["(:N)"], ["{a: 'map'}"]], observed [["<null>"], ["<null>"], ["{\"nodes\":[1,2],\"relationships\":[1]}"], ["{\"a\":\"map\"}"], ["text"], ["[\"list\"]"], ["1.5"], ["1"], ["1"], ["0"]]
- `tck.clauses.return-orderby.returnorderby2.scenario-12`: expected [["[[(:A), (:B)], [(:C), (:D)], [(:D), (:E)], [(:E), (:F)]]", "1"], ["[[(:C), (:D), (:E), (:F)]]", "3"], ["[[(:C), (:D), (:E)], [(:D), (:E), (:F)]]", "2"]], observed [["[[1, 2], [3, 4], [4, 5], [5, 6]]", "1"], ["[[3, 4, 5, 6]]", "3"], ["[[3, 4, 5], [4, 5, 6]]", "2"]]
- `tck.clauses.return-orderby.returnorderby2.scenario-13`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-6`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-7`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit1.scenario-11`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-10`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-11`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-12`: expected an error but execution succeeded
- `tck.clauses.return-skip-limit.returnskiplimit2.scenario-13`: expected an error but execution succeeded
- `tck.clauses.set.set1.scenario-1`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set1.scenario-2`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set1.scenario-3`: expected set_item at byte 16..16
- `tck.clauses.set.set1.scenario-4`: expected set_item at byte 25..25
- `tck.clauses.set.set1.scenario-7`: expected [["[1, 2, 3, 4, 5]"]], observed [["[1,2,\"[3,4,5]\"]"]]
- `tck.clauses.set.set1.scenario-8`: expected [["<null>"]], observed []
- `tck.clauses.set.set1.scenario-10`: expected an error but execution succeeded
- `tck.clauses.set.set3.scenario-8`: expected [["<null>"]], observed []
- `tck.clauses.set.set4.scenario-2`: side effect +properties expected 2, observed 0
- `tck.clauses.set.set4.scenario-3`: side effect +properties expected 2, observed 0
- `tck.clauses.set.set4.scenario-5`: expected [["<null>"]], observed []
- `tck.clauses.set.set5.scenario-1`: expected [["<null>"]], observed []
- `tck.clauses.set.set5.scenario-2`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-1`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-2`: side effect +properties expected 1, observed 0
- `tck.clauses.set.set6.scenario-3`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-4`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-5`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-6`: side effect +properties expected 5, observed 0
- `tck.clauses.set.set6.scenario-7`: side effect +properties expected 5, observed 0
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
- `tck.clauses.unwind.unwind1.scenario-6`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 59..90; mutation execution failed: Cypher mutation binding failed: unknown variable `x` at byte 137..139; query:
UNWIND $events AS event
MATCH (y:Year {year: event.year})
MERGE (e:Event {id: event.id})
MERGE (y)<-[:IN]-(e)
RETURN e.id AS x
ORDER BY x
- `tck.clauses.unwind.unwind1.scenario-12`: query execution failed: Parse error: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 86..90; mutation execution failed: Cypher mutation binding failed: reusing a non-node variable in a node pattern is not supported in the initial graph slice at byte 86..90; query:
MATCH (a:S)-[:X]->(b1)
WITH a, collect(b1) AS bees
UNWIND bees AS b2
MATCH (a)-[:Y]->(b2)
RETURN a, b2
- `tck.clauses.unwind.unwind1.scenario-13`: expected [["1", "[1, 2]", "3", "[3, 4]", "5", "[5, 6]"], ["1", "[1, 2]", "3", "[3, 4]", "6", "[5, 6]"], ["1", "[1, 2]", "4", "[3, 4]", "5", "[5, 6]"], ["1", "[1, 2]", "4", "[3, 4]", "6", "[5, 6]"], ["2", "[1, 2]", "3", "[3, 4]", "5", "[5, 6]"], ["2", "[1, 2]", "3", "[3, 4]", "6", "[5, 6]"], ["2", "[1, 2]", "4", "[3, 4]", "5", "[5, 6]"], ["2", "[1, 2]", "4", "[3, 4]", "6", "[5, 6]"]], observed [["[1, 2]", "[3, 4]", "[5, 6]", "1", "3", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "3", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "4", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "1", "4", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "3", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "3", "6"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "4", "5"], ["[1, 2]", "[3, 4]", "[5, 6]", "2", "4", "6"]]
- `tck.clauses.with.with1.scenario-1`: expected [["(:A)", "(:B)"]], observed [["(:A)", "[:REL]", "(:B)"]]
- `tck.clauses.with.with1.scenario-2`: expected [["(:A)", "(:B)", "(:X)"]], observed [["(:A)", "(:X)", "[:REL]", "(:B)"]]
- `tck.clauses.with.with1.scenario-4`: expected [[""]], observed [["<()>"]]
- `tck.clauses.with.with6.scenario-4`: expected [["[( {num: 1}), ( {num: 2})]"], ["[( {num: 3}), ( {num: 4})]"]], observed [["[1, 2]"], ["[3, 4]"]]
- `tck.clauses.with.with6.scenario-7`: query execution failed: Parse error: unknown variable `me` at byte 55..57; mutation execution failed: Cypher mutation binding failed: unknown variable `me` at byte 55..57; query:
MATCH (me: Person)--(you: Person)
WITH me.age AS age, me.age + count(you.age) AS agg
RETURN *
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
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-1`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-2`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-9.examples-1-row-3`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:B {list: [1, 2], list2: [2, -2]})"], ["(:C {list: [300, 0], list2: [1, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:B {list: '[1,2]', list2: '[2,-2]'})"], ["(:C {list: '[300,0]', list2: '[1,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-10.examples-1-row-1`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:D {list: [1, -20], list2: [4, -2]})"], ["(:E {list: [2, -2, 100], list2: [5, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:D {list: '[1,-20]', list2: '[4,-2]'})"], ["(:E {list: '[2,-2,100]', list2: '[5,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-10.examples-1-row-2`: expected [["(:A {list: [2, -2], list2: [3, -2]})"], ["(:D {list: [1, -20], list2: [4, -2]})"], ["(:E {list: [2, -2, 100], list2: [5, -2]})"]], observed [["(:A {list: '[2,-2]', list2: '[3,-2]'})"], ["(:D {list: '[1,-20]', list2: '[4,-2]'})"], ["(:E {list: '[2,-2,100]', list2: '[5,-2]'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-1`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-2`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-15.examples-1-row-3`: expected [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:D {time: '12:35:15+05:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]], observed [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:E {time: '12:30:14.645876123+01:01'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-16.examples-1-row-1`: expected [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]], observed [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"], ["(:D {time: '12:35:15+05:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-16.examples-1-row-2`: expected [["(:A {time: '10:35-08:00'})"], ["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"]], observed [["(:B {time: '12:31:14.645876123+01:00'})"], ["(:C {time: '12:31:14.645876124+01:00'})"], ["(:D {time: '12:35:15+05:00'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-1`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-2`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby2.scenario-19.examples-1-row-3`: expected [["(:B {datetime: '1984-10-11T12:31:14.645876123+00:17'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]], observed [["(:A {datetime: '1984-10-11T12:30:14.000000012+00:15'})"], ["(:C {datetime: '0001-01-01T01:01:01.000000001-11:59'})"], ["(:E {datetime: '1980-12-11T12:31:14-11:59'})"]]
- `tck.clauses.with-orderby.withorderby4.scenario-11`: query execution failed: Parse error: unknown variable `a` at byte 79..80; mutation execution failed: Cypher mutation binding failed: unknown variable `a` at byte 79..80; query:
MATCH (a:A)
WITH a.num2 % 3 AS mod, sum(a.num + a.num2) AS sum
  ORDER BY sum(a.num + a.num2)
  LIMIT 2
RETURN mod, sum
- `tck.clauses.with-skip-limit.withskiplimit2.scenario-3`: expected [["(:B)", "(:A)", "(:X)"]], observed [["(:A)", "(:B)", "[:REL]", "(:X)"]]
- `tck.clauses.with-where.withwhere1.scenario-4`: expected [["(:B {id: 2})"]], observed []
- `tck.clauses.with-where.withwhere4.scenario-2`: query execution failed: Invalid argument supplied: graph snapshot 1 is not built; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 103..121; query:
MATCH (a), (b)
WITH a, b
WHERE a.id = 0
  AND (a)-[:T]->(b:TheLabel)
  OR (a)-[:T*]->(b:MissingLabel)
RETURN DISTINCT b
- `tck.expressions.aggregation.aggregation2.scenario-9`: expected [["[2, 1]"]], observed [["[2]"]]
- `tck.expressions.aggregation.aggregation2.scenario-11`: expected [["1"]], observed [["b"]]
- `tck.expressions.aggregation.aggregation2.scenario-12`: expected [["[1, 2]"]], observed [["0.2"]]
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
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-1`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-2`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-3`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-8.examples-1-row-4`: expected [["0", "1"]], observed [["<null>", "<null>"]]
- `tck.expressions.comparison.comparison1.scenario-12`: expected [], observed [["4611686018427387905"]]
- `tck.expressions.comparison.comparison1.scenario-13`: expected [], observed [["4611686018427387905"]]
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-1`: expected [["1", "3.14"]], observed [["", "[]"], ["", "{\"nodes\":[1,2],\"relationships\":[1]}"], ["", "{}"], ["1", "3.14"], ["1", "3.14"], ["1", "3.14"], ["1", "3.14"], ["[]", "{\"nodes\":[1,2],\"relationships\":[1]}"], ["[]", "{}"], ["{\"nodes\":[1,2],\"relationships\":[1]}", "{}"]]
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-2`: expected [["1", "3.14"]], observed [["", "[]"], ["", "{\"nodes\":[1,2],\"relationships\":[1]}"], ["", "{}"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "3.14"], ["1", "3.14"], ["1", "3.14"], ["1", "3.14"], ["[]", "{\"nodes\":[1,2],\"relationships\":[1]}"], ["[]", "{}"], ["{\"nodes\":[1,2],\"relationships\":[1]}", "{}"]]
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-3`: expected [["3.14", "1"]], observed [["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["1", "1"], ["3.14", "1"], ["3.14", "1"], ["3.14", "1"], ["3.14", "1"], ["[]", ""], ["{\"nodes\":[1,2],\"relationships\":[1]}", ""], ["{\"nodes\":[1,2],\"relationships\":[1]}", "[]"], ["{}", ""], ["{}", "[]"], ["{}", "{\"nodes\":[1,2],\"relationships\":[1]}"]]
- `tck.expressions.comparison.comparison2.scenario-3.examples-1-row-4`: expected [["3.14", "1"]], observed [["3.14", "1"], ["3.14", "1"], ["3.14", "1"], ["3.14", "1"], ["[]", ""], ["{\"nodes\":[1,2],\"relationships\":[1]}", ""], ["{\"nodes\":[1,2],\"relationships\":[1]}", "[]"], ["{}", ""], ["{}", "[]"], ["{}", "{\"nodes\":[1,2],\"relationships\":[1]}"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-1`: expected [["1"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-2`: expected [["1"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-3`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-4.examples-1-row-4`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-1`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-2`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.comparison.comparison2.scenario-5.examples-1-row-3`: expected [["0", "0", "0", "0"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.conditional.conditional2.scenario-1.examples-1-row-11`: expected [["something else"]], observed [["one"]]
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
- `tck.expressions.graph.graph3.scenario-7`: expected [["<null>", "<null>"]], observed []
- `tck.expressions.graph.graph4.scenario-3`: expected [["<null>", "<null>"]], observed [["NOT_THERE", "<null>"]]
- `tck.expressions.graph.graph4.scenario-4`: expected [["<null>"], ["T"]], observed [["T"], ["T"]]
- `tck.expressions.graph.graph4.scenario-5`: query execution failed: Parse error: no such function: type; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 39..60; query:
MATCH (a)-[r]->()
WITH [r, 1] AS list
RETURN type(list[0])
- `tck.expressions.graph.graph5.scenario-2`: expected [["[:T1]", "0"], ["[:T2]", "1"], ["[:T3]", "0"], ["[:T4]", "0"], ["[:t2]", "0"]], observed [["[:T1]", "0"], ["[:T2]", "0"], ["[:T3]", "0"], ["[:T4]", "0"], ["[:t2]", "0"]]
- `tck.expressions.graph.graph5.scenario-5`: expected [["<null>"]], observed [["0"]]
- `tck.expressions.graph.graph6.scenario-3`: expected [["<null>"]], observed []
- `tck.expressions.graph.graph6.scenario-4`: expected [["<null>", "<null>", "42"]], observed [["<null>", "<null>", "<null>"]]
- `tck.expressions.graph.graph6.scenario-6`: expected [["<null>", "<null>", "42"]], observed [["<null>", "<null>", "42"], ["<null>", "<null>", "<null>"]]
- `tck.expressions.graph.graph6.scenario-7`: expected [["<null>"]], observed []
- `tck.expressions.graph.graph6.scenario-8`: expected [["<null>", "<null>", "42"]], observed [["<null>", "<null>", "<null>"]]
- `tck.expressions.graph.graph7.scenario-1`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 32..33; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 25..56; query:
MATCH (n {name: 'Apa'})
RETURN n['nam' + 'e'] AS value
- `tck.expressions.graph.graph7.scenario-2`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..26; mutation execution failed: Cypher mutation binding failed: indexing this operand/key combination is not supported in the initial graph slice at byte 33..34; query:
CREATE (n {name: 'Apa'})
RETURN n['nam' + 'e'] AS value
- `tck.expressions.graph.graph7.scenario-3`: query execution failed: Parse error: mutation clauses in read queries is not supported in the initial graph slice at byte 1..26; mutation execution failed: Cypher mutation binding failed: indexing this operand/key combination is not supported in the initial graph slice at byte 33..34; query:
CREATE (n {name: 'Apa'})
RETURN n[$idx] AS value
- `tck.expressions.graph.graph9.scenario-1`: expected [["{level: 9001, name: 'Popeye'}"]], observed [["{\"name\":\"Popeye\",\"level\":9001}"]]
- `tck.expressions.graph.graph9.scenario-2`: expected [["{level: 9001, name: 'Popeye'}"]], observed [["{\"name\":\"Popeye\",\"level\":9001}"]]
- `tck.expressions.graph.graph9.scenario-3`: expected [["<null>", "<null>", "<null>"]], observed []
- `tck.expressions.graph.graph9.scenario-4`: expected [["{level: 9001, name: 'Popeye'}"]], observed [["{\"name\":\"Popeye\",\"level\":9001}"]]
- `tck.expressions.list.list1.scenario-9.examples-1-row-1`: expected an error but execution succeeded
- `tck.expressions.list.list12.scenario-1`: query execution failed: Parse error: property access requires a node or relationship at byte 69..70; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 69..70; query:
MATCH (a:Label1)
WITH collect(a) AS nodes
WITH nodes, [x IN nodes | x.name] AS oldNames
UNWIND nodes AS n
SET n.name = 'newName'
RETURN n.name, oldNames
- `tck.expressions.list.list12.scenario-2`: query execution failed: Parse error: property access requires a node or relationship at byte 73..74; mutation execution failed: Cypher mutation binding failed: property access requires a node or relationship at byte 73..74; query:
MATCH (a:Label1)
WITH collect(a) AS nodes
WITH nodes, [x IN nodes WHERE x.name = 'original'] AS noopFiltered
UNWIND nodes AS n
SET n.name = 'newName'
RETURN n.name, size(noopFiltered)
- `tck.expressions.list.list12.scenario-4`: expected [["[(:A), (:A)]"]], observed [["[1, 1]"]]
- `tck.expressions.list.list12.scenario-5`: expected [["[(:A), (:A)]", "2"]], observed [["[1, 1]", "2"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-1`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-2`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-3`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-4`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list2.scenario-9.examples-1-row-5`: expected [["<null>"]], observed [["[]"]]
- `tck.expressions.list.list4.scenario-1`: expected [["[1, 10, 100, 4, 5]"]], observed [["0"]]
- `tck.expressions.list.list6.scenario-3`: expected [["3"]], observed [["1"]]
- `tck.expressions.list.list6.scenario-7`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 36..36
- `tck.expressions.list.list6.scenario-8`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 34..34
- `tck.expressions.list.list6.scenario-9`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 38..38
- `tck.expressions.list.list6.scenario-10`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 44..44
- `tck.expressions.literals.literals5.scenario-6`: expected [["1e-305"]], observed [["9.999999999999999e-306"]]
- `tck.expressions.literals.literals5.scenario-9`: expected [["0.0"]], observed [["-0.0"]]
- `tck.expressions.literals.literals5.scenario-10`: expected [["0.0"]], observed [["-0.0"]]
- `tck.expressions.literals.literals5.scenario-12`: expected [["-1e-305"]], observed [["-9.999999999999999e-306"]]
- `tck.expressions.literals.literals6.scenario-5`: expected [["a\\\\bcn5t'\"\\\\//\\\\\"'"]], observed [["a\\bcn5t'\"\\//\\\"'"]]
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
- `tck.expressions.literals.literals8.scenario-11`: expected [["{k: -1e-6}"]], observed [["{\"k\":-1.0e-6}"]]
- `tck.expressions.literals.literals8.scenario-12`: expected [["{k: 'ab: c, as#?lßdj '}"]], observed [["{\"k\":\"ab: c, as#?lßdj \"}"]]
- `tck.expressions.literals.literals8.scenario-13`: expected [["{a: {}}"]], observed [["{\"a\":{}}"]]
- `tck.expressions.literals.literals8.scenario-14`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-15`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {a7: {a8: {a9: {a10: {a11: {a12: {a13: {a14: {a15: {a16: {a17: {a18: {a19: {}}}}}}}}}}}}}}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{\"a7\":{\"a8\":{\"a9\":{\"a10\":{\"a11\":{\"a12\":{\"a13\":{\"a14\":{\"a15\":{\"a16\":{\"a17\":{\"a18\":{\"a19\":{}}}}}}}}}}}}}}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-16`: expected [["{a1: {a2: {a3: {a4: {a5: {a6: {a7: {a8: {a9: {a10: {a11: {a12: {a13: {a14: {a15: {a16: {a17: {a18: {a19: {a20: {a21: {a22: {a23: {a24: {a25: {a26: {a27: {a28: {a29: {a30: {a31: {a32: {a33: {a34: {a35: {a36: {a37: {a38: {a39: {}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}"]], observed [["{\"a1\":{\"a2\":{\"a3\":{\"a4\":{\"a5\":{\"a6\":{\"a7\":{\"a8\":{\"a9\":{\"a10\":{\"a11\":{\"a12\":{\"a13\":{\"a14\":{\"a15\":{\"a16\":{\"a17\":{\"a18\":{\"a19\":{\"a20\":{\"a21\":{\"a22\":{\"a23\":{\"a24\":{\"a25\":{\"a26\":{\"a27\":{\"a28\":{\"a29\":{\"a30\":{\"a31\":{\"a32\":{\"a33\":{\"a34\":{\"a35\":{\"a36\":{\"a37\":{\"a38\":{\"a39\":{}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}"]]
- `tck.expressions.literals.literals8.scenario-17`: expected [["{a: ' { b : ', c: {d: ' '}, d: ' } '}"]], observed [["{\"a\":\" { b : \",\"c\":{\"d\":\" \"},\"d\":\" } \"}"]]
- `tck.expressions.literals.literals8.scenario-18`: expected [["{data: [{batters: {batter: [{id: '1001', type: 'Regular'}, {id: '1002', type: 'Chocolate'}, {id: '1003', type: 'Blueberry'}, {id: '1004', type: 'Devils Food'}]}, id: '0001', name: 'Cake', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5005', type: 'Sugar'}, {id: '5007', type: 'Powdered Sugar'}, {id: '5006', type: 'Chocolate Sprinkles'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}, {batters: {batter: [ {id: '1001', type: 'Regular'}]}, id: '0002', name: 'Raised', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5005', type: 'Sugar'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}, {batters: {batter: [{id: '1001', type: 'Regular'}, {id: '1002', type: 'Chocolate'}]}, id: '0003', name: 'Old Fashioned', ppu: 0.55, topping: [{id: '5001', type: 'None'}, {id: '5002', type: 'Glazed'}, {id: '5003', type: 'Chocolate'}, {id: '5004', type: 'Maple'}], type: 'donut'}]}"]], observed [["{\"data\":[{\"id\":\"0001\",\"type\":\"donut\",\"name\":\"Cake\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"},{\"id\":\"1003\",\"type\":\"Blueberry\"},{\"id\":\"1004\",\"type\":\"Devils Food\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5007\",\"type\":\"Powdered Sugar\"},{\"id\":\"5006\",\"type\":\"Chocolate Sprinkles\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0002\",\"type\":\"donut\",\"name\":\"Raised\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5005\",\"type\":\"Sugar\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]},{\"id\":\"0003\",\"type\":\"donut\",\"name\":\"Old Fashioned\",\"ppu\":0.55,\"batters\":{\"batter\":[{\"id\":\"1001\",\"type\":\"Regular\"},{\"id\":\"1002\",\"type\":\"Chocolate\"}]},\"topping\":[{\"id\":\"5001\",\"type\":\"None\"},{\"id\":\"5002\",\"type\":\"Glazed\"},{\"id\":\"5003\",\"type\":\"Chocolate\"},{\"id\":\"5004\",\"type\":\"Maple\"}]}]}"]]
- `tck.expressions.map.map1.scenario-5.examples-1-row-5`: expected identifier at byte 6..6
- `tck.expressions.map.map1.scenario-5.examples-1-row-6`: expected identifier at byte 6..6
- `tck.expressions.map.map2.scenario-1`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 40..44; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..59; query:
WITH $expr AS expr, $idx AS idx
RETURN expr[idx] AS value
- `tck.expressions.map.map2.scenario-2`: query execution failed: Parse error: indexing this operand/key combination is not supported in the initial graph slice at byte 40..44; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 33..69; query:
WITH $expr AS expr, $idx AS idx
RETURN expr[toString(idx)] AS value
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
- `tck.expressions.map.map3.scenario-2`: expected [["[\"address\",\"name\",\"age\"]"]], observed [["[\"address\",\"age\",\"name\"]"]]
- `tck.expressions.map.map3.scenario-3`: expected [["<null>", "<null>"]], observed [["[null]", "<null>"]]
- `tck.expressions.null.null1.scenario-3`: expected [["1"]], observed []
- `tck.expressions.null.null2.scenario-3`: expected [["0"]], observed []
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
- `tck.expressions.pattern.pattern2.scenario-1`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 31..31
- `tck.expressions.pattern.pattern2.scenario-2`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 35..35
- `tck.expressions.pattern.pattern2.scenario-3`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 41..41
- `tck.expressions.pattern.pattern2.scenario-4`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 32..32
- `tck.expressions.pattern.pattern2.scenario-5`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 32..32
- `tck.expressions.pattern.pattern2.scenario-6`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 45..45
- `tck.expressions.pattern.pattern2.scenario-7`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 65..65
- `tck.expressions.pattern.pattern2.scenario-8`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 35..35
- `tck.expressions.pattern.pattern2.scenario-9`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 42..42
- `tck.expressions.pattern.pattern2.scenario-10`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 39..39
- `tck.expressions.pattern.pattern2.scenario-11`: expected AND, OR, relationship_pattern, xor_op, comparison_op, predicate_suffix, additive_op, multiplicative_op, power_op, json_op, or postfix_suffix at byte 38..38
- `tck.expressions.precedence.precedence1.scenario-1`: expected [["1", "1", "0"]], observed [["0", "1", "0"]]
- `tck.expressions.precedence.precedence1.scenario-14`: expected [["1"]], observed [["0"]]
- `tck.expressions.precedence.precedence2.scenario-2.examples-1-row-1`: expected [["512.0", "512.0", "68719476736.0"]], observed [["512.0", "512.0", "1.109067877648326e130"]]
- `tck.expressions.precedence.precedence2.scenario-2.examples-1-row-2`: expected [["8.0", "8.0", "64.0"]], observed [["8.0", "8.0", "4.0"]]
- `tck.expressions.precedence.precedence2.scenario-2.examples-1-row-3`: expected [["0.0", "0.0", "64.0"]], observed [["0.0", "0.0", "4.0"]]
- `tck.expressions.precedence.precedence2.scenario-3.examples-1-row-1`: expected [["72.0", "72.0", "1073741824.0"]], observed [["72.0", "72.0", "1.8092513943330656e75"]]
- `tck.expressions.precedence.precedence2.scenario-3.examples-1-row-2`: expected [["56.0", "56.0", "64.0"]], observed [["56.0", "56.0", "4.0"]]
- `tck.expressions.precedence.precedence2.scenario-4`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence2.scenario-5.examples-1-row-1`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence2.scenario-5.examples-1-row-2`: expected not_expression at byte 49..49
- `tck.expressions.precedence.precedence3.scenario-1`: expected [["[[1], [2, 3], [4, 5], 10]", "[[1], [2, 3], [4, 5], 10]", "5"]], observed [["[[1], [2, 3], [4, 5], 10]", "[[1], [2, 3], [4, 5], 10]", "<null>"]]
- `tck.expressions.precedence.precedence3.scenario-2`: expected [["[[1], [2, 3], [4, 5], 8, 9]", "[[1], [2, 3], [4, 5], 8, 9]", "[4, 5]"]], observed [["[[1], [2, 3], [4, 5], [8, 9]]", "[[1], [2, 3], [4, 5], [8, 9]]", "<null>"]]
- `tck.expressions.precedence.precedence3.scenario-3`: expected [["[[1], [2, 3], [4, 5], [6, 7], [8, 9]]", "[[1], [2, 3], [4, 5], [6, 7], [8, 9]]", "[[2, 3], [4, 5]]"]], observed [["0", "0", "[]"]]
- `tck.expressions.precedence.precedence3.scenario-5`: expected [["0", "0", "[0, 4]", "[1, 0, 4]"]], observed [["1", "1", "0", "[1, 0, 4]"]]
- `tck.expressions.precedence.precedence4.scenario-4`: query execution failed: Parse error: string predicates on non-string operands is not supported in the initial graph slice at byte 146..160; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..334; query:
RETURN ('abc' STARTS WITH null OR true) = (('abc' STARTS WITH null) OR true) AS a,
       ('abc' STARTS WITH null OR true) <> ('abc' STARTS WITH (null OR true)) AS b,
       (true OR null STARTS WITH 'abc') = (true OR (null STARTS WITH 'abc')) AS c,
       (true OR null STARTS WITH 'abc') <> ((true OR null) STARTS WITH 'abc') AS d
- `tck.expressions.quantifier.quantifier1.scenario-8`: query execution failed: Parse error: property access requires a node or relationship at byte 99..100; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..123; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, none(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier1.scenario-9`: query execution failed: Parse error: property access requires a node or relationship at byte 154..155; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..178; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, none(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier2.scenario-8`: query execution failed: Parse error: property access requires a node or relationship at byte 101..102; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..125; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, single(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier2.scenario-9`: query execution failed: Parse error: property access requires a node or relationship at byte 156..157; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..180; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, single(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier3.scenario-8`: query execution failed: Parse error: property access requires a node or relationship at byte 98..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..122; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, any(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier3.scenario-9`: query execution failed: Parse error: property access requires a node or relationship at byte 153..154; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..177; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, any(x IN relationships WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier4.scenario-8`: query execution failed: Parse error: property access requires a node or relationship at byte 98..99; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 63..122; query:
MATCH p = (:SNodes)-[*0..3]->(x)
WITH tail(nodes(p)) AS nodes
RETURN nodes, all(x IN nodes WHERE x.name = 'a') AS result
- `tck.expressions.quantifier.quantifier4.scenario-9`: query execution failed: Parse error: property access requires a node or relationship at byte 153..154; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 102..177; query:
MATCH p = (:SRelationships)-[*0..4]->(x)
WITH tail(relationships(p)) AS relationships, COUNT(*) AS c
RETURN relationships, all(x IN relationships WHERE x.name = 'a') AS result
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
- `tck.expressions.string.string1.scenario-1`: expected [["123456789"]], observed [["0123456789"]]
- `tck.expressions.temporal.temporal1.scenario-1.examples-1-row-13`: expected [["1817-01-08"]], observed [["1816-01-10"]]
- `tck.expressions.temporal.temporal1.scenario-1.examples-1-row-14`: expected [["1817-01-07"]], observed [["1816-01-09"]]
- `tck.expressions.temporal.temporal1.scenario-2.examples-1-row-13`: expected [["1817-01-08T00:00"]], observed [["1816-01-10T00:00"]]
- `tck.expressions.temporal.temporal1.scenario-2.examples-1-row-14`: expected [["1817-01-07T00:00"]], observed [["1816-01-09T00:00"]]
- `tck.expressions.temporal.temporal1.scenario-3.examples-1-row-13`: expected [["1817-01-08T00:00Z"]], observed [["1816-01-10T00:00Z"]]
- `tck.expressions.temporal.temporal1.scenario-3.examples-1-row-14`: expected [["1817-01-07T00:00Z"]], observed [["1816-01-09T00:00Z"]]
- `tck.expressions.temporal.temporal1.scenario-11`: query execution failed: Parse error: invalid resolved function or parameter name: datetime.fromepoch; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 1..105; query:
RETURN datetime.fromepoch(416779, 999999999) AS d1,
       datetime.fromepochmillis(237821673987) AS d2
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-2`: expected [["P5M1DT12H"]], observed [["P5M1D"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-3`: expected [["P22DT19H51M49.5S"]], observed [["PT0S"]]
- `tck.expressions.temporal.temporal1.scenario-12.examples-1-row-4`: expected [["P17DT12H"]], observed [["P17D"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-1`: expected [["12:34:56+02:05"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-2`: expected [["12:34:56+02:05:59"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-3`: expected [["12:34:56-02:05:07"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal1.scenario-13.examples-1-row-4`: expected [["1984-10-11T12:34:56+02:05:59"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-13`: expected [["PT6H10M32.142S"]], observed [["PT7H10M32.142S"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-15`: expected [["PT1H"]], observed [["PT2H"]]
- `tck.expressions.temporal.temporal10.scenario-2.examples-1-row-25`: expected [["PT-4H-10M-36.143S"]], observed [["PT-5H-10M-36.143S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-13`: expected [["PT6H10M32.142S"]], observed [["PT7H10M32.142S"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-15`: expected [["PT1H"]], observed [["PT2H"]]
- `tck.expressions.temporal.temporal10.scenario-5.examples-1-row-25`: expected [["PT-4H-10M-36.143S"]], observed [["PT-5H-10M-36.143S"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-1`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-2`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-3`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-4`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-5`: expected [["PT5H"]], observed [["PT4H"]]
- `tck.expressions.temporal.temporal10.scenario-8.examples-1-row-6`: expected [["PT25H"]], observed [["PT24H"]]
- `tck.expressions.temporal.temporal10.scenario-9`: expected [["P1999999998Y11M30D"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-10`: expected [["PT17531639991215H59M59S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-4`: expected [["PT0S"]], observed [["PT0.000001S"]]
- `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-5`: expected [["PT0S"]], observed [["PT0.000001S"]]
- `tck.expressions.temporal.temporal2.scenario-6.examples-1-row-5`: expected [["1818-07-21T21:40:32.142+00:53:28[Europe/Stockholm]"]], observed [["1818-07-21T21:40:32.142+00:53[Europe/Stockholm]"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-2`: expected [["P5M1DT12H"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-3`: expected [["P22DT19H51M49.5S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-4`: expected [["PT45S"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-5`: expected [["P17DT12H"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal2.scenario-7.examples-1-row-7`: expected [["P2012Y2M2DT14H37M21.545S"]], observed [["<null>"]]
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
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-11`: expected [["12:31:14.645Z"]], observed [["12:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-16`: expected [["12:00+01:00"]], observed [["<null>"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-17`: expected [["12:00+01:00"]], observed [["12:00+02:00"]]
- `tck.expressions.temporal.temporal3.scenario-3.examples-1-row-19`: expected [["12:00:42+01:00"]], observed [["12:00:42+02:00"]]
- `tck.expressions.temporal.temporal3.scenario-7.examples-1-row-1`: query execution failed: Parse error: no such function: localdatetime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 119..157; query:
WITH localdatetime({year: 1984, week: 10, dayOfWeek: 3, hour: 12, minute: 31, second: 14, millisecond: 645}) AS other
RETURN localdatetime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-7.examples-1-row-4`: query execution failed: Parse error: no such function: localdatetime; mutation execution failed: Cypher mutation binding failed: projection clauses in mutation queries is not supported in the initial graph slice at byte 88..126; query:
WITH datetime({year: 1984, month: 10, day: 11, hour: 12, timezone: '+01:00'}) AS other
RETURN localdatetime(other) AS result
- `tck.expressions.temporal.temporal3.scenario-10.examples-1-row-32`: expected [["1984-03-28T00:00:42-10:00[Pacific/Honolulu]"]], observed [["1984-03-28T01:00:42-10:00[Pacific/Honolulu]"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-1`: expected [["1984-03-07T12:31:14.645Z"]], observed [["1984-03-07 12:31:14"]]
- `tck.expressions.temporal.temporal3.scenario-11.examples-1-row-6`: expected [["1984-10-11T12:00+01:00[Europe/Stockholm]"]], observed [["<null>"]]
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
- `tck.expressions.temporal.temporal5.scenario-1`: expected [["1984", "4", "10", "41", "1984", "11", "285", "4", "11"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-2`: expected [["1984", "1983", "52", "7"]], observed [["<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-3`: expected [["12", "31", "14", "645", "645876", "645876123"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-4`: expected [["12", "31", "14", "645", "645876", "645876123", "+01:00", "+01:00", "60", "3600"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-5`: expected [["1984", "4", "11", "45", "1984", "11", "316", "7", "42", "12", "31", "14", "645", "645876", "645876123"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-6`: expected [["1984", "4", "11", "45", "1984", "11", "316", "7", "42", "12", "31", "14", "645", "645876", "645876123", "Europe/Stockholm", "+01:00", "60", "3600", "469020674", "469020674645"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal5.scenario-7`: expected [["1", "5", "16", "1", "10", "1", "61", "3661", "3661111", "3661111111", "3661111111111", "1", "1", "4", "3", "1", "1", "111", "111111", "111111111"]], observed [["<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>", "<null>"]]
- `tck.expressions.temporal.temporal7.scenario-3.examples-1-row-1`: expected [["0", "1", "0", "1", "0"]], observed [["1", "0", "1", "0", "0"]]
- `tck.expressions.temporal.temporal8.scenario-1.examples-1-row-3`: expected [["1997-10-11", "1971-10-12"]], observed [["1997-09-25", "1971-10-28"]]
- `tck.expressions.temporal.temporal8.scenario-2.examples-1-row-3`: expected [["22:29:27.500000004", "02:33:00.499999998"]], observed [["05:14:54.000000004", "19:47:33.999999998"]]
- `tck.expressions.temporal.temporal8.scenario-3.examples-1-row-3`: expected [["22:29:27.500000004+01:00", "02:33:00.499999998+01:00"]], observed [["05:14:54.000000004+01:00", "19:47:33.999999998+01:00"]]
- `tck.expressions.temporal.temporal8.scenario-4.examples-1-row-3`: expected [["1997-10-11T22:29:27.500000004", "1971-10-12T02:33:00.499999998"]], observed [["1997-09-26T05:14:54.000000004", "1971-10-27T19:47:33.999999998"]]
- `tck.expressions.temporal.temporal8.scenario-5.examples-1-row-3`: expected [["1997-10-11T22:29:27.500000004+01:00", "1971-10-12T02:33:00.499999998+01:00"]], observed [["1997-09-26T05:14:54.000000004+01:00", "1971-10-27T19:47:33.999999998+01:00"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-3`: expected [["P25Y4M43DT50H11M23.500000004S", "P-6M-15DT-17H-45M-3.500000002S"]], observed [["P25Y4M28DT32H56M50.000000004S", "P-6MT-30M-30.000000002S"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-6`: expected [["P13Y15DT49H47M23.500000003S", "P-12Y-10M-43DT-18H-9M-3.500000003S"]], observed [["P13YT32H32M50.000000003S", "P-12Y-10M-28DT-54M-30.000000003S"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-7`: expected [["P25Y4M43DT50H11M23.500000004S", "P6M15DT17H45M3.500000002S"]], observed [["P25Y4M28DT32H56M50.000000004S", "P6MT30M30.000000002S"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-8`: expected [["P13Y15DT49H47M23.500000003S", "P12Y10M43DT18H9M3.500000003S"]], observed [["P13YT32H32M50.000000003S", "P12Y10M28DT54M30.000000003S"]]
- `tck.expressions.temporal.temporal8.scenario-6.examples-1-row-9`: expected [["P25Y10M58DT67H56M27.000000006S", "PT0S"]], observed [["P25Y10M28DT33H27M20.000000006S", "PT0S"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-1`: expected [["P12Y5M14DT16H13M10.000000001S", "P12Y5M14DT16H13M10.000000001S"]], observed [["0", "<null>"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-2`: expected [["P24Y10M28DT32H26M20.000000002S", "P6Y2M22DT13H21M8S"]], observed [["0", "<null>"]]
- `tck.expressions.temporal.temporal8.scenario-7.examples-1-row-3`: expected [["P6Y2M22DT13H21M8S", "P24Y10M28DT32H26M20.000000002S"]], observed [["0.0", "<null>"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-43`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-45`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-1.examples-1-row-47`: expected [["1984-10-09"]], observed [["1984-10-08"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-64`: expected [["1984-10-09T00:00Z"]], observed [["1984-10-08T00:00Z"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-67`: expected [["1984-10-09T00:00+01:00"]], observed [["1984-10-08T00:00+01:00"]]
- `tck.expressions.temporal.temporal9.scenario-2.examples-1-row-70`: expected [["1984-10-09T00:00Z"]], observed [["1984-10-08T00:00Z"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-43`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-45`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
- `tck.expressions.temporal.temporal9.scenario-3.examples-1-row-47`: expected [["1984-10-09T00:00"]], observed [["1984-10-08T00:00"]]
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
- `tck.usecases.triadicselection.triadicselection1.scenario-1`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-2`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-3`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-4`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-5`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-6`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-7`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-8`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-9`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-10`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-11`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-12`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-13`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-14`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-15`: TCK named graph `binary-tree-1` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-16`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-17`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-18`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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
- `tck.usecases.triadicselection.triadicselection1.scenario-19`: TCK named graph `binary-tree-2` setup failed: query execution failed: Parse error: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; mutation execution failed: Cypher parse failed: expected EOI, UNION, clause, or relationship_pattern at byte 841..841; query: CREATE (a:A {name: 'a'}),
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

- Runs: 219
- Result records: 440471
- Unique test identities: 10291
