use std::{collections::HashMap, num::NonZero, sync::Arc};

use thiserror::Error;
use turso_core::{Connection, Statement, Value};
use turso_graph_ir::GraphId;
use turso_graph_runtime::{BuildLimits, Cancellation, NeverCancelled};

use crate::{
    execute_cypher_mutation, graph_frontend_id, install_graph_catalog, GraphCompilationCatalog,
    GraphCompiler, MutationError, MutationParameters, MutationSummary, ParameterTypes,
    RegisteredGraph, SessionSnapshotStore, SnapshotError, SnapshotStore,
};

#[derive(Debug, Error)]
pub enum GraphSessionError {
    #[error(transparent)]
    Parse(#[from] turso_graph_cypher::ParseError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Mutation(#[from] MutationError),
    #[error(transparent)]
    Database(#[from] turso_core::LimboError),
    #[error("query parameter `${0}` was not declared for this graph session")]
    UndeclaredParameter(String),
    #[error("query parameter `${0}` has no bound value")]
    MissingParameter(String),
}

/// Connection-local graph service boundary.
///
/// Read compilation stays on `FrontendCompiler`. Variable traversal rebuilds
/// a private snapshot immediately before execution, so it observes the rows
/// visible to this connection without publishing uncommitted state globally.
pub struct GraphSession {
    connection: Arc<Connection>,
    graph: GraphId,
    graph_name: String,
    catalog: Arc<dyn GraphCompilationCatalog>,
    snapshots: Arc<SessionSnapshotStore>,
    limits: BuildLimits,
}

impl GraphSession {
    pub fn install(
        connection: Arc<Connection>,
        graph: &RegisteredGraph,
        catalog: Arc<dyn GraphCompilationCatalog>,
        parameters: ParameterTypes,
        shared_snapshots: Arc<SnapshotStore>,
        limits: BuildLimits,
    ) -> Result<Self, GraphSessionError> {
        let snapshots = Arc::new(SessionSnapshotStore::new(shared_snapshots.clone()));
        shared_snapshots.register_session(&connection, &snapshots)?;
        install_graph_catalog(connection.as_ref(), shared_snapshots)?;
        connection.register_frontend_compiler(
            graph_frontend_id(),
            Arc::new(GraphCompiler::new(graph.id, catalog.clone(), parameters)),
        )?;
        Ok(Self {
            connection,
            graph: graph.id,
            graph_name: graph.name.clone(),
            catalog,
            snapshots,
            limits,
        })
    }

    pub fn graph_id(&self) -> GraphId {
        self.graph
    }

    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }

    pub fn query(
        &self,
        source: &str,
        parameters: &MutationParameters,
    ) -> Result<Vec<Vec<Value>>, GraphSessionError> {
        self.query_cancellable(source, parameters, &NeverCancelled)
    }

    pub fn query_cancellable(
        &self,
        source: &str,
        parameters: &MutationParameters,
        cancellation: &dyn Cancellation,
    ) -> Result<Vec<Vec<Value>>, GraphSessionError> {
        let mut statement = self.prepare_query_cancellable(source, parameters, cancellation)?;
        Ok(statement.run_collect_rows()?)
    }

    pub fn prepare_query(
        &self,
        source: &str,
        parameters: &MutationParameters,
    ) -> Result<Statement, GraphSessionError> {
        self.prepare_query_cancellable(source, parameters, &NeverCancelled)
    }

    pub fn prepare_query_cancellable(
        &self,
        source: &str,
        parameters: &MutationParameters,
        cancellation: &dyn Cancellation,
    ) -> Result<Statement, GraphSessionError> {
        let syntax = turso_graph_cypher::parse(source)?;
        if requires_traversal_snapshot(&syntax) {
            self.snapshots.refresh_visible_if_stale(
                &self.connection,
                &self.graph_name,
                self.limits,
                cancellation,
            )?;
        }
        let mut statement = self
            .connection
            .prepare_frontend(&graph_frontend_id(), source)?;
        bind_query_parameters(&mut statement, parameters)?;
        Ok(statement)
    }

    pub fn mutate(
        &self,
        source: &str,
        parameters: &MutationParameters,
    ) -> Result<MutationSummary, GraphSessionError> {
        let result = execute_cypher_mutation(
            &self.connection,
            self.graph,
            self.catalog.clone(),
            source,
            parameters,
        );
        self.snapshots.clear()?;
        Ok(result?)
    }
}

fn requires_traversal_snapshot(query: &turso_graph_cypher::Query) -> bool {
    query.clauses.iter().any(|clause| match &clause.value {
        turso_graph_cypher::Clause::Match(value) => value.paths.iter().any(|path| {
            path.steps
                .iter()
                .any(|(relationship, _)| relationship.range.is_some())
        }),
        _ => false,
    })
}

fn bind_query_parameters(
    statement: &mut Statement,
    parameters: &HashMap<String, Value>,
) -> Result<(), GraphSessionError> {
    for (name, value) in parameters {
        let parameter = format!("${name}");
        let index = statement
            .parameter_index(&parameter)
            .ok_or_else(|| GraphSessionError::UndeclaredParameter(name.clone()))?;
        statement.bind_at(index, value.clone())?;
    }
    for raw_index in 1..=statement.parameters_count() {
        let index = NonZero::new(raw_index).expect("parameter indexes start at one");
        let Some(name) = statement.parameters().name(index) else {
            continue;
        };
        let name = name.strip_prefix('$').unwrap_or(name.as_str()).to_owned();
        if !parameters.contains_key(&name) {
            return Err(GraphSessionError::MissingParameter(name));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        register_graph, CatalogEntity, GraphCatalogSnapshot, GraphRegistration,
        NodeSourceRegistration, NodeTableLayout, RelationalCatalogSnapshot,
        RelationshipSourceRegistration, RelationshipTableLayout, ResolvedProperty,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect};
    use turso_graph_ir as ir;

    struct Catalog {
        node_source: ir::SourceTableId,
        relationship_source: ir::SourceTableId,
    }

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: GraphId) -> Option<ir::SourceTableId> {
            Some(self.node_source)
        }

        fn relationship_source(&self, _graph: GraphId) -> Option<ir::SourceTableId> {
            Some(self.relationship_source)
        }

        fn label(&self, _graph: GraphId, name: &str) -> Option<ir::LabelId> {
            (name == "Person").then(|| ir::LabelId::new(1).unwrap())
        }

        fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
            (name == "KNOWS").then(|| ir::RelationshipTypeId::new(1).unwrap())
        }

