# Ladybug test source

- Repository: <https://github.com/mwatts/ladybug>
- Revision: `7eab431c6becf64f58f7c2ff4c0fb1f160acb492`
- Upstream path: `test/test_files/`
- License: MIT
- Adaptation: vendored test corpus; `.test` files are unmodified

The 477 source files contain 15,937 `-STATEMENT` assertions. The importer uses
each statement assertion as a stable result identity, retains its dataset,
case/log context and expected-output contract, and intersects exact duplicate
contracts before parser execution.

The applicable upstream license is retained under
`licenses/graph/ladybug-mit-license.md`.
