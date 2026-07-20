//! CypherBench execution benchmark: Wikidata-derived property graphs with
//! gold Cypher queries and pre-verified results (megagonlabs/cypherbench,
//! Apache-2.0). Runs as a performance/accuracy benchmark, deliberately
//! outside the conformance corpus: queries are template-generated
//! analytics, not clause-semantics coverage.

use std::{collections::HashMap, fs, path::Path, time::Instant};

use anyhow::{Context, Result};
use serde::Deserialize;
use turso_graph_frontend::MutationParameters;

use crate::runner::{empty_fixture, GraphFixture};

#[derive(Debug, Deserialize)]
pub struct SimpleKg {
    pub entities: Vec<KgEntity>,
    pub relations: Vec<KgRelation>,
}

#[derive(Debug, Deserialize)]
pub struct KgEntity {
    pub eid: String,
    pub label: String,
    pub name: Option<String>,
    #[serde(default)]
    pub properties: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct KgRelation {
    pub label: String,
    pub subj_id: String,
    pub obj_id: String,
    #[serde(default)]
    pub properties: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BenchTask {
    pub qid: String,
    pub graph: String,
    pub gold_cypher: String,
    pub answer_json: String,
}

#[derive(Debug, serde::Serialize)]
pub struct DomainReport {
    pub domain: String,
    pub entities: usize,
    pub relations: usize,
    pub load_ms: u64,
    pub queries: usize,
    pub matched: usize,
    pub mismatched: usize,
    pub errored: usize,
    pub query_ms_total: u64,
}

fn sql_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn scalar_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_owned(),
        serde_json::Value::Bool(b) => if *b { "1" } else { "0" }.to_owned(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", sql_quote(s)),
        other => format!("'{}'", sql_quote(&other.to_string())),
    }
}

/// Loads a SimpleKG graph into a fresh fixture: entities into the node
/// table (one ALTERed column per property, reserved names prefixed),
/// relations into the relationship table, labels and types into the
/// junction tables the frontend scans.
pub fn load_graph(kg: &SimpleKg) -> Result<GraphFixture> {
    let fixture = empty_fixture("cypherbench").context("fixture")?;
    let connection = &fixture.connection;
    let mut columns: HashMap<String, String> = HashMap::new();
    let mut ensure_column = |connection: &std::sync::Arc<turso_core::Connection>,
                             table: &str,
                             name: &str|
     -> Result<String> {
        let key = format!("{table}.{name}");
        if let Some(column) = columns.get(&key) {
            return Ok(column.clone());
        }
        let physical = match (table, name) {
            (_, "id") | ("relationships", "src") | ("relationships", "dst") => {
                format!("cyprop_{name}")
            }
            ("people", "name") | ("people", "age") => name.to_owned(),
            _ => name.to_owned(),
        };
        if !(table == "people" && (physical == "name" || physical == "age")) {
            connection
                .execute(format!(
                    "ALTER TABLE {table} ADD COLUMN \"{}\"",
                    physical.replace('"', "\"\"")
                ))
                .with_context(|| format!("add column {physical}"))?;
        }
        columns.insert(key, physical.clone());
        Ok(physical)
    };

    // Pre-provision every property column, then batch multi-row inserts
    // grouped by column signature inside one transaction.
    for entity in &kg.entities {
        for key in entity.properties.keys() {
            ensure_column(connection, "people", key)?;
        }
    }
    for relation in &kg.relations {
        for key in relation.properties.keys() {
            ensure_column(connection, "relationships", key)?;
        }
    }
    const CHUNK: usize = 400;
    let flush = |connection: &std::sync::Arc<turso_core::Connection>,
                 table: &str,
                 names: &str,
                 rows: &mut Vec<String>|
     -> Result<()> {
        for chunk in rows.chunks(CHUNK) {
            connection
                .execute(format!(
                    "INSERT INTO {table}({names}) VALUES {}",
                    chunk.join(", ")
                ))
                .with_context(|| format!("bulk insert into {table}"))?;
        }
        rows.clear();
        Ok(())
    };
    connection.execute("BEGIN")?;
    let mut ids: HashMap<&str, i64> = HashMap::new();
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let mut labels = Vec::new();
    for (index, entity) in kg.entities.iter().enumerate() {
        let id = index as i64 + 1;
        ids.insert(entity.eid.as_str(), id);
        let mut names = vec!["id".to_owned()];
        let mut values = vec![id.to_string()];
        if let Some(name) = &entity.name {
            names.push("name".to_owned());
            values.push(format!("'{}'", sql_quote(name)));
        }
        for (key, value) in &entity.properties {
            if value.is_null() {
                continue;
            }
            names.push(format!(
                "\"{}\"",
                ensure_column(connection, "people", key)?.replace('"', "\"\"")
            ));
            values.push(scalar_sql(value));
        }
        groups
            .entry(names.join(", "))
            .or_default()
            .push(format!("({})", values.join(", ")));
        labels.push(format!("({id}, '{}')", sql_quote(&entity.label)));
    }
    for (names, mut rows) in groups.drain() {
        flush(connection, "people", &names, &mut rows)?;
    }
    flush(
        connection,
        &format!("\"{}\"", fixture.labels_table.replace('"', "\"\"")),
        "node_id, label",
        &mut labels,
    )?;
    let mut types = Vec::new();
    for (index, relation) in kg.relations.iter().enumerate() {
        let id = index as i64 + 1;
        let (Some(subject), Some(object)) = (
            ids.get(relation.subj_id.as_str()),
            ids.get(relation.obj_id.as_str()),
        ) else {
            continue;
        };
        let mut names = vec!["id".to_owned(), "src".to_owned(), "dst".to_owned()];
        let mut values = vec![id.to_string(), subject.to_string(), object.to_string()];
        for (key, value) in &relation.properties {
            if value.is_null() {
                continue;
            }
            names.push(format!(
                "\"{}\"",
                ensure_column(connection, "relationships", key)?.replace('"', "\"\"")
            ));
            values.push(scalar_sql(value));
        }
        groups
            .entry(names.join(", "))
            .or_default()
            .push(format!("({})", values.join(", ")));
        types.push(format!("({id}, '{}')", sql_quote(&relation.label)));
    }
    for (names, mut rows) in groups.drain() {
        flush(connection, "relationships", &names, &mut rows)?;
    }
    flush(
        connection,
        &format!("\"{}\"", fixture.types_table.replace('"', "\"\"")),
        "relationship_id, type",
        &mut types,
    )?;
    connection.execute("COMMIT")?;
    Ok(fixture)
}

/// Runs one domain: load, then execute its gold queries, comparing row
/// sets (order-insensitive, stringified) against the pre-verified answer.
pub fn run_domain(
    domain: &str,
    graph_path: &Path,
    tasks: &[BenchTask],
    limit: Option<usize>,
) -> Result<DomainReport> {
    let kg: SimpleKg = serde_json::from_str(
        &fs::read_to_string(graph_path)
            .with_context(|| format!("reading {}", graph_path.display()))?,
    )
    .context("parsing SimpleKG")?;
    let started = Instant::now();
    let fixture = load_graph(&kg)?;
    let load_ms = started.elapsed().as_millis() as u64;
    eprintln!("{domain}: loaded in {load_ms}ms");

    let selected: Vec<&BenchTask> = tasks
        .iter()
        .filter(|task| task.graph == domain)
        .take(limit.unwrap_or(usize::MAX))
        .collect();
    let mut matched = 0;
    let mut mismatched = 0;
    let mut errored = 0;
    let mut query_ms_total = 0;
    let parameters = MutationParameters::new();
    for task in &selected {
        let query_started = Instant::now();
        eprintln!("  [{}] start", task.qid);
        match fixture.session.query(&task.gold_cypher, &parameters) {
            Ok(rows) => {
                query_ms_total += query_started.elapsed().as_millis() as u64;
                let mut observed: Vec<Vec<String>> = rows
                    .into_iter()
                    .map(|row| row.into_iter().map(|value| value.to_string()).collect())
                    .collect();
                let expected: Vec<Vec<serde_json::Value>> =
                    serde_json::from_str(&task.answer_json).unwrap_or_default();
                let mut expected: Vec<Vec<String>> = expected
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|value| match value {
                                serde_json::Value::String(s) => s,
                                serde_json::Value::Null => String::new(),
                                other => other.to_string(),
                            })
                            .collect()
                    })
                    .collect();
                eprintln!(
                    "  [{}] ok {}ms rows={}",
                    task.qid,
                    query_started.elapsed().as_millis(),
                    observed.len()
                );
                observed.sort();
                expected.sort();
                if observed == expected {
                    matched += 1;
                } else {
                    mismatched += 1;
                }
            }
            Err(error) => {
                query_ms_total += query_started.elapsed().as_millis() as u64;
                eprintln!(
                    "  [{}] err {}ms {}",
                    task.qid,
                    query_started.elapsed().as_millis(),
                    error.to_string().chars().take(80).collect::<String>()
                );
                errored += 1;
            }
        }
    }
    Ok(DomainReport {
        domain: domain.to_owned(),
        entities: kg.entities.len(),
        relations: kg.relations.len(),
        load_ms,
        queries: selected.len(),
        matched,
        mismatched,
        errored,
        query_ms_total,
    })
}

pub fn load_tasks(path: &Path) -> Result<Vec<BenchTask>> {
    serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?,
    )
    .context("parsing test.json")
}
