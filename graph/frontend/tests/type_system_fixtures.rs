// Fixture setup mirrors graph/frontend/tests/fixed_pattern_fixtures.rs —
// see that file for the established connection/registration helpers this
// file reuses.

use std::sync::Arc;

use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, GraphCompilationCatalog, GraphRegistration, GraphSession, MutationParameters,
    NodeSourceRegistration, ParameterTypes, SchemaCatalog, SnapshotStore,
};
use turso_graph_ir::{self as ir, GraphId, PlanKind};
use turso_graph_runtime::{BuildLimits, NeverCancelled};

fn connect(enable_custom_types: bool) -> Arc<turso_core::Connection> {
    let io = Arc::new(MemoryIO::new());
    Database::open_file_with_flags(
        io,
        ":memory:type-system-fixtures",
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(enable_custom_types),
        None,
        Arc::new(SqliteDialect),
    )
    .expect("open database")
    .connect()
    .expect("connect")
}

/// Registers `table` as the sole node source of a single-node-source graph
/// and returns the `SchemaCatalog` backing it, ready for `bind`.
fn node_source_catalog(connection: &Arc<turso_core::Connection>, table: &str) -> SchemaCatalog {
    let graph = turso_graph_frontend::register_graph(
        connection,
        &GraphRegistration {
            name: "typesys".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Node".to_owned(),
                table: table.to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![],
        },
    )
    .expect("register graph");
    SchemaCatalog::new(connection.clone(), graph)
}

/// Binds `query` against `catalog` and returns the single `RETURN`
/// projection's resolved `value_type`.
fn returned_value_type(catalog: &SchemaCatalog, query: &str) -> ir::ValueType {
    let parsed = parse(query).unwrap_or_else(|error| panic!("{query} did not parse: {error}"));
    let parameters = ParameterTypes::new();
    let bound = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        catalog,
        &parameters,
    )
    .unwrap_or_else(|error| panic!("{query} did not bind: {error}"));
    match bound.plan.kind() {
        PlanKind::Project(project) => {
            assert_eq!(
                project.projections.len(),
                1,
                "expected a single RETURN projection"
            );
            project.projections[0].expression.value_type.clone()
        }
        other => panic!("expected a Project plan for a RETURN query, got {other:?}"),
    }
}

#[test]
fn custom_scalar_column_resolves_to_custom_value_type() {
    let connection = connect(true);
    connection
        .execute(
            "CREATE TYPE cents BASE integer; \
             CREATE TABLE prices(id INTEGER PRIMARY KEY, amount cents) STRICT;",
        )
        .expect("create custom-typed source");
    let catalog = node_source_catalog(&connection, "prices");

    let value_type = returned_value_type(&catalog, "MATCH (p) RETURN p.amount");

    assert_eq!(
        value_type,
        ir::ValueType::Custom {
            name: "cents".to_owned(),
            base: Box::new(ir::ValueType::Integer),
        }
    );
}

#[test]
fn integer_array_column_resolves_to_nested_list_value_type() {
    let connection = connect(true);
    connection
        .execute("CREATE TABLE tags(id INTEGER PRIMARY KEY, labels INTEGER[]) STRICT;")
        .expect("create array source");
    let catalog = node_source_catalog(&connection, "tags");

    let value_type = returned_value_type(&catalog, "MATCH (t) RETURN t.labels");

    assert_eq!(
        value_type,
        ir::ValueType::List(Box::new(ir::ValueType::Integer))
    );
}

#[test]
fn blob_column_resolves_to_bytes_value_type() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create blob source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(&catalog, "MATCH (e) RETURN e.vector");

    assert_eq!(value_type, ir::ValueType::Bytes);
}

