use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path, time::Instant};

use anyhow::{Context, Result};
use serde::Deserialize;
use turso_graph_frontend::MutationParameters;

use crate::{
    history::{recorded_at, result_digest},
    identity::TestId,
    model::{
        Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
        HISTORY_SCHEMA_VERSION,
    },
    runner::{empty_fixture, GraphFixture},
};

#[derive(Debug, Deserialize)]
pub struct PerformanceManifest {
    version: u32,
    profile: Vec<PerformanceProfile>,
}

#[derive(Debug, Deserialize)]
struct PerformanceProfile {
    name: String,
    scales: Vec<u64>,
    iterations: u32,
}

impl PerformanceManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let manifest: Self = toml::from_str(
            &fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?,
        )
        .with_context(|| format!("parsing {}", path.display()))?;
        anyhow::ensure!(
            manifest.version == 1,
            "unsupported performance manifest version"
        );
        for profile in &manifest.profile {
            anyhow::ensure!(
                matches!(profile.name.as_str(), "smoke" | "deep")
                    && !profile.scales.is_empty()
                    && profile.scales.iter().all(|scale| *scale > 1)
                    && profile.iterations > 0,
                "invalid performance profile `{}`",
                profile.name
            );
        }
        Ok(manifest)
    }

    pub fn run(
        &self,
        profile_name: &str,
        environment: RunEnvironment,
        run_id: &str,
    ) -> Result<Vec<ResultRecord>> {
        let profile = self
            .profile
            .iter()
            .find(|profile| profile.name == profile_name)
            .with_context(|| format!("unknown performance profile `{profile_name}`"))?;
        let mut records = Vec::new();
        for &scale in &profile.scales {
            records.extend(run_scale(
                profile_name,
                environment.clone(),
                run_id,
                scale,
                profile.iterations,
            ));
        }
        Ok(records)
    }
}

struct Measurement {
    duration_ns: u64,
    units: u64,
    rows: Vec<Vec<String>>,
    nodes: u64,
    relationships: u64,
}

fn run_scale(
    profile: &str,
    environment: RunEnvironment,
    run_id: &str,
    scale: u64,
    iterations: u32,
) -> Vec<ResultRecord> {
    ["create", "bulk-load", "load", "query", "delete"]
        .into_iter()
        .map(|operation| {
            let result = match operation {
                "create" => measure_create(scale),
                "bulk-load" => measure_bulk_load(scale),
                "load" => measure_load(scale),
                "query" => measure_query(scale, iterations),
                "delete" => measure_delete(scale),
                _ => unreachable!(),
            };
            record(
                profile,
                environment.clone(),
                run_id,
                operation,
                scale,
                iterations,
                result,
            )
        })
        .collect()
}

fn measure_create(scale: u64) -> Result<Measurement> {
    let fixture = empty_fixture(&format!("perf-create-{scale}"))?;
    let parameters = MutationParameters::new();
    let started = Instant::now();
    for id in 1..=scale {
        fixture.session.mutate(
            &format!("CREATE (:Person {{id: {id}, name: 'node-{id}', age: {id}}})"),
            &parameters,
        )?;
    }
    let duration_ns = elapsed_ns(started);
    validate_counts(&fixture, scale, 0)?;
    Ok(measurement(duration_ns, scale, Vec::new(), scale, 0))
}

fn measure_bulk_load(scale: u64) -> Result<Measurement> {
    let fixture = empty_fixture(&format!("perf-bulk-{scale}"))?;
    let sql = line_graph_sql(scale);
    let started = Instant::now();
    fixture.connection.execute(&sql)?;
    seed_labels(&fixture)?;
    let duration_ns = elapsed_ns(started);
    validate_counts(&fixture, scale, scale - 1)?;
    Ok(measurement(
        duration_ns,
        scale.saturating_mul(2).saturating_sub(1),
        Vec::new(),
        scale,
        scale - 1,
    ))
}

