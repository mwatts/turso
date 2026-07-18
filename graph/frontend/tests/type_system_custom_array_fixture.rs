// Regression coverage for the `Custom` arm of `SchemaCatalog::column_value_type`
// (graph/frontend/src/schema_catalog.rs), which had the same physical-affinity-
// vs-declared-type bug as the `Builtin | Domain` arm fixed alongside this test:
// array columns are always packed as Blob-affinity record-format values by
// core's `BTreeTable::resolve_custom_type_affinities`, so reading
// `column.affinity()` for a custom-typed array column lost the real base
// type. Mirrors the connection/registration/assertion pattern established by
// graph/frontend/tests/type_system_fixtures.rs (see that file for the
// established helpers this file reuses the shape of).

use std::sync::Arc;

use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, GraphRegistration, NodeSourceRegistration, ParameterTypes, SchemaCatalog,
};
use turso_graph_ir::{self as ir, GraphId, PlanKind};

fn connect(enable_custom_types: bool) -> Arc<turso_core::Connection> {
    let io = Arc::new(MemoryIO::new());
    Database::open_file_with_flags(
        io,
        ":memory:type-system-custom-array-fixtures",
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
fn custom_scalar_array_column_resolves_to_nested_custom_value_type() {
    let connection = connect(true);
    connection
        .execute(
            "CREATE TYPE cents BASE integer; \
             CREATE TABLE baskets(id INTEGER PRIMARY KEY, prices cents[]) STRICT;",
        )
        .expect("create custom-typed array source");
    let catalog = node_source_catalog(&connection, "baskets");

    let value_type = returned_value_type(&catalog, "MATCH (b) RETURN b.prices");

    assert_eq!(
        value_type,
        ir::ValueType::List(Box::new(ir::ValueType::Custom {
            name: "cents".to_owned(),
            base: Box::new(ir::ValueType::Integer),
        }))
    );
}
