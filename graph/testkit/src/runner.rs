use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

use thiserror::Error;
use turso_core::{Connection, DatabaseOpts, MemoryIO, Numeric, OpenFlags, Value};
use turso_graph_frontend::{
    register_graph, CatalogEntity, GraphCompilationCatalog, GraphConnection, GraphRegistration,
    NodeSourceRegistration, ParameterTypes, Parameters, RelationshipSourceRegistration,
    SnapshotStore,
};
use turso_graph_ir as ir;

use crate::{
    history::{recorded_at, result_digest_with, ResultOrdering},
    manifest::Scenario,
    model::{Outcome, ResultRecord, RunEnvironment, HISTORY_SCHEMA_VERSION},
};

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("failed to initialize graph fixture: {0}")]
    Fixture(String),
    #[error("scenario parameter `{name}` has unsupported TOML value {value}")]
    Parameter { name: String, value: toml::Value },
}

pub struct ScenarioRunner {
    environment: RunEnvironment,
    run_id: String,
    suite: String,
}

impl ScenarioRunner {
    pub fn new(
        environment: RunEnvironment,
        run_id: impl Into<String>,
        suite: impl Into<String>,
    ) -> Self {
        Self {
            environment,
            run_id: run_id.into(),
            suite: suite.into(),
        }
    }

    pub fn run(&self, scenario: &Scenario) -> Result<ResultRecord, RunnerError> {
        let parameters = parameters(&scenario.parameters)?;
        let fixture = fixture(&scenario.fixture, scenario, parameter_types(&parameters))?;
        let started = Instant::now();
        let result = execute(&fixture.session, scenario, &parameters);
        let duration_ns = started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX);
        let (outcome, rows, message) = classify(scenario, result);
        let row_count = rows.as_ref().map(|rows| rows.len() as u64);
        let result_digest = rows
            .as_ref()
            .map(|rows| result_digest_with(rows, scenario_ordering(scenario)));
        Ok(ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
            semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
            run_id: self.run_id.clone(),
            recorded_at: recorded_at(),
            environment: self.environment.clone(),
            suite: self.suite.clone(),
            test_id: scenario.id.clone(),
            kind: scenario.kind,
            area: scenario.area.clone(),
            fixture: scenario.fixture.clone(),
            expectation: scenario.expectation,
            outcome,
            duration_ns,
            source: scenario.source.clone(),
            operation: None,
            graph_shape: None,
            scale: None,
            iterations: None,
            throughput_per_second: None,
            row_count,
            node_count: None,
            relationship_count: None,
            result_digest,
            message,
            dimensions: BTreeMap::new(),
        })
    }
}

pub struct GraphFixture {
    pub connection: Arc<Connection>,
    pub session: GraphConnection,
    pub node_source: turso_graph_ir::SourceTableId,
    pub relationship_source: turso_graph_ir::SourceTableId,
    pub labels_table: String,
    pub types_table: String,
}

pub fn empty_fixture(name: &str) -> Result<GraphFixture, RunnerError> {
    build_fixture(name, "", ParameterTypes::new())
}

/// Empty fixture backed by a database file instead of MemoryIO, so
/// residency is bounded by the page cache rather than the whole database.
/// Any existing database at `path` (and its WAL) is removed first.
pub fn empty_fixture_on_disk(
    name: &str,
    path: &std::path::Path,
) -> Result<GraphFixture, RunnerError> {
    for suffix in ["", "-wal", "-shm"] {
        let target = format!("{}{suffix}", path.display());
        if std::path::Path::new(&target).exists() {
            std::fs::remove_file(&target)
                .map_err(|error| RunnerError::Fixture(error.to_string()))?;
        }
    }
    let io = Arc::new(
        turso_core::PlatformIO::new().map_err(|error| RunnerError::Fixture(error.to_string()))?,
    );
    build_fixture_with_io(
        name,
        "",
        ParameterTypes::new(),
        io,
        &path.display().to_string(),
    )
}

pub(crate) fn empty_fixture_with_parameters(
    name: &str,
    parameters: &Parameters,
) -> Result<GraphFixture, RunnerError> {
    build_fixture(name, "", parameter_types(parameters))
}

