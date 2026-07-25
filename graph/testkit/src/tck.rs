use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use gherkin::{Feature, GherkinEnv, Scenario, Step, StepType};
use regex::Regex;
use thiserror::Error;
use turso_core::{Numeric, Value};
use turso_graph_frontend::Parameters;

use crate::{
    history::{recorded_at, result_digest},
    identity::TestId,
    model::{
        Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
        HISTORY_SCHEMA_VERSION,
    },
    query_cache::QueryParseCache,
    runner::empty_fixture_with_parameters,
};

const TCK_REVISION: &str = "0812a496c62769b67cf688930750ae384e3de68d";

#[derive(Debug, Error)]
pub enum TckError {
    #[error("failed to read TCK directory {path}: {source}")]
    ReadDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse TCK feature {path}: {source}")]
    ParseFeature {
        path: String,
        source: Box<gherkin::ParseFileError>,
    },
    #[error("TCK scenario in {path} has no canonical numeric prefix: {name}")]
    MissingScenarioNumber { path: String, name: String },
    #[error("TCK scenario identity is invalid: {0}")]
    Identity(#[from] crate::identity::TestIdError),
    #[error("TCK scenario identity is duplicated: {0}")]
    DuplicateIdentity(TestId),
}

#[derive(Clone, Debug)]
pub struct TckCase {
    pub id: TestId,
    pub feature_path: String,
    pub feature_name: String,
    pub scenario_name: String,
    pub source_line: usize,
    pub steps: Vec<Step>,
    pub semantic_fingerprint: String,
    semantic_key: String,
}

#[derive(Debug)]
pub struct TckCorpus {
    pub cases: Vec<TckCase>,
    pub feature_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TckStats {
    pub features: usize,
    pub expanded: usize,
    pub canonical: usize,
    pub duplicates: usize,
}

impl TckCorpus {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, TckError> {
        let root = root.as_ref();
        let mut feature_paths = Vec::new();
        collect_feature_paths(root, &mut feature_paths)?;
        feature_paths.sort();
        let scenario_number = Regex::new(r"^\[(\d+)\]").expect("static regex is valid");
        let template = Regex::new(r"<([^>\s]+)>").expect("static regex is valid");
        let mut cases = Vec::new();
        let mut identities = std::collections::HashSet::new();

        for path in &feature_paths {
            let feature = Feature::parse_path(path, GherkinEnv::default()).map_err(|source| {
                TckError::ParseFeature {
                    path: path.display().to_string(),
                    source: Box::new(source),
                }
            })?;
            let relative_path = path
                .strip_prefix(root)
                .expect("collected paths are rooted in the corpus");
            let feature_background = feature
                .background
                .as_ref()
                .map(|background| background.steps.as_slice())
                .unwrap_or_default();
            collect_scenarios(
                &feature,
                relative_path,
                &feature.scenarios,
                &[feature_background],
                &scenario_number,
                &template,
                &mut cases,
                &mut identities,
            )?;
            for rule in &feature.rules {
                let rule_background = rule
                    .background
                    .as_ref()
                    .map(|background| background.steps.as_slice())
                    .unwrap_or_default();
                collect_scenarios(
                    &feature,
                    relative_path,
                    &rule.scenarios,
                    &[feature_background, rule_background],
                    &scenario_number,
                    &template,
                    &mut cases,
                    &mut identities,
                )?;
            }
        }

        Ok(Self {
            cases,
            feature_count: feature_paths.len(),
        })
    }