        fn property(
            &self,
            _graph: GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            let (id, value_type, nullability) = match (entity, name) {
                (CatalogEntity::Node, "id") => {
                    (1, ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                (CatalogEntity::Node, "name") => {
                    (2, ir::ValueType::Text, ir::Nullability::Nullable)
                }
                _ => return None,
            };
            Some(ResolvedProperty {
                id: ir::PropertyId::new(id).unwrap(),
                value_type,
                nullability,
            })
        }
    }

    impl RelationalCatalogSnapshot for Catalog {
        fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
            (source == self.node_source).then(|| NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(
            &self,
            source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            (source == self.relationship_source).then(|| RelationshipTableLayout {
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
            })
        }

        fn property_column(
            &self,
            source: ir::SourceTableId,
            property: ir::PropertyId,
        ) -> Option<String> {
            match (source, property.get()) {
                (source, 1) if source == self.node_source => Some("id".to_owned()),
                (source, 2) if source == self.node_source => Some("name".to_owned()),
                _ => None,
            }
        }
    }

    struct Fixture {
        writer: Arc<Connection>,
        reader: Arc<Connection>,
        writer_session: GraphSession,
        reader_session: GraphSession,
    }

    fn fixture(name: &str) -> Fixture {
        fixture_with_mode(name, false)
    }

    fn fixture_with_mode(name: &str, mvcc: bool) -> Fixture {
        let io = Arc::new(MemoryIO::new());
        let database = Database::open_file(io, name, Arc::new(SqliteDialect)).unwrap();
        let writer = database.connect().unwrap();
        if mvcc {
            writer.execute("PRAGMA journal_mode = 'mvcc'").unwrap();
        }
        let reader = database.connect().unwrap();
        writer
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER); \
                 INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace');",
            )
            .unwrap();
        let registered = register_graph(
            &writer,
            &GraphRegistration {
                name: "social".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "KNOWS".to_owned(),
                    table: "relationships".to_owned(),
                    identity_column: "id".to_owned(),
                    start_column: "src".to_owned(),
                    end_column: "dst".to_owned(),
                    start_node_source: "Person".to_owned(),
                    end_node_source: "Person".to_owned(),
                }],
            },
        )
        .unwrap();
        let catalog = Arc::new(Catalog {
            node_source: registered.node_sources[0].id,
            relationship_source: registered.relationship_sources[0].id,
        });
        let shared = Arc::new(SnapshotStore::default());
        shared
            .refresh(&writer, "social", BuildLimits::default(), &NeverCancelled)
            .unwrap();
        let writer_session = GraphSession::install(
            writer.clone(),
            &registered,
            catalog.clone(),
            ParameterTypes::new(),
            shared.clone(),
            BuildLimits::default(),
        )
        .unwrap();
        let reader_session = GraphSession::install(
            reader.clone(),
            &registered,
            catalog,
            ParameterTypes::new(),
            shared,
            BuildLimits::default(),
        )
        .unwrap();
        Fixture {
            writer,
            reader,
            writer_session,
            reader_session,
        }
    }

    fn outgoing(session: &GraphSession) -> Vec<Vec<Value>> {
        session
            .query(
                "MATCH (a:Person {id: 1})-[:KNOWS*1..1]->(b) RETURN b.name AS name ORDER BY b.name",
                &MutationParameters::new(),
            )
            .unwrap()
    }

    #[test]
    fn explicit_transaction_reads_its_writes_without_global_publication() {
        let fixture = fixture(":memory:graph-session-explicit");
        fixture.writer.execute("BEGIN").unwrap();
        fixture
            .writer_session
            .mutate(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &MutationParameters::new(),
            )
            .unwrap();
        assert_eq!(
            outgoing(&fixture.writer_session),
            vec![vec![Value::build_text("C")]]
        );
        assert!(outgoing(&fixture.reader_session).is_empty());

        fixture.writer.execute("ROLLBACK").unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
        assert!(outgoing(&fixture.reader_session).is_empty());
    }

    #[test]
    fn committed_and_autocommit_writes_become_visible_on_the_next_read() {
        let fixture = fixture(":memory:graph-session-commit");
        fixture.writer.execute("BEGIN").unwrap();
        fixture
            .writer_session
            .mutate(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &MutationParameters::new(),
            )
            .unwrap();
        assert!(outgoing(&fixture.reader_session).is_empty());
        fixture.writer.execute("COMMIT").unwrap();
        assert_eq!(
            outgoing(&fixture.reader_session),
            vec![vec![Value::build_text("C")]]
        );

        fixture
            .writer_session
            .mutate(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 4, name: 'D'})",
                &MutationParameters::new(),
            )
            .unwrap();
        assert_eq!(
            outgoing(&fixture.reader_session),
            vec![vec![Value::build_text("C")], vec![Value::build_text("D")]]
        );
    }

    #[test]
    fn savepoint_rollback_discards_the_next_transaction_local_snapshot() {
        let fixture = fixture(":memory:graph-session-savepoint");
        fixture
            .writer
            .execute("BEGIN; SAVEPOINT user_change")
            .unwrap();
        fixture
            .writer_session
            .mutate(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &MutationParameters::new(),
            )
            .unwrap();
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM people WHERE id = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "same-connection SQL must see the created node"
        );
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM relationships WHERE src = 1 AND dst = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "same-connection SQL must see the created relationship"
        );
        assert_eq!(outgoing(&fixture.writer_session).len(), 1);
        fixture
            .writer
            .execute("ROLLBACK TO user_change; RELEASE user_change")
            .unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
        fixture.writer.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn failed_mutation_does_not_leak_partial_rows_into_traversal() {
        let fixture = fixture(":memory:graph-session-failure");
        let result = fixture.writer_session.mutate(
            "CREATE (:Person {id: 3, name: 'C'}), (:Person {id: 1, name: 'duplicate'})",
            &MutationParameters::new(),
        );
        assert!(result.is_err());
        assert!(outgoing(&fixture.writer_session).is_empty());
        assert_eq!(
            fixture
                .reader
                .prepare("SELECT count(*) FROM people")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(2)]]
        );
    }

    #[test]
    fn cancelled_local_rebuild_is_not_installed() {
        struct Cancelled;

        impl Cancellation for Cancelled {
            fn is_cancelled(&self) -> bool {
                true
            }
        }

        let fixture = fixture(":memory:graph-session-cancelled");
        let error = fixture
            .writer_session
            .query_cancellable(
                "MATCH (a:Person {id: 1})-[:KNOWS*1..1]->(b) RETURN b.name",
                &MutationParameters::new(),
                &Cancelled,
            )
            .expect_err("cancelled rebuild must fail");
        assert!(matches!(
            error,
            GraphSessionError::Snapshot(SnapshotError::Runtime(
                turso_graph_runtime::RuntimeError::Cancelled
            ))
        ));
        assert!(outgoing(&fixture.writer_session).is_empty());
    }

    #[test]
    fn traversal_snapshot_rebuilds_once_per_visible_generation() {
        let fixture = fixture(":memory:graph-session-refresh-frequency");
        let (initial, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(!rebuilt, "the current shared snapshot should be reused");

        fixture
            .writer
            .execute("INSERT INTO relationships VALUES (10, 1, 2)")
            .unwrap();
        let (refreshed, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(rebuilt);
        assert!(!Arc::ptr_eq(&initial, &refreshed));

        let (reused, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(!rebuilt);
        assert!(Arc::ptr_eq(&refreshed, &reused));
    }

    #[test]
    fn mvcc_transaction_reads_its_writes_without_cross_connection_leakage() {
        let fixture = fixture_with_mode(":memory:graph-session-mvcc", true);
        fixture.writer.execute("BEGIN CONCURRENT").unwrap();
        fixture
            .writer_session
            .mutate(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &MutationParameters::new(),
            )
            .unwrap();
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM people WHERE id = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "MVCC SQL must see the created node"
        );
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM relationships WHERE src = 1 AND dst = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "MVCC SQL must see the created relationship"
        );
        let visible = crate::build_visible_traversal_snapshot(
            &fixture.writer,
            "social",
            BuildLimits::default(),
            &NeverCancelled,
        )
        .unwrap();
        assert_eq!(visible.graph().node_count(), 3);
        assert_eq!(visible.graph().edge_count(), 1);
        assert_eq!(outgoing(&fixture.writer_session).len(), 1);
        assert!(outgoing(&fixture.reader_session).is_empty());
        fixture.writer.execute("ROLLBACK").unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
    }
}
