//! Alignment regressions for GraphDialect + compile seams.

mod fixture;

use std::sync::{atomic::Ordering, Arc};

use turso_core::{DatabaseOpts, MemoryIO, OpenFlags};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, install_graph_catalog, lower_relational, open_database_with_io, register_graph,
    take_closed_create_fast_path_hit, CatalogEntity, GraphCatalogSnapshot, GraphConnection,
    GraphHostMode, GraphRegistration, NodeSourceRegistration, NodeTableLayout, ParameterTypes,
    Parameters, RelationalCatalogSnapshot, RelationshipRoleLayout, RelationshipSourceRegistration,
    RelationshipTableLayout, ResolvedProperty, SnapshotStore, Value, GRAPH_EXPAND_TABLE_NAME,
};
use turso_graph_ir::{
    Binding, BindingId, Direction, FixedExpand, GraphId, LabelId, NodeScan, Nullability, Plan,
    PlanKind, PropertyId, RelationshipTypeId, ResultShape, RoleCardinality, RoleId, Scope,
    SourceTableId, ValueType,
};

#[test]
fn prepare_returns_cypher_result_types_for_boolean_projection() {
    let (_database, graph) = fixture::social_graph_connection();
    let stmt = graph
        .prepare(
            "MATCH (n:Person) RETURN n.name AS name, true AS flag",
            &Parameters::new(),
        )
        .expect("prepare");
    let types = stmt.result_types();
    assert!(
        !types.is_empty(),
        "result_types must come from the single compile path"
    );
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], ValueType::Text);
    assert_eq!(types[1], ValueType::Boolean);
    assert_eq!(stmt.num_columns(), 2);

    let rows = graph
        .query(
            "MATCH (n:Person {name: 'Ada'}) RETURN n.name AS name, true AS flag",
            &Parameters::new(),
        )
        .expect("query");
    assert_eq!(
        rows,
        vec![vec![Value::build_text("Ada"), Value::from_i64(1)]]
    );
}

#[test]
fn prepare_result_types_match_projection_width() {
    let (_database, graph) = fixture::social_graph_connection();
    let stmt = graph
        .prepare(
            "MATCH (n:Person) RETURN n.name AS name, n.age AS age",
            &Parameters::new(),
        )
        .expect("prepare");
    assert_eq!(stmt.result_types().len(), stmt.num_columns());
    assert_eq!(stmt.result_types().len(), 2);
    assert_eq!(stmt.result_types()[0], ValueType::Text);
    assert_eq!(stmt.result_types()[1], ValueType::Integer);
}

/// Dialect-pinned open still installs the temporal extension for InternalHelper
/// mutation SQL (SQLite symbol table), while Root reads resolve via GraphDialect.
#[test]
fn dialect_pinned_open_installs_temporal_extension_and_runs_duration() {
    let before = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-duration",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);
    let after_open = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    assert!(
        after_open - before >= 1,
        "dialect-pinned GraphConnection::open must still install_temporal_extension for InternalHelper symbols (delta={})",
        after_open - before
    );

    let rows = session
        .query("RETURN duration('P1DT25H') AS d", &Parameters::new())
        .expect("duration must resolve (dialect Root path; extension also present)");
    assert_eq!(rows, vec![vec![Value::build_text("P1DT25H")]]);
}

/// Mutation helpers use `prepare_internal` → SQLite function resolve only.
/// Dialect-owned GraphDialect resolve does not cover that path, so
/// `install_temporal_extension` must run even under DialectPinned — otherwise
/// lowered `cypher_equals` (list IN membership) fails at prepare.
#[test]
fn dialect_pinned_mutation_in_predicate_needs_temporal_extension() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-mutation-cypher-equals",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);

    session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36})",
            &Parameters::new(),
        )
        .expect("seed create");

    // IN lowers through cypher_equals inside mutation helper SQL.
    let summary = session
        .execute(
            "MATCH (n:Person) WHERE n.name IN ['Ada'] SET n.age = 40",
            &Parameters::new(),
        )
        .expect("dialect-pinned mutation with cypher_equals must work when install always runs");
    assert_eq!(summary.matched_rows, 1);
    assert!(summary.operations_executed >= 1);

    let rows = session
        .query(
            "MATCH (n:Person {id: 1}) RETURN n.age AS age",
            &Parameters::new(),
        )
        .expect("match after set");
    assert_eq!(rows, vec![vec![Value::from_i64(40)]]);
}

/// Attach mode (foreign dialect) still installs the temporal extension.
#[test]
fn attach_mode_install_increments_temporal_install_count() {
    let before = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    let (_database, session) = fixture::social_graph_connection();
    assert_eq!(session.host_mode(), GraphHostMode::Attach);
    let after = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    assert!(
        after > before,
        "attach-mode install must call install_temporal_extension"
    );
}