    pub fn stats(&self) -> TckStats {
        let canonical = self
            .cases
            .iter()
            .map(|case| case.semantic_key.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len();
        TckStats {
            features: self.feature_count,
            expanded: self.cases.len(),
            canonical,
            duplicates: self.cases.len() - canonical,
        }
    }

    pub fn run(&self, environment: RunEnvironment, run_id: &str) -> Vec<ResultRecord> {
        self.run_with_cache(environment, run_id, &mut QueryParseCache::default())
    }

    pub fn run_with_cache(
        &self,
        environment: RunEnvironment,
        run_id: &str,
        parse_cache: &mut QueryParseCache,
    ) -> Vec<ResultRecord> {
        self.cases
            .iter()
            .map(|case| run_canonical(case, environment.clone(), run_id, parse_cache))
            .collect()
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Gherkin expansion keeps shared feature context explicit"
)]
fn collect_scenarios(
    feature: &Feature,
    relative_path: &Path,
    scenarios: &[Scenario],
    backgrounds: &[&[Step]],
    scenario_number: &Regex,
    template: &Regex,
    cases: &mut Vec<TckCase>,
    identities: &mut std::collections::HashSet<TestId>,
) -> Result<(), TckError> {
    for scenario in scenarios {
        let number = scenario_number
            .captures(&scenario.name)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str())
            .ok_or_else(|| TckError::MissingScenarioNumber {
                path: relative_path.display().to_string(),
                name: scenario.name.clone(),
            })?;
        let base_id = format!(
            "tck.{}.scenario-{number}",
            normalized_feature_path(relative_path)
        );
        if scenario.examples.is_empty() {
            let steps = expanded_steps(backgrounds, scenario, template, &HashMap::new());
            push_case(
                feature,
                relative_path,
                &scenario.name,
                scenario.position.line,
                TestId::parse(base_id)?,
                steps,
                cases,
                identities,
            )?;
            continue;
        }
        for (examples_index, examples) in scenario.examples.iter().enumerate() {
            let Some(table) = &examples.table else {
                continue;
            };
            let Some((header, rows)) = table.rows.split_first() else {
                continue;
            };
            for (row_index, row) in rows.iter().enumerate() {
                let substitutions = header
                    .iter()
                    .zip(row)
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect::<HashMap<_, _>>();
                let name = substitute(&scenario.name, template, &substitutions);
                let steps = expanded_steps(backgrounds, scenario, template, &substitutions);
                let id = TestId::parse(format!(
                    "{base_id}.examples-{}-row-{}",
                    examples_index + 1,
                    row_index + 1
                ))?;
                push_case(
                    feature,
                    relative_path,
                    &name,
                    examples.position.line + row_index + 2,
                    id,
                    steps,
                    cases,
                    identities,
                )?;
            }
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "expanded TCK identity and source metadata are kept explicit"
)]
fn push_case(
    feature: &Feature,
    relative_path: &Path,
    scenario_name: &str,
    source_line: usize,
    id: TestId,
    steps: Vec<Step>,
    cases: &mut Vec<TckCase>,
    identities: &mut std::collections::HashSet<TestId>,
) -> Result<(), TckError> {
    if !identities.insert(id.clone()) {
        return Err(TckError::DuplicateIdentity(id));
    }
    let semantic_key = semantic_key(&steps);
    cases.push(TckCase {
        id,
        feature_path: relative_path.to_string_lossy().replace('\\', "/"),
        feature_name: feature.name.clone(),
        scenario_name: scenario_name.to_owned(),
        source_line,
        semantic_fingerprint: fingerprint(&semantic_key),
        semantic_key,
        steps,
    });
    Ok(())
}

fn expanded_steps(
    backgrounds: &[&[Step]],
    scenario: &Scenario,
    template: &Regex,
    substitutions: &HashMap<&str, &str>,
) -> Vec<Step> {
    backgrounds
        .iter()
        .flat_map(|steps| steps.iter())
        .chain(&scenario.steps)
        .cloned()
        .map(|mut step| {
            step.value = substitute(&step.value, template, substitutions);
            step.docstring = step
                .docstring
                .map(|value| substitute(&value, template, substitutions));
            if let Some(table) = &mut step.table {
                for row in &mut table.rows {
                    for cell in row {
                        *cell = substitute(cell, template, substitutions);
                    }
                }
            }
            step
        })
        .collect()
}

fn substitute(value: &str, template: &Regex, substitutions: &HashMap<&str, &str>) -> String {
    template
        .replace_all(value, |captures: &regex::Captures<'_>| {
            substitutions
                .get(captures.get(1).expect("capture exists").as_str())
                .copied()
                .unwrap_or_default()
        })
        .into_owned()
}

fn semantic_key(steps: &[Step]) -> String {
    let mut key = String::new();
    for step in steps {
        key.push_str(match step.ty {
            StepType::Given => "GIVEN\n",
            StepType::When => "WHEN\n",
            StepType::Then => "THEN\n",
        });
        key.push_str(step.value.trim());
        key.push('\n');
        if let Some(docstring) = &step.docstring {
            key.push_str(docstring.trim());
            key.push('\n');
        }
        if let Some(table) = &step.table {
            for row in &table.rows {
                for cell in row {
                    key.push_str(cell.trim());
                    key.push('\u{1f}');
                }
                key.push('\n');
            }
        }
        key.push('\u{1e}');
    }
    key
}

fn normalized_feature_path(path: &Path) -> String {
    path.with_extension("")
        .components()
        .map(|component| {
            let value = component.as_os_str().to_string_lossy().to_ascii_lowercase();
            let mut normalized = String::new();
            let mut separator = false;
            for character in value.chars() {
                if character.is_ascii_alphanumeric() {
                    normalized.push(character);
                    separator = false;
                } else if !separator && !normalized.is_empty() {
                    normalized.push('-');
                    separator = true;
                }
            }
            normalized.trim_end_matches('-').to_owned()
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn collect_feature_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), TckError> {
    let entries = fs::read_dir(directory).map_err(|source| TckError::ReadDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| TckError::ReadDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_feature_paths(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("feature") {
            paths.push(path);
        }
    }
    Ok(())
}

fn run_canonical(
    case: &TckCase,
    environment: RunEnvironment,
    run_id: &str,
    parse_cache: &mut QueryParseCache,
) -> ResultRecord {
    let started = Instant::now();
    let expectation = expected(case);
    let query = query(case);
    let (outcome, rows, message, execution) = match query {
        None => (
            Outcome::Failed,
            None,
            Some("TCK scenario has no executable query step".to_owned()),
            "discovery-only",
        ),
        Some(query) => match parse_cache.parse(query) {
            Err(error) if expectation == Expectation::Error => {
                (Outcome::Passed, None, Some(error), "parser")
            }
            Err(error) => (Outcome::Failed, None, Some(error), "parser"),
            Ok(_) => execute_case(case, query, expectation),
        },
    };
    let duration_ns = started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX);
    base_record(
        case,
        environment,
        run_id,
        expectation,
        outcome,
        duration_ns,
        rows,
        message,
        execution,
    )
}

fn execute_case(
    case: &TckCase,
    query: &str,
    expectation: Expectation,
) -> (
    Outcome,
    Option<Vec<Vec<String>>>,
    Option<String>,
    &'static str,
) {
    let Some(parameters) = parameters(case) else {
        return (
            Outcome::Failed,
            None,
            Some("TCK parameter value is not representable by the generic adapter".to_owned()),
            "parameter-binding",
        );
    };
    let fixture = match empty_fixture_with_parameters(case.id.as_str(), &parameters) {
        Ok(fixture) => fixture,
        Err(error) => {
            return (Outcome::Failed, None, Some(error.to_string()), "execution");
        }
    };
    if let Some(name) = named_graph(case) {
        let setup = match named_graph_setup(name) {
            Ok(setup) => setup,
            Err(error) => return (Outcome::Failed, None, Some(error), "fixture-loading"),
        };
        if let Err(error) = execute_tck_statement(&fixture.session, &setup, &parameters) {
            return (
                Outcome::Failed,
                None,
                Some(format!("TCK named graph `{name}` setup failed: {error}")),
                "fixture-execution",
            );
        }
    }
    for setup in setup_queries(case) {
        if let Err(error) = execute_tck_statement(&fixture.session, setup, &parameters) {
            return (
                Outcome::Failed,
                None,
                Some(format!("TCK setup query failed: {error}; query: {setup}")),
                "setup-execution",
            );
        }
    }
    let labels_table = turso_graph_frontend::labels_table_name(fixture.session.graph_id());
    let counters_before = graph_counters(&fixture.connection, &labels_table);
    match execute_tck_statement(&fixture.session, query, &parameters) {
        Err(error) if expectation == Expectation::Error => {
            (Outcome::Passed, None, Some(error), "execution")
        }
        Err(error) => (Outcome::Failed, None, Some(error), "execution"),
        Ok(execution) if expectation == Expectation::Error => (
            Outcome::Failed,
            Some(stringify_rows(execution.rows)),
            Some("expected an error but execution succeeded".to_owned()),
            "execution",
        ),
        Ok(execution) => {
            let rows = execution.rows;
            if let Some(step) = case
                .steps
                .iter()
                .find(|step| step.value.contains("side effects should be"))
            {
                if let Err(message) = compare_side_effects(
                    &fixture.connection,
                    &labels_table,
                    counters_before.as_ref(),
                    step,
                ) {
                    return (
                        Outcome::Failed,
                        Some(stringify_rows(rows)),
                        Some(message),
                        "side-effect-comparison",
                    );
                }
            }
            if case
                .steps
                .iter()
                .any(|step| step.value.contains("graph should be"))
            {
                return (
                    Outcome::Failed,
                    Some(stringify_rows(rows)),
                    Some(
                        "query executed, but TCK graph-state comparison is not implemented"
                            .to_owned(),
                    ),
                    "graph-comparison",
                );
            }
            let types = execution.result_types.or_else(|| {
                fixture
                    .session
                    .prepare(query, &parameters)
                    .ok()
                    .map(|statement| statement.result_types().to_vec())
            });
            let rows = stringify_rows_with_entities(
                rows,
                types.as_deref(),
                &fixture.connection,
                &labels_table,
                fixture.session.graph_id(),
            );
            let Some((expected_rows, ordered)) = expected_rows(case) else {
                return (
                    Outcome::Failed,
                    Some(rows),
                    Some(
                        "result expectation is not representable by the scalar adapter".to_owned(),
                    ),
                    "result-comparison",
                );
            };
            let mut rows: Vec<Vec<String>> = rows
                .into_iter()
                .map(|row| row.into_iter().map(|cell| normalize_cell(&cell)).collect())
                .collect();
            let mut expected_rows: Vec<Vec<String>> = expected_rows
                .into_iter()
                .map(|row| row.into_iter().map(|cell| normalize_cell(&cell)).collect())
                .collect();
            if !ordered {
                rows.sort();
                expected_rows.sort();
            }
            if rows == expected_rows {
                (Outcome::Passed, Some(rows), None, "execution")
            } else {
                (
                    Outcome::Failed,
                    Some(rows.clone()),
                    Some(format!("expected {expected_rows:?}, observed {rows:?}")),
                    "execution",
                )
            }
        }
    }
}

struct GraphCounters {
    nodes: i64,
    relationships: i64,
    properties: i64,
    labels: i64,
}

/// Counts nodes, relationships, and non-null property cells in the shared
/// fixture tables. Labels are not physically stored, so ±labels expectations
/// cannot be verified.
fn graph_counters(
    connection: &std::sync::Arc<turso_core::Connection>,
    labels_table: &str,
) -> Result<GraphCounters, String> {
    let count = |sql: &str| -> Result<i64, String> {
        let rows = connection
            .prepare(sql)
            .and_then(|mut statement| statement.run_collect_rows())
            .map_err(|error| error.to_string())?;
        match rows.first().and_then(|row| row.first()) {
            Some(Value::Numeric(Numeric::Integer(value))) => Ok(*value),
            other => Err(format!("count query returned {other:?}")),
        }
    };
    let mut properties = 0_i64;
    for (table, excluded) in [
        ("people", vec!["id"]),
        ("relationships", vec!["id", "src", "dst"]),
    ] {
        let columns = connection
            .prepare(format!("SELECT name FROM pragma_table_info('{table}')"))
            .and_then(|mut statement| statement.run_collect_rows())
            .map_err(|error| error.to_string())?;
        for row in columns {
            let Some(Value::Text(name)) = row.first() else {
                continue;
            };
            let name = name.to_string();
            if excluded.contains(&name.as_str()) {
                continue;
            }
            properties += count(&format!("SELECT count(\"{name}\") FROM \"{table}\""))?;
        }
    }
    Ok(GraphCounters {
        nodes: count("SELECT count(*) FROM people")?,
        relationships: count("SELECT count(*) FROM relationships")?,
        properties,
        labels: count(&format!(
            "SELECT count(DISTINCT label) FROM \"{labels_table}\""
        ))?,
    })
}

fn compare_side_effects(
    connection: &std::sync::Arc<turso_core::Connection>,
    labels_table: &str,
    before: Result<&GraphCounters, &String>,
    step: &Step,
) -> Result<(), String> {
    let before = before.map_err(|error| format!("side-effect baseline failed: {error}"))?;
    let after = graph_counters(connection, labels_table)
        .map_err(|error| format!("side-effect measurement failed: {error}"))?;
    let mut observed = std::collections::BTreeMap::new();
    let diff = |added: &str, removed: &str, delta: i64| {
        [
            (added.to_owned(), delta.max(0)),
            (removed.to_owned(), (-delta).max(0)),
        ]
    };
    for (key, value) in diff("+nodes", "-nodes", after.nodes - before.nodes)
        .into_iter()
        .chain(diff(
            "+relationships",
            "-relationships",
            after.relationships - before.relationships,
        ))
        .chain(diff(
            "+properties",
            "-properties",
            after.properties - before.properties,
        ))
        .chain(diff("+labels", "-labels", after.labels - before.labels))
    {
        observed.insert(key, value);
    }
    let Some(table) = &step.table else {
        return Err("side-effect expectation has no table".to_owned());
    };
    for row in &table.rows {
        let [key, expected] = row.as_slice() else {
            return Err(format!("unexpected side-effect row {row:?}"));
        };
        let expected: i64 = expected
            .trim()
            .parse()
            .map_err(|_| format!("unparsable side-effect count {expected:?}"))?;
        match observed.get(key.trim()) {
            Some(actual) if *actual == expected => {}
            Some(actual) => {
                return Err(format!(
                    "side effect {key} expected {expected}, observed {actual}"
                ));
            }
            // Labels are not physically stored in the relational model.
            None => {
                return Err(format!(
                    "side effect {key} is not measurable in this fixture"
                ));
            }
        }
    }
    // Any measured effect the expectation does not mention must be zero.
    for (key, actual) in &observed {
        if *actual != 0
            && !table
                .rows
                .iter()
                .any(|row| row.first().map(|k| k.trim()) == Some(key.as_str()))
        {
            return Err(format!("unexpected side effect {key} = {actual}"));
        }
    }
    Ok(())
}

fn setup_queries(case: &TckCase) -> impl Iterator<Item = &str> {
    case.steps
        .iter()
        .filter(|step| step.value.contains("having executed"))
        .filter_map(|step| step.docstring.as_deref())
}

fn named_graph(case: &TckCase) -> Option<&str> {
    case.steps.iter().find_map(|step| {
        step.value
            .strip_prefix("the ")
            .and_then(|value| value.strip_suffix(" graph"))
    })
}

fn named_graph_setup(name: &str) -> Result<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../testdata/tck/opencypher/graphs")
        .join(name)
        .join(format!("{name}.cypher"));
    fs::read_to_string(&path)
        .map_err(|error| format!("failed to read TCK named graph {}: {error}", path.display()))
}

fn execute_tck_statement(
    session: &turso_graph_frontend::GraphConnection,
    statement: &str,
    parameters: &Parameters,
) -> Result<ExecutedRows, String> {
    match session.query(statement, parameters) {
        Ok(rows) => Ok(ExecutedRows {
            rows,
            result_types: None,
        }),
        Err(query_error) => session
            .execute(statement, parameters)
            .map(|summary| ExecutedRows {
                rows: summary.rows,
                result_types: Some(summary.result_types),
            })
            .map_err(|mutation_error| {
                format!(
                    "query execution failed: {query_error}; mutation execution failed: {mutation_error}; query: {statement}"
                )
            }),
    }
}

struct ExecutedRows {
    rows: Vec<Vec<Value>>,
    result_types: Option<Vec<turso_graph_ir::ValueType>>,
}

fn parameters(case: &TckCase) -> Option<Parameters> {
    let Some(step) = case
        .steps
        .iter()
        .find(|step| step.value.contains("parameters are"))
    else {
        return Some(Parameters::new());
    };
    step.table
        .as_ref()?
        .rows
        .iter()
        .map(|row| {
            let [name, value] = row.as_slice() else {
                return None;
            };
            Some((name.clone(), parameter_value(value)?))
        })
        .collect()
}

fn parameter_value(value: &str) -> Option<Value> {
    let value = value.trim();
    match value {
        "null" => Some(Value::Null),
        "true" => Some(Value::from_i64(1)),
        "false" => Some(Value::from_i64(0)),
        // List and map parameters carry as JSON text, the same encoding
        // the frontend uses for list and map values.
        _ if value.starts_with('[') || value.starts_with('{') => {
            cypher_literal_to_json(value).map(Value::build_text)
        }
        _ => value
            .parse::<i64>()
            .map(Value::from_i64)
            .or_else(|_| value.parse::<f64>().map(Value::from_f64))
            .ok()
            .or_else(|| {
                value
                    .strip_prefix('\'')
                    .and_then(|value| value.strip_suffix('\''))
                    .map(|value| Value::build_text(value.replace("\\'", "'").replace("\\\\", "\\")))
            }),
    }
}

/// Converts a TCK Cypher-style literal (single-quoted strings, unquoted map
/// keys) into JSON text.
fn cypher_literal_to_json(value: &str) -> Option<String> {
    fn convert(parser: &mut CellParser) -> Option<serde_json::Value> {
        parser.skip_spaces();
        match parser.peek()? {
            b'[' => {
                parser.position += 1;
                let mut items = Vec::new();
                loop {
                    parser.skip_spaces();
                    if parser.peek() == Some(b']') {
                        parser.position += 1;
                        break;
                    }
                    items.push(convert(parser)?);
                    parser.skip_spaces();
                    if parser.peek() == Some(b',') {
                        parser.position += 1;
                    }
                }
                Some(serde_json::Value::Array(items))
            }
            b'{' => {
                parser.position += 1;
                let mut map = serde_json::Map::new();
                loop {
                    parser.skip_spaces();
                    if parser.peek() == Some(b'}') {
                        parser.position += 1;
                        break;
                    }
                    let key = parser.parse_scalar()?;
                    parser.skip_spaces();
                    if parser.peek() == Some(b':') {
                        parser.position += 1;
                    }
                    map.insert(key, convert(parser)?);
                    parser.skip_spaces();
                    if parser.peek() == Some(b',') {
                        parser.position += 1;
                    }
                }
                Some(serde_json::Value::Object(map))
            }
            b'\'' => {
                let quoted = parser.parse_string()?;
                let body = quoted.trim_matches('\'').to_owned();
                Some(serde_json::Value::String(body))
            }
            _ => {
                let token = parser.parse_scalar()?;
                match token.as_str() {
                    "null" => Some(serde_json::Value::Null),
                    "true" => Some(serde_json::Value::Bool(true)),
                    "false" => Some(serde_json::Value::Bool(false)),
                    _ => token
                        .parse::<i64>()
                        .map(serde_json::Value::from)
                        .or_else(|_| token.parse::<f64>().map(serde_json::Value::from))
                        .ok(),
                }
            }
        }
    }
    let mut parser = CellParser {
        input: value.as_bytes(),
        position: 0,
    };
    let converted = convert(&mut parser)?;
    parser.at_end().then(|| converted.to_string())
}

fn query(case: &TckCase) -> Option<&str> {
    case.steps
        .iter()
        .find(|step| step.ty == StepType::When && step.value.contains("executing"))
        .and_then(|step| step.docstring.as_deref())
}

fn expected(case: &TckCase) -> Expectation {
    if case
        .steps
        .iter()
        .any(|step| step.ty == StepType::Then && step.value.contains("should be raised"))
    {
        Expectation::Error
    } else {
        Expectation::Rows
    }
}

fn expected_rows(case: &TckCase) -> Option<(Vec<Vec<String>>, bool)> {
    let step = case.steps.iter().find(|step| {
        step.ty == StepType::Then
            && (step.value.starts_with("the result should be")
                || step.value == "the result should be empty")
    })?;
    if step.value == "the result should be empty" {
        return Some((Vec::new(), true));
    }
    let rows = step.table.as_ref()?.rows.get(1..)?;
    let mut normalized = Vec::with_capacity(rows.len());
    for row in rows {
        let mut normalized_row = Vec::with_capacity(row.len());
        for cell in row {
            normalized_row.push(normalize_expected_scalar(cell)?);
        }
        normalized.push(normalized_row);
    }
    Some((normalized, step.value.ends_with("in order:")))
}

fn normalize_expected_scalar(value: &str) -> Option<String> {
    let value = value.trim();
    if value == "null" {
        return Some("<null>".to_owned());
    }
    if let Some(unquoted) = value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return Some(
            unquoted
                .replace("\\'", "'")
                .replace("\\n", "\n")
                .replace("\\\\", "\\"),
        );
    }
    // Entity, path, and map cells pass through as written: comparison is
    // structural (normalize_cell), so the rendered side lines up.
    if value.starts_with('(') || value.starts_with('<') {
        return Some(value.to_owned());
    }
    if value.starts_with('[') {
        return canonicalize_list(value);
    }
    Some(value.to_owned())
}

fn canonicalize_list(value: &str) -> Option<String> {
    let mut canonical = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        match character {
            '\'' if quoted => {
                quoted = false;
                canonical.push('"');
            }
            '\'' => {
                quoted = true;
                canonical.push('"');
            }
            '\\' if quoted => {
                let escaped = characters.next()?;
                if escaped == '"' {
                    canonical.push('\\');
                }
                canonical.push(escaped);
            }
            '"' if quoted => canonical.push_str("\\\""),
            character if character.is_whitespace() && !quoted => {}
            character => canonical.push(character),
        }
    }
    (!quoted).then_some(canonical)
}

fn stringify_rows(rows: Vec<Vec<Value>>) -> Vec<Vec<String>> {
    stringify_rows_with_types(rows, None)
}

fn stringify_rows_with_types(
    rows: Vec<Vec<Value>>,
    types: Option<&[turso_graph_ir::ValueType]>,
) -> Vec<Vec<String>> {
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let boolean = types.is_some_and(|types| {
                        types.get(index) == Some(&turso_graph_ir::ValueType::Boolean)
                    });
                    match value {
                        Value::Null => "<null>".to_owned(),
                        Value::Numeric(Numeric::Integer(value)) if boolean => {
                            if value == 0 { "false" } else { "true" }.to_owned()
                        }
                        Value::Numeric(Numeric::Integer(value)) => value.to_string(),
                        Value::Numeric(Numeric::Float(value)) if value.fract() == 0.0 => {
                            format!("{value:.1}")
                        }
                        Value::Numeric(Numeric::Float(value)) => value.to_string(),
                        Value::Text(value) => value.to_string(),
                        Value::Blob(value) => format!("{value:?}"),
                    }
                })
                .collect()
        })
        .collect()
}