fn measure_load(scale: u64) -> Result<Measurement> {
    let fixture = empty_fixture(&format!("perf-load-{scale}"))?;
    fixture.connection.execute(line_graph_sql(scale))?;
    seed_labels(&fixture)?;
    let started = Instant::now();
    let rows = fixture.session.query(
        "MATCH (:Person {id: 1})-[:KNOWS*1..3]->(b:Person) RETURN count(b)",
        &MutationParameters::new(),
    )?;
    let duration_ns = elapsed_ns(started);
    anyhow::ensure!(
        stringify_rows(rows.clone()) == vec![vec![scale.saturating_sub(1).min(3).to_string()]],
        "snapshot load returned an unexpected traversal count"
    );
    validate_counts(&fixture, scale, scale - 1)?;
    Ok(measurement(
        duration_ns,
        scale.saturating_mul(2).saturating_sub(1),
        stringify_rows(rows),
        scale,
        scale - 1,
    ))
}

fn measure_query(scale: u64, iterations: u32) -> Result<Measurement> {
    let fixture = empty_fixture(&format!("perf-query-{scale}"))?;
    fixture.connection.execute(line_graph_sql(scale))?;
    seed_labels(&fixture)?;
    let query = format!("MATCH (n:Person {{id: {}}}) RETURN n.name", scale / 2);
    let parameters = MutationParameters::new();
    fixture.session.query(&query, &parameters)?;
    let started = Instant::now();
    let mut rows = Vec::new();
    for _ in 0..iterations {
        rows = stringify_rows(fixture.session.query(&query, &parameters)?);
    }
    let duration_ns = elapsed_ns(started);
    anyhow::ensure!(
        rows == vec![vec![format!("node-{}", scale / 2)]],
        "point query returned an unexpected row"
    );
    validate_counts(&fixture, scale, scale - 1)?;
    Ok(measurement(
        duration_ns,
        u64::from(iterations),
        rows,
        scale,
        scale - 1,
    ))
}

fn measure_delete(scale: u64) -> Result<Measurement> {
    let fixture = empty_fixture(&format!("perf-delete-{scale}"))?;
    fixture.connection.execute(line_graph_sql(scale))?;
    seed_labels(&fixture)?;
    let started = Instant::now();
    fixture.session.mutate(
        "MATCH (n:Person) DETACH DELETE n",
        &MutationParameters::new(),
    )?;
    let duration_ns = elapsed_ns(started);
    validate_counts(&fixture, 0, 0)?;
    Ok(measurement(duration_ns, scale, Vec::new(), 0, 0))
}

fn validate_counts(fixture: &GraphFixture, nodes: u64, relationships: u64) -> Result<()> {
    let parameters = MutationParameters::new();
    let actual_nodes = stringify_rows(
        fixture
            .session
            .query("MATCH (n:Person) RETURN count(n)", &parameters)?,
    );
    let actual_relationships = stringify_rows(
        fixture
            .session
            .query("MATCH ()-[r:KNOWS]->() RETURN count(r)", &parameters)?,
    );
    anyhow::ensure!(
        actual_nodes == vec![vec![nodes.to_string()]]
            && actual_relationships == vec![vec![relationships.to_string()]],
        "expected {nodes} nodes and {relationships} relationships, observed {actual_nodes:?} and {actual_relationships:?}"
    );
    Ok(())
}

fn measurement(
    duration_ns: u64,
    units: u64,
    rows: Vec<Vec<String>>,
    nodes: u64,
    relationships: u64,
) -> Measurement {
    Measurement {
        duration_ns,
        units,
        rows,
        nodes,
        relationships,
    }
}

fn elapsed_ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
}