/// Expand stays session-activated: double `install_graph_catalog` must not error.
#[test]
fn install_graph_catalog_is_idempotent() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:expand-idempotent",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    let store = Arc::new(SnapshotStore::default());
    let first = install_graph_catalog(connection.as_ref(), store.clone())
        .expect("first expand catalog install");
    let second = install_graph_catalog(connection.as_ref(), store)
        .expect("second expand catalog install must be idempotent");
    assert_eq!(first, GRAPH_EXPAND_TABLE_NAME);
    assert_eq!(second, GRAPH_EXPAND_TABLE_NAME);
}

/// Mutation helper SQL is prepared via `prepare_internal` (InternalHelper).
/// Under GraphDialect, CREATE must still commit and be visible to MATCH.
#[test]
fn simple_create_mutation_commits_under_graph_dialect() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-create",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);

    let summary = session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36})",
            &Parameters::new(),
        )
        .expect("create under GraphDialect");
    assert_eq!(summary.matched_rows, 1);
    assert!(summary.operations_executed >= 1);

    let rows = session
        .query(
            "MATCH (n:Person {id: 1}) RETURN n.name AS name",
            &Parameters::new(),
        )
        .expect("match after create");
    assert_eq!(rows, vec![vec![Value::build_text("Ada")]]);
}

/// Closed single-node CREATE must take the closed CREATE fast path under
/// GraphDialect and remain visible to MATCH (including label membership).
///
/// A hit means the fast-path branch ran (one prepare for the node INSERT).
/// Labeled creates may still use extra prepares for label-junction rows —
/// this is not a claim of one VDBE program for the whole mutation.
#[test]
fn single_create_node_uses_closed_create_fast_path() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-closed-create-fast-path",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    session
        .execute(
            "CREATE (:Person {id: 42, name: 'Grace'})",
            &Parameters::new(),
        )
        .expect("single create");
    assert!(
        take_closed_create_fast_path_hit(),
        "closed CREATE node must take the closed CREATE fast path"
    );

    let rows = session
        .query(
            "MATCH (n:Person {id: 42}) RETURN n.name AS name",
            &Parameters::new(),
        )
        .expect("match after closed-create fast path");
    assert_eq!(rows, vec![vec![Value::build_text("Grace")]]);
}

/// Multi-stage mutations must not take the closed CREATE fast path.
#[test]
fn multi_stage_mutation_still_uses_savepoint_path() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-multi-stage-mutation",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    let summary = session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada'}) WITH 1 AS x RETURN x",
            &Parameters::new(),
        )
        .expect("multi-stage mutation");
    assert_eq!(summary.rows, vec![vec![Value::from_i64(1)]]);
    assert!(
        !take_closed_create_fast_path_hit(),
        "WITH stages must stay on the multi-prepare savepoint path"
    );
}

/// EXPLAIN must lower Cypher once, then prepare pure SQL `EXPLAIN QUERY PLAN`
/// against Core — never re-parse the Cypher text as a dialect statement.
#[test]
fn explain_match_returns_core_eqp_rows() {
    let (_database, graph) = fixture::social_graph_connection();
    let rows = graph
        .query("EXPLAIN MATCH (n:Person) RETURN n.name", &Parameters::new())
        .expect("explain");
    assert!(!rows.is_empty(), "EXPLAIN must return EQP rows from core");
    let plan_text = rows
        .iter()
        .flatten()
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        plan_text.contains("SCAN")
            || plan_text.contains("SEARCH")
            || plan_text.contains("USING INDEX")
            || plan_text.contains("USING COVERING INDEX"),
        "EXPLAIN QUERY PLAN must describe a core access method, got:\n{plan_text}"
    );

    // EXPLAIN reports core plan columns, not the Cypher projection types.
    let stmt = graph
        .prepare("EXPLAIN MATCH (n:Person) RETURN n.name", &Parameters::new())
        .expect("prepare explain");
    assert!(
        stmt.result_types().is_empty(),
        "EXPLAIN must not surface Cypher result_types"
    );
}

/// A two-role (`start`/`end`) relation over `people`/`relationships`, wired
/// for both `GraphCatalogSnapshot` (binding) and `RelationalCatalogSnapshot`
/// (lowering) exactly as production catalogs are. Used to pin the SQL a
/// binary relation lowers to before and after role-based lowering lands.
struct BinaryCatalog;