/// Installs a `GraphSession` for a single-node-source graph over `table`,
/// reusing the writer-session setup `session.rs`'s own tests establish
/// (register the graph, publish an initial shared traversal snapshot, then
/// install the session), but backed by `SchemaCatalog` so STRICT struct/union
/// columns resolve to their real declared types instead of a stub catalog.
fn graph_session_for_node_source(
    connection: &Arc<turso_core::Connection>,
    label: &str,
    table: &str,
) -> GraphSession {
    let registered = turso_graph_frontend::register_graph(
        connection,
        &GraphRegistration {
            name: "typesys-write".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: label.to_owned(),
                table: table.to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    shared_snapshots
        .refresh(
            connection,
            &registered.name,
            BuildLimits::default(),
            &NeverCancelled,
        )
        .expect("build initial traversal snapshot");
    GraphSession::install(
        connection.clone(),
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session")
}

#[test]
fn create_with_struct_map_literal_lowers_and_executes() {
    let connection = connect(true);
    connection
        .execute(
            // Field types are DOMAINs, not the bare `INTEGER` keyword the
            // struct/union write fixture originally described: SchemaCatalog's
            // `resolve_named_type` (graph/frontend/src/schema_catalog.rs)
            // resolves a STRUCT/UNION field's `type_name` only through
            // `Schema::resolve_type`, which looks the name up in the custom
            // type registry and returns `None` for a bare primitive keyword —
            // so a field declared `x INTEGER` resolves to `ValueType::Any`
            // instead of `Integer`, and binding `{x: 1, ...}` against it then
            // fails bind-time type-equality with `Unsupported { feature:
            // "struct field type mismatch" }` (confirmed by direct
            // observation: `MATCH (s) RETURN s.origin` resolved to
            // `Struct([("x", Any), ("y", Any)])` on a bare-`INTEGER` field).
            // A DOMAIN field type takes the `is_domain` branch of
            // `resolve_named_type`, which resolves straight to
            // `primitive_value_type(&resolved.primitive)` and correctly
            // yields `Integer` — sidestepping the bug without touching
            // non-test code, while still exercising this task's actual
            // target (the `Map` lowering arm) end to end.
            "CREATE DOMAIN posint AS integer; \
             CREATE TYPE point AS STRUCT(x posint, y posint); \
             CREATE TABLE shapes(id INTEGER PRIMARY KEY, origin point) STRICT;",
        )
        .expect("create struct-typed source");
    let session = graph_session_for_node_source(&connection, "Shape", "shapes");

    session
        .mutate(
            "CREATE (:Shape {origin: {x: 1, y: 2}})",
            &MutationParameters::new(),
        )
        .expect("create struct-valued node");

    let stored = connection
        .prepare("SELECT origin FROM shapes")
        .expect("prepare select origin")
        .run_collect_rows()
        .expect("select origin");
    let expected = connection
        .prepare("SELECT struct_pack(1, 2)")
        .expect("prepare struct_pack")
        .run_collect_rows()
        .expect("select struct_pack");
    assert_eq!(stored, expected);
}

#[test]
fn create_with_union_map_literal_lowers_and_executes() {
    let connection = connect(true);
    connection
        .execute(
            // See the comment in `create_with_struct_map_literal_lowers_and_executes`:
            // bare-primitive UNION variant types hit the same
            // `resolve_named_type` fallback-to-`Any` bug, so variants use a
            // TEXT-based DOMAIN instead.
            "CREATE DOMAIN reach_text AS text; \
             CREATE TYPE contact AS UNION(email reach_text, phone reach_text); \
             CREATE TABLE people(id INTEGER PRIMARY KEY, reach contact) STRICT;",
        )
        .expect("create union-typed source");
    let session = graph_session_for_node_source(&connection, "Person", "people");

    session
        .mutate(
            "CREATE (:Person {reach: {email: 'a@example.com'}})",
            &MutationParameters::new(),
        )
        .expect("create union-valued node");

    // Unlike `struct_pack`, `union_value()` cannot be evaluated in a bare
    // `SELECT`: core resolves its tag against `program.target_union_type`,
    // which is only set while translating an INSERT/UPDATE that targets a
    // union-typed column (core/translate/expr/translator.rs) — a bare SELECT
    // rejects it with "union_value() can only be used in INSERT/UPDATE
    // targeting a union-typed column". So the expected value is produced by
    // a direct SQL INSERT into the same union-typed column instead.
    connection
        .execute("INSERT INTO people(id, reach) VALUES (2, union_value('email', 'a@example.com'))")
        .expect("insert expected union value");

    let rows = connection
        .prepare("SELECT reach FROM people ORDER BY id")
        .expect("prepare select reach")
        .run_collect_rows()
        .expect("select reach");
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0], rows[1],
        "CREATE's map-literal-lowered union value must match a direct SQL union_value() insert"
    );
}

#[test]
fn matches_two_level_nested_struct_field_read_executes() {
    let connection = connect(true);
    connection
        .execute(
            "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
             CREATE TYPE region AS STRUCT(origin point, label TEXT); \
             CREATE TABLE zones(id INTEGER PRIMARY KEY, bounds region) STRICT;",
        )
        .expect("create nested struct-typed source");
    let session = graph_session_for_node_source(&connection, "Zone", "zones");

    // `bind_map_property` only recurses for the outer struct/union property
    // assignment; a nested map-literal *value* for a struct-typed field
    // (e.g. `{origin: {x: 3, y: 4}, label: 'north'}`'s `origin` entry) is
    // bound through the general `bind_expression` path instead, which
    // rejects any bare `cypher::Expression::Map` ("map literal outside a
    // property assignment", `binder.rs`). That is a real, pre-existing gap
    // in CREATE's map-literal support for genuinely 2-level-nested struct
    // literals, orthogonal to this task's property-*read* lowering fix, so
    // (mirroring `create_with_union_map_literal_lowers_and_executes`'s same
    // workaround for an unrelated CREATE-side limitation) the nested value
    // is inserted directly via SQL `struct_pack` here; only the
    // MATCH...RETURN 2-level nested field *read* below — this fix's actual
    // target — goes through the graph session.
    connection
        .execute(
            "INSERT INTO zones(id, bounds) VALUES (1, struct_pack(struct_pack(3, 4), 'north'))",
        )
        .expect("insert nested struct-valued row");

    let rows = session
        .query(
            "MATCH (z:Zone) RETURN z.bounds.origin.x",
            &MutationParameters::new(),
        )
        .expect("2-level nested field read must execute, not fail with a SQL syntax error");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec![turso_core::Value::from_i64(3)]);
}

