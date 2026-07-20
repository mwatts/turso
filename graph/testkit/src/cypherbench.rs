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

#[derive(Clone, Debug, Deserialize)]
pub struct BenchTask {
    pub qid: String,
    pub graph: String,
    pub gold_cypher: String,
    pub answer_json: String,
}

/// Per-query outcome for triage: verdict, timing, and a bounded sample of
/// the observed/expected divergence.
#[derive(Debug, serde::Serialize)]
pub struct QueryDetail {
    pub qid: String,
    pub domain: String,
    pub verdict: &'static str,
    pub duration_ms: u64,
    pub observed_rows: usize,
    pub expected_rows: usize,
    pub observed_sample: String,
    pub expected_sample: String,
    pub error: String,
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

/// Supervises a domain run on a worker thread with a per-query watchdog.
/// A query exceeding `query_timeout` records as "timeout" and the domain
/// is abandoned (the wedged worker detaches and dies with the process) —
/// upstream filtered its gold set to <=30s per query, so anything beyond
/// the timeout is a pathological plan, not a slow answer.
pub fn run_domain(
    domain: &str,
    graph_path: &Path,
    tasks: &[BenchTask],
    limit: Option<usize>,
    detail: Option<&mut Vec<QueryDetail>>,
    query_timeout: std::time::Duration,
) -> Result<DomainReport> {
    enum Event {
        Started(String),
        Query(QueryDetail),
        Done(Box<DomainReport>),
        Failed(String),
    }
    let spawn_worker = |skip: std::collections::HashSet<String>| {
        let (sender, receiver) = std::sync::mpsc::channel::<Event>();
        let domain = domain.to_owned();
        let graph_path = graph_path.to_owned();
        let tasks = tasks.to_vec();
        std::thread::spawn(move || {
            let query_sender = sender.clone();
            let started_sender = sender.clone();
            let mut sink = move |entry: QueryDetail| {
                let _ = query_sender.send(Event::Query(entry));
            };
            let mut notify_started = move |qid: &str| {
                let _ = started_sender.send(Event::Started(qid.to_owned()));
            };
            let result = run_domain_worker(
                &domain,
                &graph_path,
                &tasks,
                limit,
                &skip,
                &mut sink,
                &mut notify_started,
            );
            match result {
                Ok(report) => {
                    let _ = sender.send(Event::Done(Box::new(report)));
                }
                Err(error) => {
                    let _ = sender.send(Event::Failed(error.to_string()));
                }
            }
        });
        receiver
    };
    let mut skip: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut receiver = spawn_worker(skip.clone());
    let total = tasks
        .iter()
        .filter(|task| task.graph == domain)
        .take(limit.unwrap_or(usize::MAX))
        .count();
    let timeout_cap = (total / 10).max(1);
    let mut details_out = detail;
    let mut in_flight: Option<String> = None;
    let mut timeouts = 0_usize;
    let mut matched = 0;
    let mut mismatched = 0;
    let mut errored = 0;
    let mut query_ms_total = 0_u64;
    let mut seen = 0_usize;
    let record_timeout = |qid: String, details_out: &mut Option<&mut Vec<QueryDetail>>| {
        if let Some(details) = details_out.as_deref_mut() {
            details.push(QueryDetail {
                qid,
                domain: domain.to_owned(),
                verdict: "timeout",
                duration_ms: query_timeout.as_millis() as u64,
                observed_rows: 0,
                expected_rows: 0,
                observed_sample: String::new(),
                expected_sample: String::new(),
                error: "per-query watchdog expired".to_owned(),
            });
        }
    };
    loop {
        // The load phase (before any query starts) gets a generous fixed
        // window; queries get the per-query watchdog.
        let window = if in_flight.is_none() && seen == 0 {
            query_timeout.max(std::time::Duration::from_secs(300))
        } else {
            query_timeout
        };
        match receiver.recv_timeout(window) {
            Ok(Event::Started(qid)) => in_flight = Some(qid),
            Ok(Event::Query(entry)) => {
                in_flight = None;
                seen += 1;
                match entry.verdict {
                    "matched" => matched += 1,
                    "mismatched" => mismatched += 1,
                    _ => errored += 1,
                }
                query_ms_total += entry.duration_ms;
                if let Some(details) = details_out.as_deref_mut() {
                    details.push(entry);
                }
            }
            Ok(Event::Done(report)) => {
                let mut report = *report;
                report.errored += timeouts;
                report.queries += timeouts;
                report.query_ms_total += timeouts as u64 * query_timeout.as_millis() as u64;
                return Ok(report);
            }
            Ok(Event::Failed(error)) => {
                anyhow::bail!("{domain}: {error}");
            }
            Err(_) => {
                let qid = in_flight
                    .take()
                    .unwrap_or_else(|| format!("{domain}:load-or-stall"));
                timeouts += 1;
                record_timeout(qid.clone(), &mut details_out);
                // More than 10% of a domain timing out is itself the
                // signal: stop respawning and report the partial result.
                if timeouts > timeout_cap {
                    eprintln!(
                        "{domain}: {timeouts} timeouts exceed 10% of {total} \
                         queries; terminating domain"
                    );
                    return Ok(DomainReport {
                        domain: domain.to_owned(),
                        entities: 0,
                        relations: 0,
                        load_ms: 0,
                        queries: seen + timeouts,
                        matched,
                        mismatched,
                        errored: errored + timeouts,
                        query_ms_total: query_ms_total
                            + timeouts as u64 * query_timeout.as_millis() as u64,
                    });
                }
                eprintln!(
                    "{domain}: [{qid}] exceeded {}s ({timeouts}/{timeout_cap} \
                     timeout budget); skipping and reloading",
                    query_timeout.as_secs()
                );
                // The wedged worker detaches (dies with the process); a
                // fresh worker reloads the graph and resumes past the
                // wedged query and everything already completed.
                skip.insert(qid);
                if let Some(details) = details_out.as_deref() {
                    for entry in details.iter() {
                        skip.insert(entry.qid.clone());
                    }
                }
                receiver = spawn_worker(skip.clone());
            }
        }
    }
}

/// Runs one domain: load, then execute its gold queries, comparing row
/// sets (order-insensitive, stringified) against the pre-verified answer.
#[allow(clippy::too_many_arguments)]
fn run_domain_worker(
    domain: &str,
    graph_path: &Path,
    tasks: &[BenchTask],
    limit: Option<usize>,
    skip: &std::collections::HashSet<String>,
    sink: &mut dyn FnMut(QueryDetail),
    notify_started: &mut dyn FnMut(&str),
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
    let mut record = |entry: QueryDetail| sink(entry);
    let parameters = MutationParameters::new();
    for task in &selected {
        if skip.contains(&task.qid) {
            continue;
        }
        notify_started(&task.qid);
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
                let verdict = if observed == expected {
                    matched += 1;
                    "matched"
                } else {
                    mismatched += 1;
                    "mismatched"
                };
                let sample = |rows: &[Vec<String>]| {
                    let mut text = format!("{rows:?}");
                    text.truncate(220);
                    text
                };
                record(QueryDetail {
                    qid: task.qid.clone(),
                    domain: domain.to_owned(),
                    verdict,
                    duration_ms: query_started.elapsed().as_millis() as u64,
                    observed_rows: observed.len(),
                    expected_rows: expected.len(),
                    observed_sample: if verdict == "mismatched" {
                        sample(&observed)
                    } else {
                        String::new()
                    },
                    expected_sample: if verdict == "mismatched" {
                        sample(&expected)
                    } else {
                        String::new()
                    },
                    error: String::new(),
                });
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
                record(QueryDetail {
                    qid: task.qid.clone(),
                    domain: domain.to_owned(),
                    verdict: "errored",
                    duration_ms: query_started.elapsed().as_millis() as u64,
                    observed_rows: 0,
                    expected_rows: 0,
                    observed_sample: String::new(),
                    expected_sample: String::new(),
                    error: error.to_string().chars().take(160).collect(),
                });
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