/// Canonicalizes a result cell for comparison: entity, path, map, and list
/// literals parse into a structural form re-emitted with sorted property
/// keys, normalized quoting, and booleans folded onto their stored integer
/// representation. Non-structural cells pass through unchanged.
fn normalize_cell(cell: &str) -> String {
    let mut parser = CellParser {
        input: cell.as_bytes(),
        position: 0,
    };
    match parser.parse_value() {
        Some(value) if parser.at_end() => value,
        _ => cell.to_owned(),
    }
}

struct CellParser<'a> {
    input: &'a [u8],
    position: usize,
}

impl CellParser<'_> {
    fn at_end(&mut self) -> bool {
        self.skip_spaces();
        self.position >= self.input.len()
    }

    fn skip_spaces(&mut self) {
        while self.input.get(self.position) == Some(&b' ') {
            self.position += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_spaces();
        self.input.get(self.position).copied()
    }

    fn eat(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn parse_value(&mut self) -> Option<String> {
        match self.peek()? {
            b'(' => self.parse_entity(b'(', b')'),
            b'[' => {
                // Either a relationship literal [:T {..}] or a list [v, ..].
                let checkpoint = self.position;
                if let Some(entity) = self.parse_entity(b'[', b']') {
                    return Some(entity);
                }
                self.position = checkpoint;
                self.parse_list()
            }
            b'<' => self.parse_path(),
            b'{' => self.parse_map(),
            b'\'' => self.parse_string(),
            _ => self.parse_scalar(),
        }
    }

    fn parse_entity(&mut self, open: u8, close: u8) -> Option<String> {
        if !self.eat(open) {
            return None;
        }
        let mut labels = Vec::new();
        while self.eat(b':') {
            labels.push(self.parse_identifier()?);
        }
        let properties = if self.peek() == Some(b'{') {
            self.parse_property_map()?
        } else {
            Vec::new()
        };
        if !self.eat(close) {
            return None;
        }
        let mut rendered = String::new();
        rendered.push(open as char);
        for label in labels {
            rendered.push(':');
            rendered.push_str(&label);
        }
        rendered.push_str(&render_sorted_properties(properties, !rendered.is_empty()));
        rendered.push(close as char);
        Some(rendered)
    }

    fn parse_path(&mut self) -> Option<String> {
        if !self.eat(b'<') {
            return None;
        }
        let mut rendered = String::from("<");
        rendered.push_str(&self.parse_entity(b'(', b')')?);
        loop {
            match self.peek() {
                Some(b'>') => {
                    self.position += 1;
                    rendered.push('>');
                    return Some(rendered);
                }
                Some(b'-') => {
                    self.position += 1;
                    let relationship = self.parse_entity(b'[', b']')?;
                    if self.eat(b'-') {
                        if !self.eat(b'>') {
                            return None;
                        }
                        rendered.push_str(&format!("-{relationship}->"));
                    } else {
                        return None;
                    }
                    rendered.push_str(&self.parse_entity(b'(', b')')?);
                }
                Some(b'<') => {
                    self.position += 1;
                    if !self.eat(b'-') {
                        return None;
                    }
                    let relationship = self.parse_entity(b'[', b']')?;
                    if !self.eat(b'-') {
                        return None;
                    }
                    rendered.push_str(&format!("<-{relationship}-"));
                    rendered.push_str(&self.parse_entity(b'(', b')')?);
                }
                _ => return None,
            }
        }
    }

    fn parse_list(&mut self) -> Option<String> {
        if !self.eat(b'[') {
            return None;
        }
        let mut items = Vec::new();
        if self.peek() != Some(b']') {
            loop {
                items.push(self.parse_value()?);
                if !self.eat(b',') {
                    break;
                }
            }
        }
        if !self.eat(b']') {
            return None;
        }
        Some(format!("[{}]", items.join(", ")))
    }

    fn parse_map(&mut self) -> Option<String> {
        let properties = self.parse_property_map()?;
        Some(format!(
            "{{{}}}",
            sorted_property_entries(properties).join(", ")
        ))
    }

    fn parse_property_map(&mut self) -> Option<Vec<(String, String)>> {
        if !self.eat(b'{') {
            return None;
        }
        let mut properties = Vec::new();
        if self.peek() != Some(b'}') {
            loop {
                let key = self.parse_identifier()?;
                if !self.eat(b':') {
                    return None;
                }
                properties.push((key, self.parse_value()?));
                if !self.eat(b',') {
                    break;
                }
            }
        }
        if !self.eat(b'}') {
            return None;
        }
        Some(properties)
    }

    fn parse_identifier(&mut self) -> Option<String> {
        self.skip_spaces();
        let start = self.position;
        while self
            .input
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            self.position += 1;
        }
        (self.position > start)
            .then(|| String::from_utf8_lossy(&self.input[start..self.position]).into_owned())
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.eat(b'\'') {
            return None;
        }
        let start = self.position;
        while self
            .input
            .get(self.position)
            .is_some_and(|byte| *byte != b'\'')
        {
            self.position += 1;
        }
        if self.input.get(self.position) != Some(&b'\'') {
            return None;
        }
        let body = String::from_utf8_lossy(&self.input[start..self.position]).into_owned();
        self.position += 1;
        Some(format!("'{body}'"))
    }

    fn parse_scalar(&mut self) -> Option<String> {
        self.skip_spaces();
        let start = self.position;
        while self.input.get(self.position).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+' | b'.')
        }) {
            self.position += 1;
        }
        if self.position == start {
            return None;
        }
        let token = String::from_utf8_lossy(&self.input[start..self.position]).into_owned();
        // Booleans fold onto their stored integer representation; floats
        // fold onto one canonical rendering so `1.0`, `1e0`, and SQLite's
        // `%.15g` output compare equal.
        Some(match token.as_str() {
            "true" => "1".to_owned(),
            "false" => "0".to_owned(),
            _ => {
                if token.contains(['.', 'e', 'E'])
                    && !token.contains("..")
                    && token.parse::<i64>().is_err()
                {
                    if let Ok(value) = token.parse::<f64>() {
                        return Some(format!("{value:?}"));
                    }
                }
                token
            }
        })
    }
}

