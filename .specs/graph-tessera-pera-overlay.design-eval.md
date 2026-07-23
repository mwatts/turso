# Moved: Foedus Turso Ontology-Store Specification

The integration design moved to the Foedus repository on 2026-07-23. The
adapter implements Foedus storage ports and participates in Foedus runtime
composition, so Foedus owns it. Tessera remains storage-neutral.

Authoritative location:
`foedus/docs/superpowers/specs/2026-07-23-turso-ontology-store-design.md`
(sibling checkout:
`~/code/github/mwatts/foedus/docs/superpowers/specs/2026-07-23-turso-ontology-store-design.md`).

The dependency direction is Foedus adapter → Tessera + Turso graph frontend.
Neither Tessera nor Turso depends on the adapter. Turso-side requirements
remain in this repository:

- `.specs/graph-semantic-schema-overlay.agent-spec.md` — the
  semantic-schema catalog requirements (Milestones 1-4, amended).
- `.specs/graph-native-capabilities.agent-spec.md` — procedures, FTS,
  endpoint functions, snapshot diagnostics.
- `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`
  — the implementation plan for Milestones 1-2.

The Tessera repository retains
`.specs/tessera-turso.design-spec.md` only as a relocation pointer.
