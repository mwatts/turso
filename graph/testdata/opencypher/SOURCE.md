# openCypher grammar and testable CIPs

- Source: https://github.com/opencypher/openCypher
- Revision: 677cbafabb8c3c5eed458fd3b1ec0daec8d67d23
- License: Apache-2.0 (see LICENSE)
- Contents:
  - `grammar/openCypher.bnf` — the canonical ISO WG3 BNF grammar, vendored
    for parser-surface tracking and as a fuzzing seed. Upstream tracks the
    ISO/IEC 39075 GQL grammar where possible (e.g. SHORTEST).
  - `cip-testable/` — Cypher Improvement Proposals adopted upstream but not
    yet covered by TCK scenarios: dynamic property lookup, date-time,
    STARTS WITH/ENDS WITH, parameter syntax, type-conversion functions.
    These document semantics the conformance corpus does not exercise.

The TCK feature files are vendored separately under
`graph/testdata/tck/opencypher/` and were verified byte-identical to this
revision's `tck/features/` (plus one local `@extension` scenario).