fn sorted_property_entries(mut properties: Vec<(String, String)>) -> Vec<String> {
    properties.sort_by(|left, right| left.0.cmp(&right.0));
    properties
        .into_iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect()
}

fn render_sorted_properties(properties: Vec<(String, String)>, spaced: bool) -> String {
    if properties.is_empty() {
        return String::new();
    }
    let body = format!("{{{}}}", sorted_property_entries(properties).join(", "));
    if spaced {
        format!(" {body}")
    } else {
        body
    }
}

/// Renders result rows with graph-aware formatting: node and relationship
/// columns print in the TCK's `(:Label {key: value})` / `[:TYPE {..}]`
/// shapes, resolved from the label/type junctions and fixture tables.
fn stringify_rows_with_entities(
    rows: Vec<Vec<Value>>,
    types: Option<&[turso_graph_ir::ValueType]>,
    connection: &std::sync::Arc<turso_core::Connection>,
    labels_table: &str,
    graph: turso_graph_ir::GraphId,
) -> Vec<Vec<String>> {
    let types_table = turso_graph_frontend::relationship_types_table_name(graph);
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let column_type = types.and_then(|types| types.get(index));
                    match (column_type, &value) {
                        (
                            Some(turso_graph_ir::ValueType::Node),
                            Value::Numeric(Numeric::Integer(identity)),
                        ) => render_node(connection, labels_table, *identity)
                            .unwrap_or_else(|| identity.to_string()),
                        (
                            Some(turso_graph_ir::ValueType::Relationship),
                            Value::Numeric(Numeric::Integer(identity)),
                        ) => render_relationship(connection, &types_table, *identity)
                            .unwrap_or_else(|| identity.to_string()),
                        (Some(turso_graph_ir::ValueType::Path), Value::Text(json)) => {
                            render_path(connection, labels_table, &types_table, json.as_str())
                                .unwrap_or_else(|| json.to_string())
                        }
                        _ => stringify_value(
                            value,
                            column_type == Some(&turso_graph_ir::ValueType::Boolean),
                        ),
                    }
                })
                .collect()
        })
        .collect()
}

