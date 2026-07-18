# CQLite test source

- Repository: https://github.com/cqlite/cqlite
- Revision: `e2b677e8429a4cb0ead087ffbd9195f4f3999819`
- Imported path: `tests/`
- License: MIT (see `LICENSE`)
- Adaptation: Rust syntax-tree extraction selects literal Cypher passed to
  `execute`, `query`, `prepare`, or `execute_query`, including literals reached
  through a local binding. The vendored Rust files are otherwise unmodified.

