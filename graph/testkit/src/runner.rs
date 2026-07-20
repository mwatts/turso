use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};

use thiserror::Error;
use turso_core::{
    Connection, Database, DatabaseOpts, MemoryIO, Numeric, OpenFlags, SqliteDialect, Value,
};
use turso_graph_frontend::{
    register_graph, GraphCompilationCatalog, GraphRegistration, GraphSession, MutationParameters,
    NodeSourceRegistration, ParameterTypes, RelationshipSourceRegistration, SnapshotStore,
};
use turso_graph_ir as ir;

use crate::{
    history::{recorded_at, result_digest},
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
        let result_digest = rows.as_ref().map(|rows| result_digest(rows));
        Ok(ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
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

pub(crate) struct GraphFixture {
    pub(crate) connection: Arc<Connection>,
    pub(crate) session: GraphSession,
}

pub(crate) fn empty_fixture(name: &str) -> Result<GraphFixture, RunnerError> {
    build_fixture(name, "", ParameterTypes::new())
}

pub(crate) fn empty_fixture_with_parameters(
    name: &str,
    parameters: &MutationParameters,
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
    let database = Database::open_file_with_flags(
        Arc::new(MemoryIO::new()),
        &format!(":memory:{name}"),
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(true),
        None,
        Arc::new(SqliteDialect),
    )
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    let connection = database
        .connect()
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    turso_graph_temporal::install_temporal_extension(&connection);
    connection
        .execute("CREATE TYPE duration BASE TEXT; CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);")
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    if !seed_sql.is_empty() {
        connection
            .execute(seed_sql)
            .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    }
    let registered = register_graph(
        &connection,
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
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    connection
        .execute(format!(
            "INSERT INTO \"{}\"(node_id, label) SELECT id, 'Person' FROM people",
            turso_graph_frontend::labels_table_name(registered.id)
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    connection
        .execute(format!(
            "INSERT INTO \"{}\"(relationship_id, type) SELECT id, 'KNOWS' FROM relationships",
            turso_graph_frontend::relationship_types_table_name(registered.id)
        ))
        .map_err(|error| RunnerError::Fixture(error.to_string()))?;
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(crate::dynamic_catalog::DynamicCatalog::new(
            connection.clone(),
            registered.clone(),
            "people".to_owned(),
            "relationships".to_owned(),
        ));
    let session = GraphSession::install(
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
    })
}

fn parameter_types(parameters: &MutationParameters) -> ParameterTypes {
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

fn parameters(values: &BTreeMap<String, toml::Value>) -> Result<MutationParameters, RunnerError> {
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
    session: &GraphSession,
    scenario: &Scenario,
    parameters: &MutationParameters,
) -> Result<Vec<Vec<String>>, String> {
    let rows = match scenario.action.as_str() {
        "query" => session.query(&scenario.query, parameters),
        "mutation" => match scenario.verification_query.as_deref() {
            Some(verification) => session
                .mutate(&scenario.query, parameters)
                .and_then(|_| session.query(verification, parameters)),
            // Expected-error scenarios carry no verification query; when the
            // mutation unexpectedly succeeds, classify sees empty rows.
            None => session
                .mutate(&scenario.query, parameters)
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
            .query("RETURN 1 AS value", &MutationParameters::new())
            .expect("standalone projection should execute over one unit row");
        assert_eq!(rows, vec![vec![Value::from_i64(1)]]);

        let rows = fixture
            .session
            .query("MATCH (n:Person) RETURN n.name", &MutationParameters::new())
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
                .query(query, &MutationParameters::new())
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
        let parameters = MutationParameters::new();
        let mutate = |query: &str| {
            fixture
                .session
                .mutate(query, &parameters)
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
    fn match_after_mutation_joins_current_rows() {
        let fixture = empty_fixture("staged-match").expect("fixture should initialize");
        let parameters = MutationParameters::new();
        let mutate = |query: &str| {
            fixture
                .session
                .mutate(query, &parameters)
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
}
