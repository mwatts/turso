### Task 2: Role-shaped relationship source registration

**Files:**
- Modify: `graph/frontend/src/catalog.rs:32-41` (`RelationshipSourceRegistration`), `:58-68` (`RegisteredRelationshipSource`), `:219-259` (load), `:265-495` (register), `:512-529` (`create_catalog`), `:570-586` (validation), `:745-796` (indexes)
- Modify: `graph/frontend/src/lib.rs:39`
- Test: `graph/frontend/src/catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `turso_graph_ir::{RoleId, RoleCardinality}` from Task 1.
- Produces:
  - `RoleSourceRegistration { name: String, column: String, node_source: String, cardinality: RoleCardinality }`
  - `RelationshipSourceRegistration { name: String, table: String, identity_column: String, roles: Vec<RoleSourceRegistration> }` — `start_column`, `end_column`, `start_node_source`, `end_node_source` are gone.
  - `RelationshipSourceRegistration::binary(name, table, identity_column, start_column, end_column, start_node_source, end_node_source) -> Self` — builds the two required single-valued roles named `start` and `end`.
  - `RegisteredRelationshipRole { role: ir::RoleId, name: String, column: String, node_source: ir::SourceTableId, cardinality: RoleCardinality }`
  - `RegisteredRelationshipSource { id, name, table, identity_column, roles: Vec<RegisteredRelationshipRole> }`
  - `RegisteredRelationshipSource::role_by_name(&self, name: &str) -> Option<&RegisteredRelationshipRole>` (case-insensitive)
  - `RELATIONSHIP_ROLES_TABLE: &str = "__turso_internal_graph_relationship_roles"`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/catalog.rs`, inside `mod tests`, add:

```rust
    #[test]
    fn a_two_role_registration_lands_on_todays_physical_shape() {
        // Binary is a layout of the role model, not a separate kind. The
        // registration that used to name start_column/end_column must produce
        // the same two indexed columns plus the composite pair index, or every
        // donor corpus source silently changes its access path.
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        let source = &graph.relationship_sources[0];
        assert_eq!(source.roles.len(), 2);
        assert_eq!(source.roles[0].name, "start");
        assert_eq!(source.roles[0].column, "src");
        assert_eq!(source.roles[0].cardinality, RoleCardinality::One);
        assert_eq!(source.roles[1].name, "end");
        assert_eq!(source.roles[1].column, "dst");
        assert!(source.role_by_name("START").is_some(), "role lookup is case-insensitive");

        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // One per role plus the composite pair index: exactly today's three.
        assert_eq!(indexes.len(), 3);
    }

    #[test]
    fn a_three_role_registration_indexes_every_role_and_every_ordered_pair() {
        let connection = connection();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY); \
                 CREATE TABLE texts(id INTEGER PRIMARY KEY); \
                 CREATE TABLE folios(id INTEGER PRIMARY KEY); \
                 CREATE TABLE transcriptions(\
                     id INTEGER PRIMARY KEY, scribe INTEGER, txt INTEGER, folio INTEGER);",
            )
            .expect("create ternary sources");
        let graph = register_graph(
            &connection,
            &GraphRegistration {
                name: "scriptorium".to_owned(),
                node_sources: vec![
                    NodeSourceRegistration {
                        name: "Person".to_owned(),
                        table: "people".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Text".to_owned(),
                        table: "texts".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Folio".to_owned(),
                        table: "folios".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                ],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "Transcription".to_owned(),
                    table: "transcriptions".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "scribe".to_owned(),
                            column: "scribe".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "text".to_owned(),
                            column: "txt".to_owned(),
                            node_source: "Text".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "folio".to_owned(),
                            column: "folio".to_owned(),
                            node_source: "Folio".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                    ],
                }],
            },
        )
        .expect("register ternary graph");

        assert_eq!(graph.relationship_sources[0].roles.len(), 3);
        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // Three single-role indexes plus one composite per unordered role pair
        // (scribe,text), (scribe,folio), (text,folio).
        assert_eq!(indexes.len(), 6);
    }

    #[test]
    fn a_role_must_name_a_registered_node_source() {
        let connection = connection();
        create_sources(&connection);
        let mut graph = registration("bad_endpoint");
        graph.relationship_sources[0].roles[1].node_source = "Missing".to_owned();
        assert!(matches!(
            register_graph(&connection, &graph),
            Err(CatalogError::UnknownEndpoint { relationship, node_source })
                if relationship == "KNOWS" && node_source == "Missing"
        ));
    }
```

Change the shared `registration` helper in the same test module to the
constructor form:

```rust
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS", "friendships", "id", "src", "dst", "Person", "Person",
            )],
```

and delete the now-unused `relationship_endpoints_must_name_registered_node_sources`
test (replaced by `a_role_must_name_a_registered_node_source`) and the endpoint
assertions inside `registration_installs_stable_sources_indexes_and_generation_triggers`
that duplicate `a_two_role_registration_lands_on_todays_physical_shape`.

Add `use turso_graph_ir::RoleCardinality;` to the test module imports.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: FAIL to compile with `no function or associated item named binary found` and `struct RelationshipSourceRegistration has no field named roles`.

- [ ] **Step 3: Replace the registration and registered types**

In `graph/frontend/src/catalog.rs`, replace lines 32-41 and 58-68 with:

```rust
/// One named role of a relationship source and the physical column that
/// stores its player.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleSourceRegistration {
    pub name: String,
    /// Endpoint column on the relationship table. Ignored for `Many` roles,
    /// which store players in `<table>__<role>` instead; pass an empty string.
    pub column: String,
    /// Name of a registered node source.
    pub node_source: String,
    pub cardinality: RoleCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
    /// Declaration order is stable and becomes role ordinal order.
    pub roles: Vec<RoleSourceRegistration>,
}

impl RelationshipSourceRegistration {
    /// A two-endpoint table registered as a two-role relation named
    /// `start`/`end`. This is a layout of the role model, not a separate kind:
    /// every donor corpus source registers this way and keeps working.
    #[allow(clippy::too_many_arguments)]
    pub fn binary(
        name: impl Into<String>,
        table: impl Into<String>,
        identity_column: impl Into<String>,
        start_column: impl Into<String>,
        end_column: impl Into<String>,
        start_node_source: impl Into<String>,
        end_node_source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            identity_column: identity_column.into(),
            roles: vec![
                RoleSourceRegistration {
                    name: "start".to_owned(),
                    column: start_column.into(),
                    node_source: start_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
                RoleSourceRegistration {
                    name: "end".to_owned(),
                    column: end_column.into(),
                    node_source: end_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipRole {
    pub role: ir::RoleId,
    pub name: String,
    pub column: String,
    pub node_source: SourceTableId,
    pub cardinality: RoleCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipSource {
    pub id: SourceTableId,
    pub name: String,
    pub table: String,
    pub identity_column: String,
    pub roles: Vec<RegisteredRelationshipRole>,
}

impl RegisteredRelationshipSource {
    pub fn role_by_name(&self, name: &str) -> Option<&RegisteredRelationshipRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    pub fn role_by_id(&self, role: ir::RoleId) -> Option<&RegisteredRelationshipRole> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    /// Roles stored in an endpoint column on the relation table.
    pub fn single_valued_roles(&self) -> impl Iterator<Item = &RegisteredRelationshipRole> {
        self.roles
            .iter()
            .filter(|role| role.cardinality == RoleCardinality::One)
    }

    /// Spill table holding the players of a `Many` role.
    pub fn spill_table(&self, role: &RegisteredRelationshipRole) -> String {
        format!("{}__{}", self.table, role.name)
    }
}
```

Change the imports at the top of the file to
`use turso_graph_ir::{self as ir, GraphId, RoleCardinality, SourceTableId};`.

- [ ] **Step 4: Add the role catalog table**

In `create_catalog` (`catalog.rs:512`), drop the endpoint columns from
`RELATIONSHIP_SOURCES_TABLE` and add the roles table:

```rust
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_ROLES_TABLE}(source_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE, column_name TEXT NOT NULL, node_source_id INTEGER NOT NULL, cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), PRIMARY KEY(source_id, ordinal))"
    ))?;
```

and declare the constant beside the others (`catalog.rs:21`):

```rust
pub(crate) const RELATIONSHIP_ROLES_TABLE: &str = "__turso_internal_graph_relationship_roles";
```

- [ ] **Step 5: Write the role rows and indexes during registration**

Replace the relationship-source loop body (`catalog.rs:358-417`) with:

```rust
    for relationship in &registration.relationship_sources {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'relationship')",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
        )?;
        let relationship_id = scalar_integer(
            connection,
            &format!(
                "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
            "relationship source id",
        )?;
        execute_internal(connection, format!(
            "INSERT INTO {RELATIONSHIP_SOURCES_TABLE}(source_id, table_name, identity_column) VALUES ({}, {}, {})",
            relationship_id,
            sql_string(&relationship.table),
            sql_string(&relationship.identity_column)
        ))?;
        for (ordinal, role) in relationship.roles.iter().enumerate() {
            let node_source = node_ids.get(&role.node_source).ok_or_else(|| {
                CatalogError::UnknownEndpoint {
                    relationship: relationship.name.clone(),
                    node_source: role.node_source.clone(),
                }
            })?;
            execute_internal(connection, format!(
                "INSERT INTO {RELATIONSHIP_ROLES_TABLE}(source_id, ordinal, name, column_name, node_source_id, cardinality) \
                 VALUES ({}, {}, {}, {}, {}, {})",
                relationship_id,
                ordinal + 1,
                sql_string(&role.name),
                sql_string(&role.column),
                node_source.get(),
                sql_string(match role.cardinality {
                    RoleCardinality::One => "one",
                    RoleCardinality::Many => "many",
                })
            ))?;
            match role.cardinality {
                RoleCardinality::One => {
                    install_role_index(connection, graph_id, relationship, role)?;
                }
                RoleCardinality::Many => {
                    install_spill_table(connection, graph_id, relationship, role)?;
                }
            }
        }
        // Co-membership patterns bind two role players before matching the
        // second relationship; the composite index turns that probe from an
        // in-degree scan into an exact lookup. A two-role relation gets
        // exactly one such index, which is today's (start, end) index.
        install_role_pair_indexes(connection, graph_id, relationship)?;
    }
```

- [ ] **Step 6: Replace the index installers**

Replace `install_endpoint_index` and `install_endpoint_pair_index`
(`catalog.rs:745-796`) with:

```rust
fn install_role_index(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Result<(), CatalogError> {
    let name = format!(
        "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_{}_{:016x}",
        graph.get(),
        role.name.to_ascii_lowercase(),
        stable_hash(&format!("{}:{}", source.table, role.column))
    );
    execute_internal(
        connection,
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({})",
            quote_identifier(&name),
            quote_identifier(&source.table),
            quote_identifier(&role.column)
        ),
    )?;
    Ok(())
}

/// One composite index per unordered pair of single-valued roles. A two-role
/// relation therefore gets exactly the (start, end) index it has today.
fn install_role_pair_indexes(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
) -> Result<(), CatalogError> {
    let single = source
        .roles
        .iter()
        .filter(|role| role.cardinality == RoleCardinality::One)
        .collect::<Vec<_>>();
    for (index, left) in single.iter().enumerate() {
        for right in single.iter().skip(index + 1) {
            let name = format!(
                "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_pair_{:016x}",
                graph.get(),
                stable_hash(&format!("{}:{}:{}", source.table, left.column, right.column))
            );
            execute_internal(
                connection,
                format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {}({}, {})",
                    quote_identifier(&name),
                    quote_identifier(&source.table),
                    quote_identifier(&left.column),
                    quote_identifier(&right.column)
                ),
            )?;
        }
    }
    Ok(())
}

/// A `Many` role stores its players in `<table>__<role>(relation_id, node_id)`,
/// indexed in both directions so a hop is an index probe from either side.
fn install_spill_table(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Result<(), CatalogError> {
    let table = format!("{}__{}", source.table, role.name);
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS {}(relation_id INTEGER NOT NULL, node_id INTEGER NOT NULL)",
            quote_identifier(&table)
        ),
    )?;
    for (suffix, columns) in [("fwd", "relation_id, node_id"), ("rev", "node_id, relation_id")] {
        let name = format!(
            "{TURSO_GRAPH_CATALOG_PREFIX}spill_{}_{suffix}_{:016x}",
            graph.get(),
            stable_hash(&table)
        );
        execute_internal(
            connection,
            format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({columns})",
                quote_identifier(&name),
                quote_identifier(&table)
            ),
        )?;
    }
    Ok(())
}
```

- [ ] **Step 7: Load roles back**

Replace the relationship-loading block (`catalog.rs:219-251`) with:

```rust
    let relationship_rows = query_rows(
        connection,
        &format!(
            "SELECT s.id, s.name, r.table_name, r.identity_column FROM {SOURCES_TABLE} s \
             JOIN {RELATIONSHIP_SOURCES_TABLE} r ON r.source_id = s.id \
             WHERE s.graph_id = {} ORDER BY s.id",
            graph_id.get()
        ),
    )?;
    let mut relationship_sources = Vec::with_capacity(relationship_rows.len());
    for row in relationship_rows {
        let id = source_id(integer(&row, 0, "relationship source id")?)?;
        let table = text(&row, 2, "relationship source table")?.to_owned();
        let identity_column = text(&row, 3, "relationship identity column")?.to_owned();
        let role_rows = query_rows(
            connection,
            &format!(
                "SELECT ordinal, name, column_name, node_source_id, cardinality \
                 FROM {RELATIONSHIP_ROLES_TABLE} WHERE source_id = {} ORDER BY ordinal",
                id.get()
            ),
        )?;
        let mut roles = Vec::with_capacity(role_rows.len());
        let mut required_columns = vec![identity_column.clone()];
        for role_row in role_rows {
            let ordinal = integer(&role_row, 0, "role ordinal")?;
            let role = u32::try_from(ordinal)
                .ok()
                .and_then(|value| ir::RoleId::new(value).ok())
                .ok_or(CatalogError::InvalidIdentity {
                    kind: "role",
                    value: ordinal,
                })?;
            let cardinality = match text(&role_row, 4, "role cardinality")? {
                "one" => RoleCardinality::One,
                "many" => RoleCardinality::Many,
                _ => return Err(CatalogError::InvalidCatalogValue("role cardinality")),
            };
            let column = text(&role_row, 2, "role column")?.to_owned();
            if cardinality == RoleCardinality::One {
                required_columns.push(column.clone());
            }
            roles.push(RegisteredRelationshipRole {
                role,
                name: text(&role_row, 1, "role name")?.to_owned(),
                column,
                node_source: source_id(integer(&role_row, 3, "role node source id")?)?,
                cardinality,
            });
        }
        let borrowed = required_columns.iter().map(String::as_str).collect::<Vec<_>>();
        require_columns(connection, &table, &borrowed)?;
        relationship_sources.push(RegisteredRelationshipSource {
            id,
            name: text(&row, 1, "relationship source name")?.to_owned(),
            table,
            identity_column,
            roles,
        });
    }
```

Note the role identity is the ordinal: role identities are per relation type,
so the ordinal is already a stable non-zero `u32` within its source.

- [ ] **Step 8: Update validation**

In `validate_registration_names` (`catalog.rs:570-586`), replace the
relationship arm with:

```rust
    for source in &registration.relationship_sources {
        validate_name("relationship source", &source.name)?;
        let mut columns = vec![source.identity_column.as_str()];
        let mut role_names = HashSet::new();
        for role in &source.roles {
            validate_name("role", &role.name)?;
            if !role_names.insert(role.name.to_ascii_lowercase()) {
                return Err(CatalogError::DuplicateName {
                    kind: "role",
                    name: role.name.clone(),
                });
            }
            if role.cardinality == RoleCardinality::One {
                columns.push(role.column.as_str());
            }
        }
        validate_source_identifiers(&source.table, &columns)?;
        if !source_names.insert(source.name.to_ascii_lowercase()) {
            return Err(CatalogError::DuplicateName {
                kind: "relationship source",
                name: source.name.clone(),
            });
        }
    }
```

and in `register_graph_in_transaction` (`catalog.rs:286-303`) replace the
`require_columns` call for relationships with the same
identity-plus-single-valued-role column list.

- [ ] **Step 9: Update every construction site**

Replace the struct literal with `RelationshipSourceRegistration::binary(...)` in:
`graph/testkit/src/runner.rs:211`, `tests/integration/multi_frontend_doc.rs:230`,
`graph/frontend/src/snapshot.rs:915`, `graph/frontend/src/schema_catalog.rs:1016`,
`graph/frontend/src/graph_expand.rs:633`, `graph/frontend/src/dialect.rs:414`,
`graph/frontend/src/session.rs:601` and `:1199`,
`graph/frontend/benches/semantic_prepare.rs:52`, `:128`, `:137`,
`graph/frontend/examples/snapshot_profile.rs:52`,
`graph/frontend/tests/native_capabilities.rs:893`, and the multi-source literals
in `graph/frontend/src/catalog.rs:1116` and `:1125`.

Update `graph/frontend/src/lib.rs:39` to export the new names:

```rust
    RegisteredRelationshipRole, RegisteredRelationshipSource, RelationshipSourceRegistration,
    RoleSourceRegistration, GRAPH_CATALOG_VERSION,
```

- [ ] **Step 10: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: PASS, including both new role tests.

- [ ] **Step 11: Full gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/catalog: register relationship sources as roles

A relationship source now declares an ordered list of named roles, each with
an endpoint column, a target node source, and a cardinality. Binary is not a
separate kind: RelationshipSourceRegistration::binary builds the two required
single-valued roles named start/end, so a two-endpoint table lands on exactly
the two indexed columns plus composite pair index it had before.

Many-valued roles spill to <table>__<role>(relation_id, node_id), indexed in
both directions.

Tests: catalog unit tests assert the two-role physical shape is unchanged and
that a three-role source indexes every role and every ordered pair; corpus at
8,926 with no new failure family."
```

---