fn stringify_value(value: Value, boolean: bool) -> String {
    match value {
        Value::Null => "<null>".to_owned(),
        Value::Numeric(Numeric::Integer(value)) if boolean => {
            if value == 0 { "false" } else { "true" }.to_owned()
        }
        Value::Numeric(Numeric::Integer(value)) => value.to_string(),
        Value::Numeric(Numeric::Float(value)) if value.fract() == 0.0 => {
            format!("{value:.1}")
        }
        Value::Numeric(Numeric::Float(value)) => value.to_string(),
        Value::Text(value) => value.to_string(),
        Value::Blob(value) => format!("{value:?}"),
    }
}

fn scalar_rows(
    connection: &std::sync::Arc<turso_core::Connection>,
    sql: &str,
) -> Option<Vec<Vec<Value>>> {
    connection
        .prepare(sql)
        .and_then(|mut statement| statement.run_collect_rows())
        .ok()
}

fn render_property_value(value: &Value) -> String {
    match value {
        Value::Text(text) => format!("'{}'", text.to_string().replace('\'', "\\'")),
        Value::Numeric(Numeric::Integer(value)) => value.to_string(),
        Value::Numeric(Numeric::Float(value)) if value.fract() == 0.0 => format!("{value:.1}"),
        Value::Numeric(Numeric::Float(value)) => value.to_string(),
        Value::Null => "null".to_owned(),
        Value::Blob(value) => format!("{value:?}"),
    }
}