fn record(
    profile: &str,
    environment: RunEnvironment,
    run_id: &str,
    operation: &str,
    scale: u64,
    iterations: u32,
    result: Result<Measurement>,
) -> ResultRecord {
    let measured_iterations = if operation == "query" { iterations } else { 1 };
    let test_id = TestId::parse(format!("perf.line.{operation}.s{scale:06}"))
        .expect("generated performance identity is valid");
    let source = SourceIdentity {
        name: "Turso graph frontend".to_owned(),
        repository: "https://github.com/tursodatabase/turso".to_owned(),
        revision: environment.git_commit.clone(),
        path: "graph/testkit/src/performance.rs".to_owned(),
        case: format!("line graph {operation} at scale {scale}"),
        license: "MIT".to_owned(),
        adaptation: "native-performance-case".to_owned(),
        issue: None,
        fixed_commit: None,
    };
    match result {
        Ok(measurement) => ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
            run_id: run_id.to_owned(),
            recorded_at: recorded_at(),
            environment,
            suite: format!("performance-{profile}"),
            test_id,
            kind: TestKind::Performance,
            area: "lifecycle".to_owned(),
            fixture: "line".to_owned(),
            expectation: Expectation::Rows,
            outcome: Outcome::Passed,
            duration_ns: measurement.duration_ns,
            source,
            operation: Some(operation.to_owned()),
            graph_shape: Some("line".to_owned()),
            scale: Some(scale),
            iterations: Some(measured_iterations),
            throughput_per_second: Some(
                measurement.units as f64 * 1_000_000_000.0 / measurement.duration_ns.max(1) as f64,
            ),
            row_count: Some(measurement.rows.len() as u64),
            node_count: Some(measurement.nodes),
            relationship_count: Some(measurement.relationships),
            result_digest: Some(result_digest(&measurement.rows)),
            message: None,
            dimensions: BTreeMap::from([("units".to_owned(), measurement.units.to_string())]),
        },
        Err(error) => ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
            run_id: run_id.to_owned(),
            recorded_at: recorded_at(),
            environment,
            suite: format!("performance-{profile}"),
            test_id,
            kind: TestKind::Performance,
            area: "lifecycle".to_owned(),
            fixture: "line".to_owned(),
            expectation: Expectation::Rows,
            outcome: Outcome::Failed,
            duration_ns: 0,
            source,
            operation: Some(operation.to_owned()),
            graph_shape: Some("line".to_owned()),
            scale: Some(scale),
            iterations: Some(measured_iterations),
            throughput_per_second: None,
            row_count: None,
            node_count: None,
            relationship_count: None,
            result_digest: None,
            message: Some(format!("{error:#}")),
            dimensions: BTreeMap::new(),
        },
    }
}

fn seed_labels(fixture: &crate::runner::GraphFixture) -> Result<(), turso_core::LimboError> {
    fixture.connection.execute(format!(
        "INSERT INTO \"{}\"(node_id, label) SELECT id, 'Person' FROM people \
         WHERE id NOT IN (SELECT node_id FROM \"{}\")",
        turso_graph_frontend::labels_table_name(fixture.session.graph_id()),
        turso_graph_frontend::labels_table_name(fixture.session.graph_id()),
    ))
}

fn line_graph_sql(scale: u64) -> String {
    let mut sql = "INSERT INTO people VALUES ".to_owned();
    for id in 1..=scale {
        if id > 1 {
            sql.push(',');
        }
        write!(sql, "({id},'node-{id}',{})", id % 100).unwrap();
    }
    sql.push_str("; INSERT INTO relationships VALUES ");
    for id in 1..scale {
        if id > 1 {
            sql.push(',');
        }
        write!(sql, "({id},{id},{})", id + 1).unwrap();
    }
    sql.push(';');
    sql
}

fn stringify_rows(rows: Vec<Vec<turso_core::Value>>) -> Vec<Vec<String>> {
    rows.into_iter()
        .map(|row| row.into_iter().map(|value| value.to_string()).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_graph_sql_has_stable_cardinality() {
        let sql = line_graph_sql(3);
        assert!(sql.contains("(3,'node-3',3)"));
        assert!(sql.contains("(2,2,3)"));
    }
}
