# Cypher parser gap histograms and quality impact

This report reflects the five-source corpus after removing the entire
LadybugDB/Kuzu suite, its 15,940 source identities, four curated fixture
adaptations, loader, CLI commands, history records, and vendored files. The
suite was removed as a unit because standard-looking queries still depended on
vendor datasets, types, functions, and result semantics; a statement-prefix
filter could not provide a reliable compatibility boundary.

## Current corpus

| Source | Imported identities | Canonical contracts | Exact duplicates |
| --- | ---: | ---: | ---: |
| openCypher TCK | 3,926 | 3,914 | 12 |
| Apache AGE | 3,677 | 3,057 | 620 |
| SparrowDB | 2,253 | 1,347 | 906 |
| Grafeo | 399 | 390 | 9 |
| CQLite | 137 | 115 | 22 |
| **Total** | **10,392** | **8,823** | **1,569** |

## Remaining source-specific syntax histogram

| Source | Identities | Share | Cypher/TCK impact |
| --- | ---: | ---: | --- |
| Apache AGE | 60 | 57.1% | None for core grammar: AGE-specific `EXPLAIN (...)` directives wrap an underlying query. |
| Grafeo | 22 | 21.0% | Low to medium: graph index and constraint DDL matters only if Turso adopts a compatible schema surface. |
| SparrowDB | 19 | 18.1% | Low: checkpoint and schema-administration statements are outside core Cypher queries. |
| CQLite | 4 | 3.8% | None: bare `WHERE` fragments are incomplete extraction products. |
| openCypher TCK | 0 | 0.0% | No TCK identity is in this bucket. |
| **Total** | **105** | **100%** | |

These should be removed or normalized by their source adapters, not added to
the core Cypher grammar to improve a headline count.

## Expression and projection provenance

| Source | Identities | Share | Cypher/TCK impact |
| --- | ---: | ---: | --- |
| openCypher TCK | 1,470 | 48.5% | Direct and normative. |
| Apache AGE | 1,319 | 43.5% | Medium to high after excluding AGE/PostgreSQL extensions. |
| SparrowDB | 154 | 5.1% | Secondary regression evidence. |
| Grafeo | 87 | 2.9% | Independent evidence where it agrees with the TCK. |
| CQLite | 1 | less than 0.1% | Low by itself. |
| **Total** | **3,031** | **100%** | |

Unlike source-specific administration syntax, this is a genuine Cypher
quality backlog. Prioritize TCK clusters first and implement each through
parsing, binding, lowering, execution, and result comparison. The most useful
early clusters are list predicates, map/list expressions, and projection
continuation because they recur across multiple retained sources.
