# Apache AGE regression source

- Repository: <https://github.com/apache/age>
- Revision: `6876abcab0a3281eb65a7e2a91238e0b5abfdea7`
- Upstream paths: `regress/sql/`, `regress/expected/`
- License: Apache-2.0
- Adaptation: vendored regression corpus; SQL and expected files are unmodified

All 47 SQL regression files and their expected-output files are retained. The
importer extracts every dollar-quoted query supplied to AGE's `cypher(...)`
wrapper, assigns a stable file-and-ordinal identity, and intersects identical
queries through the shared parser cache. PostgreSQL wrapper statements that do
not contain Cypher source are not counted as Cypher scenarios.

The applicable upstream license and NOTICE are retained under
`licenses/graph/apache-age-apache-license.md` and
`licenses/graph/apache-age-notice.md`.