impl GraphCatalogSnapshot for BinaryCatalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(2).ok()
    }

    fn label(&self, _graph: GraphId, _name: &str) -> Option<LabelId> {
        LabelId::new(1).ok()
    }

    fn relationship_type(&self, _graph: GraphId, _name: &str) -> Option<RelationshipTypeId> {
        RelationshipTypeId::new(1).ok()
    }

    fn property(
        &self,
        _graph: GraphId,
        _entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        let id = match name {
            "name" => 1,
            "age" => 2,
            _ => 3,
        };
        Some(ResolvedProperty {
            id: PropertyId::new(id).ok()?,
            value_type: if name == "age" {
                ValueType::Integer
            } else {
                ValueType::Text
            },
            nullability: Nullability::Nullable,
        })
    }

    fn relationship_source_roles(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        self.relationship_layout(source)
    }
}

impl RelationalCatalogSnapshot for BinaryCatalog {
    fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
        (source.get() == 1).then(|| NodeTableLayout {
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        })
    }

    fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        (source.get() == 2).then(|| RelationshipTableLayout {
            table: "relationships".to_owned(),
            identity_column: "id".to_owned(),
            roles: vec![
                RelationshipRoleLayout {
                    role: RoleId::new(1).unwrap(),
                    name: "start".to_owned(),
                    column: "src".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
                RelationshipRoleLayout {
                    role: RoleId::new(2).unwrap(),
                    name: "end".to_owned(),
                    column: "dst".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
            ],
        })
    }

    fn property_column(&self, _source: SourceTableId, property: PropertyId) -> Option<String> {
        match property.get() {
            1 => Some("name".to_owned()),
            2 => Some("age".to_owned()),
            _ => None,
        }
    }
}

/// Parses, binds, and lowers `query` against [`BinaryCatalog`], returning the
/// generated SQL text. Binary is a layout of the role model: the binder
/// always resolves `start`/`end` by name (see `binder.rs`), so this exercises
/// exactly the role-pair path role-based lowering must reproduce.
fn lower_to_sql(query: &str) -> String {
    let parsed = parse(query).expect("query must parse");
    let bound = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &BinaryCatalog,
        &ParameterTypes::new(),
    )
    .expect("query must bind");
    lower_relational(&bound.plan, &BinaryCatalog)
        .expect("query must lower")
        .to_string()
}

const BINARY_GOLDEN_QUERIES: [&str; 4] = [
    "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.name",
    "MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.name",
    "MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b.name",
    "MATCH (a:Person)-[r:KNOWS]->(b:Person) WHERE b.age > 30 RETURN b",
];

/// Prints `lower_to_sql` output for each golden query. Run with `--nocapture`
/// before touching `lower_fixed_expand` and paste the output into
/// `expected_binary_sql` below. Not a red/green driver: this printer always
/// passes, by construction.
#[test]
#[ignore]
fn print_binary_sql_goldens() {
    for query in BINARY_GOLDEN_QUERIES {
        println!("---\n{query}\n{}\n", lower_to_sql(query));
    }
}

/// SQL recorded by running `print_binary_sql_goldens` against the
/// direction-based `lower_fixed_expand`, before role-based lowering
/// replaced it. Frozen: if role-based lowering disagrees, the lowering is
/// wrong, not this map.
fn expected_binary_sql(query: &str) -> &'static str {
    match query {
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.name" => {
            r#"SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."src" = q.b1 JOIN "people" AS n ON n."id" = r."dst") AS q WHERE TRUE) AS q) AS q"#
        }
        "MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.name" => {
            r#"SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."dst" = q.b1 JOIN "people" AS n ON n."id" = r."src") AS q WHERE TRUE) AS q) AS q"#
        }
        "MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b.name" => {
            r#"SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON (r."src" = q.b1 OR r."dst" = q.b1) JOIN "people" AS n ON n."id" = CASE WHEN r."src" = q.b1 THEN r."dst" ELSE r."src" END) AS q WHERE TRUE) AS q) AS q"#
        }
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) WHERE b.age > 30 RETURN b" => {
            r#"SELECT q.b4 AS "b" FROM (SELECT q.b3 AS b4 FROM (SELECT q.* FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."age" AS b3_p2 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."src" = q.b1 JOIN "people" AS n ON n."id" = r."dst") AS q WHERE TRUE) AS q WHERE (q.b3_p2) > (30)) AS q) AS q"#
        }
        other => panic!("no recorded golden SQL for {other}"),
    }
}

#[test]
fn role_lowering_emits_byte_identical_sql_for_a_two_role_relation() {
    // Binary is a layout of the role model. If role lowering produces even a
    // different alias or predicate order, every donor query's plan shifts and
    // the corpus number stops meaning what it meant.
    for query in BINARY_GOLDEN_QUERIES {
        assert_eq!(
            lower_to_sql(query),
            expected_binary_sql(query),
            "role lowering changed the SQL for {query}"
        );
    }
}