#[test]
fn nested_struct_field_read_lowers_and_executes() {
    let connection = connect(true);
    connection
        .execute(
            "CREATE TYPE address AS STRUCT(city TEXT, zip INTEGER); \
             CREATE TYPE person AS STRUCT(name TEXT, home address); \
             CREATE TABLE people(id INTEGER PRIMARY KEY, info person) STRICT;",
        )
        .expect("create nested struct-typed source");
    let session = graph_session_for_node_source(&connection, "Person", "people");

    connection
        .execute("INSERT INTO people VALUES (1, struct_pack('Ada', struct_pack('London', 90210)))")
        .expect("insert nested struct-valued row");

    // Exactly at the two-level cap: `info` is the root property, `home` and
    // `city` are the two nested fields beyond it.
    let rows = session
        .query(
            "MATCH (p:Person) RETURN p.info.home.city",
            &MutationParameters::new(),
        )
        .expect("2-level nested field read must execute, not fail with a SQL syntax error");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec![turso_core::Value::build_text("London")]);

    // One level past the cap: `home`, `city`, `extra` are three nested
    // fields beyond the root `info` property, which core's SQL grammar
    // cannot express as a dot-chain (see commit 648e3b7b2), so bind() must
    // reject it rather than attempt to lower unparsable SQL. Asserted via
    // a direct `bind()` call against a fresh `SchemaCatalog` — the same
    // pattern `binder.rs`'s own `#[cfg(test)]` module uses for asserting a
    // specific bind-time error (e.g.
    // `rejects_field_access_deeper_than_two_levels`) — rather than through
    // `GraphSession::query`, whose `FrontendCompiler` impl (`compiler.rs`)
    // collapses every bind error into an opaque `LimboError::ParseError`
    // string. Registering a second graph on this connection allocates a
    // new `GraphId` (the first, from `graph_session_for_node_source`
    // above, already took id 1), so the id actually returned by
    // `register_graph` is used here instead of assuming id 1, matching
    // `SchemaCatalog::node_source`'s `graph == self.graph.id` gate.
    let registered = turso_graph_frontend::register_graph(
        &connection,
        &GraphRegistration {
            name: "typesys-nested-reject".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![],
        },
    )
    .expect("register second graph for the rejection catalog");
    let graph_id = registered.id;
    let catalog = SchemaCatalog::new(connection, registered);
    let parsed = parse("MATCH (p) RETURN p.info.home.city.extra").expect("query parses");
    let error = bind(&parsed, graph_id, &catalog, &ParameterTypes::new())
        .expect_err("3-level nested field access must be rejected at bind time");
    assert!(
        matches!(error, turso_graph_frontend::BindError::Unsupported { .. }),
        "expected BindError::Unsupported, got {error:?}"
    );
}