fn fixture(
    name: &str,
    scenario: &Scenario,
    parameter_types: ParameterTypes,
) -> Result<GraphFixture, RunnerError> {
    if name != "social" {
        return Err(RunnerError::Fixture(format!("unknown fixture `{name}`")));
    }
    build_fixture(
        scenario.id.as_str(),
        "INSERT INTO people VALUES (1, 'Ada', 40), (2, 'Grace', 35), (3, 'Linus', 30), (4, 'Edsger', 50); INSERT INTO relationships VALUES (10, 1, 2), (20, 2, 3), (30, 1, 3);",
        parameter_types,
    )
    .and_then(|fixture| {
        // setup_sql runs after fixture construction, so raw INSERTs into
        // people/relationships here would bypass the id/src/dst shadow-column
        // backfill in build_fixture_with_io and read as NULL under Cypher
        // property filters. Seed rows through seed_sql instead.
        for sql in &scenario.setup_sql {
            fixture
                .connection
                .execute(sql)
                .map_err(|error| RunnerError::Fixture(error.to_string()))?;
        }
        Ok(fixture)
    })
}

fn build_fixture(
    name: &str,
    seed_sql: &str,
    parameter_types: ParameterTypes,
) -> Result<GraphFixture, RunnerError> {
    build_fixture_with_io(
        name,
        seed_sql,
        parameter_types,
        Arc::new(MemoryIO::new()),
        &format!(":memory:{name}"),
    )
}

