# SparrowDB test source

- Repository: https://github.com/ryaker/SparrowDB
- Revision: `82d85b7a861dfb2e127452ed89eebbcee74bfef0`
- Imported path: `crates/sparrowdb/tests/`
- License: MIT (see `LICENSE`)
- Adaptation: Rust syntax-tree extraction selects literal Cypher passed to
  `execute`, `query`, `prepare`, or `execute_query`, including literals reached
  through a local binding. The vendored Rust files are otherwise unmodified.
