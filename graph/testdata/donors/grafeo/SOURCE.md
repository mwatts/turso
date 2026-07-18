# Grafeo test source

- Repository: <https://github.com/GrafeoDB/grafeo>
- Revision: `4ebae02f06f8f0cbc57543f74b6ba06f259dbed3`
- Upstream path: `tests/`
- License: Apache-2.0
- Adaptation: vendored test corpus; `.gtest` files are unmodified

All 157 source test manifests are retained for auditability. The importer
selects cases whose declared language is Cypher and Cypher variants from
multi-language Rosetta manifests; non-Cypher cases remain source material but
do not count as Cypher conformance scenarios.

The applicable upstream license and NOTICE are retained under
`licenses/graph/grafeo-apache-license.md` and
`licenses/graph/grafeo-notice.md`.
