# openCypher TCK source

- Repository: <https://github.com/rustic-ai/uni-db>
- Revision: `0812a496c62769b67cf688930750ae384e3de68d`
- Upstream path: `crates/uni-tck/tck/`
- License: Apache-2.0
- Adaptation: vendored test corpus; feature text and schema sidecars are
  unmodified

This directory contains the complete TCK copy used by Uni at the pinned
revision: 221 Gherkin feature files plus their 221 schema sidecars. Scenario
outlines are expanded by `turso_graph_testkit`; the source files are retained so
discovery, identities, examples, and provenance remain independently auditable.

The per-file openCypher copyright, license, and attribution headers remain in
the feature files. The applicable Apache-2.0 license is also retained under
`licenses/graph/uni-db-apache-license.md`.