#[test]
fn vector32_call_binds_to_vector_value_type() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(&catalog, "MATCH () RETURN vector32('[1.0, 2.0, 3.0]')");

    assert_eq!(
        value_type,
        ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(3))
    );
}

#[test]
fn vector_distance_cos_call_binds_to_real() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(
        &catalog,
        "MATCH () RETURN vector_distance_cos(vector32('[1.0]'), vector32('[2.0]'))",
    );

    assert_eq!(value_type, ir::ValueType::Real);
}

#[test]
fn fts_match_call_binds_to_boolean() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(
        &catalog,
        "MATCH () RETURN fts_match('needle', 'haystack text')",
    );

    assert_eq!(value_type, ir::ValueType::Boolean);
}

#[test]
fn vector_extract_call_binds_to_text() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(
        &catalog,
        "MATCH () RETURN vector_extract(vector32('[1.0, 2.0, 3.0]'))",
    );

    assert_eq!(value_type, ir::ValueType::Text);
}

#[test]
fn vector32_call_with_wrong_argument_count_is_a_bind_error() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let parsed = parse("MATCH () RETURN vector32(1.0, 2.0, 3.0)").expect("query parses");
    let error = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &catalog,
        &ParameterTypes::new(),
    )
    .expect_err("vector32 is fixed-arity-1, a 3-argument call must be rejected at bind time");
    assert!(
        matches!(error, turso_graph_frontend::BindError::Unsupported { .. }),
        "expected BindError::Unsupported, got {error:?}"
    );
}

#[test]
fn vector_distance_cos_call_with_mismatched_argument_type_is_a_bind_error() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let parsed = parse("MATCH () RETURN vector_distance_cos('not a vector', 'also not a vector')")
        .expect("query parses");
    let error = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &catalog,
        &ParameterTypes::new(),
    )
    .expect_err("vector_distance_cos requires Vector-shaped arguments, Text must be rejected");
    assert!(
        matches!(error, turso_graph_frontend::BindError::Unsupported { .. }),
        "expected BindError::Unsupported, got {error:?}"
    );
}

#[test]
fn fts_match_call_with_too_few_arguments_is_a_bind_error() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let parsed = parse("MATCH () RETURN fts_match('only one argument')").expect("query parses");
    let error = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &catalog,
        &ParameterTypes::new(),
    )
    .expect_err("fts_match requires at least 2 arguments (text, query)");
    assert!(
        matches!(error, turso_graph_frontend::BindError::Unsupported { .. }),
        "expected BindError::Unsupported, got {error:?}"
    );
}

#[test]
fn fts_match_call_with_extra_arguments_still_binds() {
    let connection = connect(false);
    connection
        .execute("CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);")
        .expect("create node source");
    let catalog = node_source_catalog(&connection, "embeddings");

    let value_type = returned_value_type(
        &catalog,
        "MATCH () RETURN fts_match('col1 text', 'col2 text', 'query')",
    );

    assert_eq!(value_type, ir::ValueType::Boolean);
}