/// A three-role relation (`scribe`, `folio`, `txt`), all `One` cardinality,
/// registered exactly as any relation is: through
/// `RelationalCatalogSnapshot`, with no arity-3 special case anywhere in the
/// fixture or in the lowering it exercises.
struct TernaryCatalog;

impl RelationalCatalogSnapshot for TernaryCatalog {
    fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
        match source.get() {
            1 => Some(NodeTableLayout {
                table: "scribes".to_owned(),
                identity_column: "id".to_owned(),
            }),
            2 => Some(NodeTableLayout {
                table: "folios".to_owned(),
                identity_column: "id".to_owned(),
            }),
            _ => None,
        }
    }

    fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        (source.get() == 10).then(|| RelationshipTableLayout {
            table: "transcriptions".to_owned(),
            identity_column: "id".to_owned(),
            roles: vec![
                RelationshipRoleLayout {
                    role: RoleId::new(1).unwrap(),
                    name: "scribe".to_owned(),
                    column: "scribe".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
                RelationshipRoleLayout {
                    role: RoleId::new(2).unwrap(),
                    name: "folio".to_owned(),
                    column: "folio".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
                RelationshipRoleLayout {
                    role: RoleId::new(3).unwrap(),
                    name: "txt".to_owned(),
                    column: "txt".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
            ],
        })
    }

    fn property_column(&self, _source: SourceTableId, _property: PropertyId) -> Option<String> {
        None
    }
}

/// Lowers a hand-built `ir::FixedExpand` over [`TernaryCatalog`]'s
/// `scribe -> folio` role pair. `_query` documents the surface syntax this
/// stands in for (`[x:Transcription](scribe: s, folio: f)`, not yet parsable
/// -- that lands in Task 12); the plan below is built directly so this test
/// does not depend on that syntax landing first.
fn lower_ternary_to_sql(_query: &str) -> String {
    let scribe_source = SourceTableId::new(1).unwrap();
    let folio_source = SourceTableId::new(2).unwrap();
    let relationship_source = SourceTableId::new(10).unwrap();
    let scribe_role = RoleId::new(1).unwrap();
    let folio_role = RoleId::new(2).unwrap();

    let from_binding = BindingId::new(1).unwrap();
    let relationship_binding = BindingId::new(2).unwrap();
    let to_binding = BindingId::new(3).unwrap();

    let from_var = Binding::new(from_binding, "s", ValueType::Node, Nullability::NonNull).unwrap();
    let relationship_var = Binding::new(
        relationship_binding,
        "x",
        ValueType::Relationship,
        Nullability::NonNull,
    )
    .unwrap();
    let to_var = Binding::new(to_binding, "f", ValueType::Node, Nullability::NonNull).unwrap();

    let scan = Plan::new(
        PlanKind::NodeScan(NodeScan {
            graph: GraphId::new(1).unwrap(),
            source: scribe_source,
            binding: from_binding,
            labels: vec![],
        }),
        Scope::new(vec![from_var.clone()]).unwrap(),
        ResultShape::default(),
    )
    .unwrap();

    let expand = Plan::new(
        PlanKind::FixedExpand(FixedExpand {
            input: Box::new(scan),
            from_node_source: scribe_source,
            relationship_source,
            target_node_source: folio_source,
            from: from_binding,
            relationship: relationship_var.clone(),
            to: to_var.clone(),
            direction: Direction::Outgoing,
            from_role: scribe_role,
            to_role: folio_role,
            symmetric: false,
            relationship_types: vec![],
            bound_target: None,
        }),
        Scope::new(vec![from_var, relationship_var, to_var]).unwrap(),
        ResultShape::default(),
    )
    .unwrap();

    lower_relational(&expand, &TernaryCatalog)
        .expect("ternary hop must lower")
        .to_string()
}

#[test]
fn a_ternary_hop_lowers_through_the_named_role_pair() {
    // Direction-based lowering has only start and end to name, so a
    // scribe -> folio hop is inexpressible: it would silently lower as
    // start -> end and return the text instead of the folio.
    let sql = lower_ternary_to_sql("MATCH [x:Transcription](scribe: s, folio: f) RETURN f.id");
    assert!(
        sql.contains("scribe"),
        "the from role must name its own column: {sql}"
    );
    assert!(
        sql.contains("folio"),
        "the to role must name its own column: {sql}"
    );
    assert!(
        !sql.contains("txt"),
        "the unnamed text role must not be joined: {sql}"
    );
}