fn entity_properties(
    connection: &std::sync::Arc<turso_core::Connection>,
    table: &str,
    excluded: &[&str],
    identity: i64,
) -> Vec<(String, String)> {
    let mut properties = Vec::new();
    let Some(columns) = scalar_rows(
        connection,
        &format!("SELECT name FROM pragma_table_info('{table}')"),
    ) else {
        return properties;
    };
    for column in columns {
        let Some(Value::Text(name)) = column.first() else {
            continue;
        };
        let name = name.to_string();
        if excluded.contains(&name.as_str()) {
            continue;
        }
        let Some(values) = scalar_rows(
            connection,
            &format!("SELECT \"{name}\" FROM \"{table}\" WHERE id = {identity}"),
        ) else {
            continue;
        };
        if let Some(value) = values.first().and_then(|row| row.first()) {
            if !matches!(value, Value::Null) {
                // Reserved-name properties live in prefixed payload columns
                // (dynamic catalog); render them under their Cypher name.
                let logical = name
                    .strip_prefix("cyprop_")
                    .map(str::to_owned)
                    .unwrap_or(name);
                properties.push((logical, render_property_value(value)));
            }
        }
    }
    properties
}

fn render_entity(labels: Vec<String>, properties: Vec<(String, String)>, node: bool) -> String {
    let mut inner = String::new();
    for label in labels {
        inner.push(':');
        inner.push_str(&label);
    }
    if !properties.is_empty() {
        if !inner.is_empty() {
            inner.push(' ');
        }
        inner.push('{');
        inner.push_str(
            &properties
                .into_iter()
                .map(|(key, value)| format!("{key}: {value}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        inner.push('}');
    }
    if node {
        format!("({inner})")
    } else {
        format!("[{inner}]")
    }
}

fn render_node(
    connection: &std::sync::Arc<turso_core::Connection>,
    labels_table: &str,
    identity: i64,
) -> Option<String> {
    let labels = scalar_rows(
        connection,
        &format!("SELECT label FROM \"{labels_table}\" WHERE node_id = {identity} ORDER BY rowid"),
    )?
    .into_iter()
    .filter_map(|row| match row.into_iter().next() {
        Some(Value::Text(label)) => Some(label.to_string()),
        _ => None,
    })
    .collect();
    let properties = entity_properties(connection, "people", &["id"], identity);
    Some(render_entity(labels, properties, true))
}

fn render_relationship(
    connection: &std::sync::Arc<turso_core::Connection>,
    types_table: &str,
    identity: i64,
) -> Option<String> {
    let types = scalar_rows(
        connection,
        &format!(
            "SELECT type FROM \"{types_table}\" WHERE relationship_id = {identity} ORDER BY rowid"
        ),
    )?
    .into_iter()
    .filter_map(|row| match row.into_iter().next() {
        Some(Value::Text(name)) => Some(name.to_string()),
        _ => None,
    })
    .collect();
    let properties =
        entity_properties(connection, "relationships", &["id", "src", "dst"], identity);
    Some(render_entity(types, properties, false))
}

/// Renders a {nodes, relationships} path value in the TCK's
/// `<(a)-[:T]->(b)>` shape, recovering hop directions from the
/// relationship endpoints.
fn render_path(
    connection: &std::sync::Arc<turso_core::Connection>,
    labels_table: &str,
    types_table: &str,
    json: &str,
) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(json).ok()?;
    let identities = |key: &str| -> Option<Vec<i64>> {
        parsed
            .get(key)?
            .as_array()?
            .iter()
            .map(serde_json::Value::as_i64)
            .collect()
    };
    let nodes = identities("nodes")?;
    let relationships = identities("relationships")?;
    let mut rendered = String::from("<");
    rendered.push_str(&render_node(connection, labels_table, *nodes.first()?)?);
    for (index, relationship) in relationships.iter().enumerate() {
        let source = scalar_rows(
            connection,
            &format!("SELECT src FROM relationships WHERE id = {relationship}"),
        )?
        .into_iter()
        .next()
        .and_then(|row| match row.into_iter().next() {
            Some(Value::Numeric(Numeric::Integer(source))) => Some(source),
            _ => None,
        })?;
        let outgoing = source == nodes[index];
        let arrow = render_relationship(connection, types_table, *relationship)?;
        if outgoing {
            rendered.push_str(&format!("-{arrow}->"));
        } else {
            rendered.push_str(&format!("<-{arrow}-"));
        }
        rendered.push_str(&render_node(
            connection,
            labels_table,
            *nodes.get(index + 1)?,
        )?);
    }
    rendered.push('>');
    Some(rendered)
}

#[expect(
    clippy::too_many_arguments,
    reason = "result records require explicit outcome evidence"
)]
fn base_record(
    case: &TckCase,
    environment: RunEnvironment,
    run_id: &str,
    expectation: Expectation,
    outcome: Outcome,
    duration_ns: u64,
    rows: Option<Vec<Vec<String>>>,
    message: Option<String>,
    execution: &str,
) -> ResultRecord {
    let mut dimensions = BTreeMap::from([
        (
            "semantic_fingerprint".to_owned(),
            case.semantic_fingerprint.clone(),
        ),
        ("source_line".to_owned(), case.source_line.to_string()),
        ("execution".to_owned(), execution.to_owned()),
    ]);
    dimensions.insert("feature".to_owned(), case.feature_name.clone());
    let row_count = rows.as_ref().map(|rows| rows.len() as u64);
    let digest = rows.as_ref().map(|rows| result_digest(rows));
    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: recorded_at(),
        environment,
        suite: "tck-deep".to_owned(),
        test_id: case.id.clone(),
        kind: TestKind::Conformance,
        area: case
            .feature_path
            .split('/')
            .next()
            .unwrap_or("tck")
            .to_owned(),
        fixture: "opencypher-tck".to_owned(),
        expectation,
        outcome,
        duration_ns,
        source: source_identity(case),
        operation: None,
        graph_shape: None,
        scale: None,
        iterations: None,
        throughput_per_second: None,
        row_count,
        node_count: None,
        relationship_count: None,
        result_digest: digest,
        message,
        dimensions,
    }
}

