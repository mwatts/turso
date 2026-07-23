# Moved: Tessera Semantic Overlay Specification

This document moved to the tessera repository on 2026-07-22, because
the adapter it specifies (tessera-turso) is built there.

New location: `tessera/.specs/tessera-turso.design-spec.md`
(sibling checkout: `~/code/github/mwatts/tessera/.specs/tessera-turso.design-spec.md`).

The move follows the layering rule the specification itself defines:
tessera-turso depends on Turso, never the inverse, so the adapter's
design lives with the adapter. Turso-side requirements remain in this
repository:

- `.specs/graph-semantic-schema-overlay.agent-spec.md` — the
  semantic-schema catalog requirements (Milestones 1-4, amended).
- `.specs/graph-native-capabilities.agent-spec.md` — procedures, FTS,
  endpoint functions, snapshot diagnostics.
- `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`
  — the implementation plan for Milestones 1-2.

Frequently referenced sections at the new location: 7 (physical
mapping and identity policy), 8.4 (fragment-interface polymorphism),
8.5 (reification), 8.6 (search and embeddings), 11.2 (combined
cross-stream ordering), 14 (tessera-turso work breakdown), 15 (foedus
integration).