fn build_fixture_with_io(
    name: &str,
    seed_sql: &str,
    parameter_types: ParameterTypes,
    io: Arc<dyn turso_core::IO>,
    path: &str,
) -> Result<GraphFixture, RunnerError> {
    let _ = name;
    let database = turso_graph_frontend::open_database_with_io(
        io,
        path,
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(true),
    )
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    let connection = database
        .connect()
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    turso_graph_temporal::install_temporal_extension(&connection);
    connection
        .execute("CREATE TYPE duration BASE TEXT; CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);")
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    // Registration only validates table schema (PRAGMA table_info), not
    // data, so it can run before seeding.
    let registered = register_graph(
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
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(crate::dynamic_catalog::DynamicCatalog::new(
            connection.clone(),
            registered.clone(),
            "people".to_owned(),
            "relationships".to_owned(),
        ));
    if !seed_sql.is_empty() {
        connection
            .execute(seed_sql)
            .map_err(|error| RunnerError::Fixture(error.to_string()))?;
        // `DynamicCatalog` routes the Cypher-visible `id`/`src`/`dst`
        // properties to shadow `cyprop_*` columns so donor writes of
        // non-integer `id` values never collide with the structural
        // identity/endpoint columns (dynamic_catalog.rs, commit
        // c70301053). This fixture seeds those structural columns
        // directly via raw SQL, bypassing that routing, so mirror the
        // seeded values into the shadow columns here — otherwise a
        // Cypher `{id: N}`/`src`/`dst` property reference against seeded
        // rows resolves the still-NULL shadow column instead.
        mirror_structural_columns_into_property_shadows(
            catalog.as_ref(),
            registered.id,
            &connection,
            "people",
            registered.node_sources[0].id,
            "relationships",
            registered.relationship_sources[0].id,
        )?;
    }
    connection
        .execute(format!(
            "INSERT INTO \"{}\"(source_id, node_id, label) \
             SELECT {}, id, 'Person' FROM people",
            turso_graph_frontend::labels_table_name(registered.id),
            registered.node_sources[0].id.get(),
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    connection
        .execute(format!(
            "INSERT INTO \"{}\"(source_id, relationship_id, type) \
             SELECT {}, id, 'KNOWS' FROM relationships",
            turso_graph_frontend::relationship_types_table_name(registered.id),
            registered.relationship_sources[0].id.get(),
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    let session = GraphConnection::install(
        connection.clone(),
        &registered,
        catalog,
        parameter_types,
        Arc::new(SnapshotStore::default()),
        Default::default(),
    )
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    Ok(GraphFixture {
        connection,
        session,
        node_source: registered.node_sources[0].id,
        relationship_source: registered.relationship_sources[0].id,
        labels_table: turso_graph_frontend::labels_table_name(registered.id),
        types_table: turso_graph_frontend::relationship_types_table_name(registered.id),
    })
}

/// Copies the raw-SQL-seeded structural `id` (nodes) and `id`/`src`/`dst`
/// (relationships) column values into the `cyprop_*` shadow columns
/// `DynamicCatalog` resolves those Cypher property names onto (see
/// dynamic_catalog.rs). Provisions each shadow column on first touch via
/// the catalog's own `property` resolution — the same path a Cypher query
/// would take — so the column and its `ResolvedProperty`/`property_column`
/// mapping stay consistent with subsequent query compilation.
fn mirror_structural_columns_into_property_shadows(
    catalog: &dyn GraphCompilationCatalog,
    graph: ir::GraphId,
    connection: &Arc<Connection>,
    node_table: &str,
    node_source: ir::SourceTableId,
    relationship_table: &str,
    relationship_source: ir::SourceTableId,
) -> Result<(), RunnerError> {
    let shadow_column = |entity: CatalogEntity, name: &str, source: ir::SourceTableId| {
        let property = catalog.property(graph, entity, name).ok_or_else(|| {
            RunnerError::Fixture(format!("failed to provision `{name}` shadow column"))
        })?;
        catalog.property_column(source, property.id).ok_or_else(|| {
            RunnerError::Fixture(format!("no physical column for `{name}` shadow property"))
        })
    };

    let node_id = shadow_column(CatalogEntity::Node, "id", node_source)?;
    connection
        .execute(format!(
            "UPDATE \"{node_table}\" SET \"{node_id}\" = \"id\""
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;

    let relationship_id = shadow_column(CatalogEntity::Relationship, "id", relationship_source)?;
    let relationship_src = shadow_column(CatalogEntity::Relationship, "src", relationship_source)?;
    let relationship_dst = shadow_column(CatalogEntity::Relationship, "dst", relationship_source)?;
    connection
        .execute(format!(
            "UPDATE \"{relationship_table}\" SET \"{relationship_id}\" = \"id\", \
             \"{relationship_src}\" = \"src\", \"{relationship_dst}\" = \"dst\""
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    Ok(())
}

fn parameter_types(parameters: &Parameters) -> ParameterTypes {
    parameters
        .iter()
        .map(|(name, value)| {
            let (value_type, nullability) = match value {
                Value::Null => (ir::ValueType::Any, ir::Nullability::Nullable),
                Value::Numeric(Numeric::Integer(_)) => {
                    (ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                Value::Numeric(Numeric::Float(_)) => {
                    (ir::ValueType::Real, ir::Nullability::NonNull)
                }
                // TCK list/map parameters arrive as JSON text (the
                // frontend's list/map encoding); type them accordingly.
                Value::Text(text) if text.value.starts_with('[') => (
                    ir::ValueType::List(Box::new(ir::ValueType::Any)),
                    ir::Nullability::NonNull,
                ),
                Value::Text(text) if text.value.starts_with('{') => {
                    (ir::ValueType::Map, ir::Nullability::NonNull)
                }
                Value::Text(_) => (ir::ValueType::Text, ir::Nullability::NonNull),
                Value::Blob(_) => (ir::ValueType::Bytes, ir::Nullability::NonNull),
            };
            (name.clone(), (value_type, nullability))
        })
        .collect()
}

fn parameters(values: &BTreeMap<String, toml::Value>) -> Result<Parameters, RunnerError> {
    values
        .iter()
        .map(|(name, value)| {
            let value = match value {
                toml::Value::String(value) => Value::build_text(value.clone()),
                toml::Value::Integer(value) => Value::from_i64(*value),
                toml::Value::Float(value) => Value::from_f64(*value),
                toml::Value::Boolean(value) => Value::from_i64(i64::from(*value)),
                value => {
                    return Err(RunnerError::Parameter {
                        name: name.clone(),
                        value: value.clone(),
                    });
                }
            };
            Ok((name.clone(), value))
        })
        .collect::<Result<HashMap<_, _>, _>>()
}

fn execute(
    session: &GraphConnection,
    scenario: &Scenario,
    parameters: &Parameters,
) -> Result<Vec<Vec<String>>, String> {
    let rows = match scenario.action.as_str() {
        "query" => session.query(&scenario.query, parameters),
        "mutation" => match scenario.verification_query.as_deref() {
            Some(verification) => session
                .execute(&scenario.query, parameters)
                .and_then(|_| session.query(verification, parameters)),
            // Expected-error scenarios carry no verification query; when the
            // mutation unexpectedly succeeds, classify sees empty rows.
            None => session
                .execute(&scenario.query, parameters)
                .map(|_| Vec::new()),
        },
        _ => unreachable!("manifest validation constrains actions"),
    }
    .map_err(|error| error.to_string())?;
    let mut rows = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|value| match value {
                    Value::Null => "<null>".to_owned(),
                    value => value.to_string(),
                })
                .collect()
        })
        .collect::<Vec<Vec<String>>>();
    if scenario.ordering == "unordered" {
        rows.sort();
    }
    Ok(rows)
}

/// A scenario declares `ordering: unordered` when its query defines no order.
/// The digest reads the same field the comparison does, so a scenario cannot
/// be compared as a multiset while being recorded as a sequence.
fn scenario_ordering(scenario: &Scenario) -> ResultOrdering {
    if scenario.ordering == "unordered" {
        ResultOrdering::Unordered
    } else {
        ResultOrdering::Ordered
    }
}

fn classify(
    scenario: &Scenario,
    result: Result<Vec<Vec<String>>, String>,
) -> (Outcome, Option<Vec<Vec<String>>>, Option<String>) {
    match (scenario.expectation, result) {
        (crate::model::Expectation::Rows, Ok(rows)) => {
            let mut expected = scenario.expected_rows.clone();
            if scenario.ordering == "unordered" {
                expected.sort();
            }
            if rows == expected {
                (Outcome::Passed, Some(rows), None)
            } else {
                (
                    Outcome::Failed,
                    Some(rows.clone()),
                    Some(format!("expected {expected:?}, observed {rows:?}")),
                )
            }
        }
        (crate::model::Expectation::Rows, Err(error)) => (Outcome::Failed, None, Some(error)),
        (crate::model::Expectation::Error, Err(error)) => {
            expected_error(scenario, error, Outcome::Passed)
        }
        (crate::model::Expectation::Unsupported, Err(error)) => {
            expected_error(scenario, error, Outcome::Failed)
        }
        (crate::model::Expectation::Error, Ok(rows)) => (
            Outcome::Failed,
            Some(rows),
            Some("expected an error but execution succeeded".to_owned()),
        ),
        (crate::model::Expectation::Unsupported, Ok(rows)) => (
            Outcome::Failed,
            Some(rows),
            Some("known unsupported scenario succeeded and requires reclassification".to_owned()),
        ),
    }
}

fn expected_error(
    scenario: &Scenario,
    error: String,
    success: Outcome,
) -> (Outcome, Option<Vec<Vec<String>>>, Option<String>) {
    let expected = scenario
        .expected_error_contains
        .as_deref()
        .expect("validated error expectation has a pattern");
    if error.contains(expected) {
        (success, None, Some(error))
    } else {
        (
            Outcome::Failed,
            None,
            Some(format!(
                "expected error containing {expected:?}, observed {error:?}"
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_keeps_database_alive_for_graph_queries() {
        let fixture = empty_fixture("database-lifetime").expect("fixture should initialize");
        let connection = Arc::downgrade(&fixture.connection);

        let rows = fixture
            .session
            .query("RETURN 1 AS value", &Parameters::new())
            .expect("standalone projection should execute over one unit row");
        assert_eq!(rows, vec![vec![Value::from_i64(1)]]);

        let rows = fixture
            .session
            .query("MATCH (n:Person) RETURN n.name", &Parameters::new())
            .expect("fixture storage should remain available after construction");

        assert!(rows.is_empty());
        drop(fixture);
        assert!(
            connection.upgrade().is_none(),
            "dropping a fixture must release its connection"
        );
    }

    #[test]
    fn duration_values_construct_access_and_shift_datetimes() {
        let fixture = empty_fixture("duration-smoke").expect("fixture should initialize");
        let text = |query: &str| {
            let rows = fixture
                .session
                .query(query, &Parameters::new())
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
            rows[0][0].to_string()
        };

        assert_eq!(
            text("RETURN duration('P1Y2M3DT4H5M6S') AS d"),
            "P1Y2M3DT4H5M6S"
        );
        assert_eq!(text("RETURN duration({years: 1, days: 2}) AS d"), "P1Y2D");
        // Cypher duration components do not cross fields: days stay days.
        assert_eq!(text("RETURN duration('P1DT1H').hours AS h"), "1");
        assert_eq!(
            text("RETURN datetime('2024-01-31T00:00:00Z') + duration({months: 1}) AS d"),
            "2024-02-29T00:00Z"
        );
        assert_eq!(
            text(
                "RETURN duration.between(datetime('2024-01-01T00:00:00Z'), \
                 datetime('2024-01-02T06:00:00Z')) AS d"
            ),
            "P1DT6H"
        );
    }

    #[test]
    fn entity_set_forms_and_merge_actions_update_rows() {
        let fixture = empty_fixture("set-forms").expect("fixture should initialize");
        let parameters = Parameters::new();
        let mutate = |query: &str| {
            fixture
                .session
                .execute(query, &parameters)
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
        };
        let row = |query: &str| {
            let rows = fixture
                .session
                .query(query, &parameters)
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
            rows[0]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("|")
        };

        mutate("CREATE (:Person {name: 'Ada', age: 1})");
        mutate("MATCH (n) SET n += {age: 2, city: 'London'}");
        assert_eq!(
            row("MATCH (n) RETURN n.name, n.age, n.city"),
            "Ada|2|London"
        );
        // Replace clears every property the map omits.
        mutate("MATCH (n) SET n = {name: 'Grace'}");
        assert_eq!(row("MATCH (n) RETURN n.name, n.age, n.city"), "Grace||");
        mutate("MERGE (m:Person {name: 'Grace'}) ON MATCH SET m.age = 42");
        assert_eq!(row("MATCH (n {name: 'Grace'}) RETURN n.age"), "42");
        mutate("MERGE (m:Person {name: 'Hopper'}) ON CREATE SET m.age = 7");
        assert_eq!(row("MATCH (n {name: 'Hopper'}) RETURN n.age"), "7");
    }

    #[test]
    fn properties_reads_and_copies_whole_entities() {
        let fixture = empty_fixture("properties-smoke").expect("fixture should initialize");
        let parameters = Parameters::new();
        let mutate = |query: &str| {
            fixture
                .session
                .execute(query, &parameters)
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
        };
        let row = |query: &str| {
            let rows = fixture
                .session
                .query(query, &parameters)
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
            rows[0]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("|")
        };
        mutate("CREATE (:Person {name: 'Ada', age: 40})");
        // Null-valued and absent properties stay out of the map.
        assert_eq!(
            row("MATCH (n {name: 'Ada'}) RETURN properties(n)"),
            "{\"name\":\"Ada\",\"age\":40}"
        );
        mutate("CREATE (:Person {name: 'Grace'})");
        mutate(
            "MATCH (a {name: 'Ada'}) WITH a MATCH (b {name: 'Grace'}) \
             SET b = properties(a)",
        );
        assert_eq!(row("MATCH (n) WHERE n.age = 40 RETURN count(n)"), "2");
    }

    #[test]
    fn mutation_return_supports_order_and_distinct() {
        let fixture = empty_fixture("return-order").expect("fixture should initialize");
        let parameters = Parameters::new();
        let summary = fixture
            .session
            .execute(
                "UNWIND [3, 1, 2, 1] AS x CREATE (:Person {age: x}) \
                 RETURN DISTINCT x ORDER BY x DESC",
                &parameters,
            )
            .expect("mutation with ordered distinct return");
        let rows: Vec<String> = summary.rows.iter().map(|row| row[0].to_string()).collect();
        assert_eq!(rows, vec!["3", "2", "1"]);
    }

    #[test]
    fn bound_relationship_list_constrains_variable_length_match() {
        let fixture = empty_fixture("bound-rellist").expect("fixture should initialize");
        let parameters = Parameters::new();
        for statement in [
            "CREATE (:Person {name: 'A'})",
            "CREATE (:Person {name: 'B'})",
            "CREATE (:Person {name: 'C'})",
            "MATCH (a {name: 'A'}) MATCH (b {name: 'B'}) CREATE (a)-[:KNOWS]->(b)",
            "MATCH (b {name: 'B'}) MATCH (c {name: 'C'}) CREATE (b)-[:KNOWS]->(c)",
        ] {
            fixture
                .session
                .execute(statement, &parameters)
                .expect("seed");
        }
        // TCK Match4 [8] shape: capture two relationships as a list, then
        // match the variable-length path bound to exactly that list.
        let rows = fixture
            .session
            .query(
                "MATCH ()-[r1]->()-[r2]->() WITH [r1, r2] AS rs LIMIT 1 \
                 MATCH (first)-[rs*]->(second) RETURN first.name, second.name",
                &parameters,
            )
            .expect("bound-list variable-length match");
        let rendered: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|v| v.to_string()).collect())
            .collect();
        assert_eq!(rendered, vec![vec!["A".to_owned(), "C".to_owned()]]);
    }

    #[test]
    fn explain_prefix_returns_core_query_plan() {
        let fixture = empty_fixture("explain-smoke").expect("fixture should initialize");
        let rows = fixture
            .session
            .query(
                "EXPLAIN (VERBOSE, COSTS OFF) MATCH (n:Person) RETURN n.name",
                &Parameters::new(),
            )
            .expect("explain should compile through core's plan output");
        assert!(!rows.is_empty(), "plan output should have rows");
    }

    #[test]
    fn optional_match_count_groups_stay_correlated() {
        let fixture = empty_fixture("opt-count").expect("fixture should initialize");
        let parameters = Parameters::new();
        fixture
            .session
            .execute(
                "CREATE (:Person {name: 'Ada'}) CREATE (:Person {name: 'Bob'})",
                &parameters,
            )
            .expect("seed");
        fixture
            .session
            .execute(
                "MATCH (a {name: 'Ada'}) MATCH (b {name: 'Bob'}) CREATE (a)-[:KNOWS]->(b)",
                &parameters,
            )
            .expect("edge");
        // Ada knows exactly one person; the optional-match count must be
        // correlated per row, not a cross-product over every relationship.
        let rows = fixture
            .session
            .query(
                "MATCH (n {name: 'Ada'}) OPTIONAL MATCH (n)-[:KNOWS]->(m)                  WITH n, count(DISTINCT m) AS num RETURN n.name, num",
                &parameters,
            )
            .expect("query");
        let rendered: Vec<String> = rows[0].iter().map(|v| v.to_string()).collect();
        assert_eq!(rendered, vec!["Ada".to_owned(), "1".to_owned()]);
        // Same shape with a label on the reused variable and an incoming
        // direction — the CypherBench failing form.
        let rows = fixture
            .session
            .query(
                "MATCH (n:Person {name: 'Bob'}) OPTIONAL MATCH (n:Person)<-[:KNOWS]-(m)                  WITH n, count(DISTINCT m) AS num RETURN n.name, num",
                &parameters,
            )
            .expect("labeled query");
        let rendered: Vec<String> = rows[0].iter().map(|v| v.to_string()).collect();
        assert_eq!(rendered, vec!["Bob".to_owned(), "1".to_owned()]);
    }

    /// CypherBench b5008120 shape: the first MATCH triggers anchor
    /// reversal (constant-property far node), then a correlated OPTIONAL
    /// MATCH aggregates per n. Ada knows Bob and Carol; Bob has 1 knower,
    /// Carol 2. Regression coverage for the optional-chain boundary fix:
    /// the mandatory left plan must lower with inner joins and applied
    /// filters even when the optional chain sits above it.
    #[test]
    fn anchor_reversal_keeps_optional_match_groups() {
        let fixture = empty_fixture("opt-reversal").expect("fixture should initialize");
        let parameters = Parameters::new();
        for statement in [
            "CREATE (:Person {name: 'Ada'}) CREATE (:Person {name: 'Bob'})",
            "CREATE (:Person {name: 'Carol'}) CREATE (:Person {name: 'Dan'})",
            "MATCH (a {name: 'Ada'}) MATCH (b {name: 'Bob'}) CREATE (a)-[:KNOWS]->(b)",
            "MATCH (a {name: 'Ada'}) MATCH (c {name: 'Carol'}) CREATE (a)-[:KNOWS]->(c)",
            "MATCH (d {name: 'Dan'}) MATCH (c {name: 'Carol'}) CREATE (d)-[:KNOWS]->(c)",
        ] {
            fixture
                .session
                .execute(statement, &parameters)
                .expect("seed");
        }
        let rows = fixture
            .session
            .query(
                "MATCH (n:Person)<-[:KNOWS]-(m1:Person {name: 'Ada'}) \
                 OPTIONAL MATCH (n:Person)<-[:KNOWS]-(m) \
                 WITH n, count(DISTINCT m) AS num \
                 RETURN n.name, num ORDER BY n.name",
                &parameters,
            )
            .expect("reversal + optional aggregate");
        let rendered: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|v| v.to_string()).collect())
            .collect();
        assert_eq!(
            rendered,
            vec![
                vec!["Bob".to_owned(), "1".to_owned()],
                vec!["Carol".to_owned(), "2".to_owned()],
            ]
        );
    }

    #[test]
    fn correlated_and_optional_match_after_mutation() {
        let fixture = empty_fixture("correlated-staged").expect("fixture should initialize");
        let parameters = Parameters::new();
        // TCK Match8 [2] shape: MERGE then re-match both endpoints.
        let summary = fixture
            .session
            .execute(
                "CREATE (a:Person {name: 'X'}) \
                 CREATE (b:Person {name: 'Y'}) \
                 CREATE (a)-[:KNOWS]->(b) \
                 WITH * MATCH (a)-[e:KNOWS]->(b) RETURN count(*)",
                &parameters,
            )
            .expect("correlated staged match");
        assert_eq!(summary.rows[0][0].to_string(), "1");
        // Optional staged match keeps the row with null outputs.
        let summary = fixture
            .session
            .execute(
                "CREATE (c:Person {name: 'Z'}) \
                 WITH * OPTIONAL MATCH (c)-[m:KNOWS]->(unmatched) RETURN count(*)",
                &parameters,
            )
            .expect("optional staged match");
        assert_eq!(summary.rows[0][0].to_string(), "1");
    }

    #[test]
    fn match_after_mutation_joins_current_rows() {
        let fixture = empty_fixture("staged-match").expect("fixture should initialize");
        let parameters = Parameters::new();
        let mutate = |query: &str| {
            fixture
                .session
                .execute(query, &parameters)
                .unwrap_or_else(|error| panic!("{query} failed: {error}"));
        };
        mutate("CREATE (:Person {name: 'A'})");
        // The staged MATCH sees the freshly created node too.
        mutate("CREATE (:Person {name: 'B'}) WITH * MATCH (n) SET n.tag = 1");
        let rows = fixture
            .session
            .query("MATCH (n) RETURN count(n.tag)", &parameters)
            .expect("count query");
        assert_eq!(rows[0][0].to_string(), "2");
    }

    /// Regression for the interaction between `DynamicCatalog`'s
    /// `id`/`src`/`dst` shadow-column routing (dynamic_catalog.rs, commit
    /// c70301053) and raw-SQL fixture seeding: the shadow columns start
    /// out NULL, so a Cypher `{id: N}`/`src`/`dst` property reference
    /// against rows seeded directly into the real structural columns must
    /// still see those values, not the empty shadow.
    #[test]
    fn raw_seeded_structural_columns_are_visible_as_cypher_properties() {
        let fixture = build_fixture(
            "raw-seeded-id-property",
            "INSERT INTO people VALUES (1, 'Ada', 40), (2, 'Grace', 35); \
             INSERT INTO relationships VALUES (10, 1, 2);",
            ParameterTypes::new(),
        )
        .expect("fixture should initialize");
        let parameters = Parameters::new();

        let rows = fixture
            .session
            .query("MATCH (n:Person {id: 1}) RETURN n.name", &parameters)
            .expect("id property filter should execute");
        assert_eq!(rows[0][0].to_string(), "Ada");

        let rows = fixture
            .session
            .query(
                "MATCH ()-[r:KNOWS {id: 10}]->() RETURN r.src, r.dst",
                &parameters,
            )
            .expect("relationship id/src/dst property reads should execute");
        assert_eq!(rows[0][0].to_string(), "1");
        assert_eq!(rows[0][1].to_string(), "2");
    }
}