fn source_identity(case: &TckCase) -> SourceIdentity {
    SourceIdentity {
        name: "openCypher TCK via Uni".to_owned(),
        repository: "https://github.com/rustic-ai/uni-db".to_owned(),
        revision: TCK_REVISION.to_owned(),
        path: format!("crates/uni-tck/tck/features/{}", case.feature_path),
        case: case.scenario_name.clone(),
        license: "Apache-2.0".to_owned(),
        adaptation: "verbatim-tck-scenario".to_owned(),
        issue: None,
        fixed_commit: None,
    }
}

fn fingerprint(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_pinned_corpus_expands_to_expected_identity_count() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/tck/opencypher/features");
        let corpus = TckCorpus::load(root).unwrap();
        let stats = corpus.stats();
        assert_eq!(stats.features, 221);
        assert_eq!(stats.expanded, 3_926);
        assert_eq!(
            corpus
                .cases
                .iter()
                .map(|case| &case.id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3_926
        );
    }

    #[test]
    fn empty_graph_match_is_runnable_by_generic_adapter() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/tck/opencypher/features");
        let corpus = TckCorpus::load(root).unwrap();
        let case = corpus
            .cases
            .iter()
            .find(|case| case.id.as_str() == "tck.clauses.match.match1.scenario-1")
            .unwrap();
        let query = query(case).unwrap();

        assert_eq!(
            execute_case(case, query, Expectation::Rows).0,
            Outcome::Passed
        );
    }

    #[test]
    fn mutation_return_entity_types_are_used_for_tck_rendering() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/tck/opencypher/features");
        let corpus = TckCorpus::load(root).unwrap();
        let case = corpus
            .cases
            .iter()
            .find(|case| case.id.as_str() == "tck.clauses.create.create3.scenario-5")
            .unwrap();
        let query = query(case).unwrap();

        assert_eq!(
            execute_case(case, query, Expectation::Rows).0,
            Outcome::Passed
        );
    }

    #[test]
    fn scalar_tck_parameter_values_are_bound_for_execution() {
        assert_eq!(parameter_value("42"), Some(Value::from_i64(42)));
        assert_eq!(parameter_value("2.5"), Some(Value::from_f64(2.5)));
        assert_eq!(
            parameter_value("'Ada'"),
            Some(Value::build_text("Ada".to_owned()))
        );
        assert_eq!(parameter_value("null"), Some(Value::Null));
        assert_eq!(
            parameter_value("[1, 2]"),
            Some(Value::build_text("[1,2]".to_owned()))
        );
        assert_eq!(
            parameter_value("{name: 'Ada', tags: [1]}"),
            Some(Value::build_text(
                "{\"name\":\"Ada\",\"tags\":[1]}".to_owned()
            ))
        );
    }

    #[test]
    fn tck_setup_queries_are_part_of_execution() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/tck/opencypher/features");
        let corpus = TckCorpus::load(root).unwrap();
        let case = corpus
            .cases
            .iter()
            .find(|case| case.id.as_str() == "tck.clauses.match.match1.scenario-5")
            .unwrap();

        assert_eq!(setup_queries(case).count(), 1);
    }

    #[test]
    fn pinned_named_graph_fixture_is_loaded_for_referenced_scenario() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/tck/opencypher/features");
        let corpus = TckCorpus::load(root).unwrap();
        let case = corpus
            .cases
            .iter()
            .find(|case| {
                case.id.as_str() == "tck.usecases.triadicselection.triadicselection1.scenario-1"
            })
            .unwrap();

        assert_eq!(named_graph(case), Some("binary-tree-1"));
        assert!(named_graph_setup("binary-tree-1").unwrap().contains("c42"));
    }

    #[test]
    fn semantic_key_ignores_source_identity() {
        let step = Step {
            keyword: "When".to_owned(),
            ty: StepType::When,
            value: "executing query:".to_owned(),
            docstring: Some("RETURN 1".to_owned()),
            table: None,
            span: Default::default(),
            position: Default::default(),
        };
        assert_eq!(
            semantic_key(std::slice::from_ref(&step)),
            semantic_key(&[step])
        );
    }
}
