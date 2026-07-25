use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Deserialize;
use serde_yaml::Value as YamlValue;
use thiserror::Error;
use turso_core::{Numeric, Value};
use turso_graph_frontend::{GraphConnection, Parameters};

use crate::{
    history::{recorded_at, result_digest},
    identity::TestId,
    model::{
        Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
        HISTORY_SCHEMA_VERSION,
    },
    query_cache::QueryParseCache,
    runner::empty_fixture,
};

const GRAFEO_REVISION: &str = "4ebae02f06f8f0cbc57543f74b6ba06f259dbed3";

/// Scenarios that exercise Neo4j administrative commands rather than
/// openCypher or GQL: index DDL (`CREATE`/`DROP INDEX`, `SHOW INDEXES`),
/// constraint DDL (`CREATE`/`DROP CONSTRAINT`, `SHOW CONSTRAINTS`), and the
/// `PROFILE` command. None of these appear in the openCypher or GQL
/// grammars, so they stay out of the conformance corpus.
const NON_GQL_CASES: &[&str] = &[
    // spec/common/index_correctness.gtest: CREATE INDEX is the only
    // supported index DDL exercised by this file (see its header comment).
    "grafeo.spec.common.index.correctness.create.index.then.query",
    "grafeo.spec.common.index.correctness.index.query.no.match",
    "grafeo.spec.common.index.correctness.index.multiple.matches",
    "grafeo.spec.common.index.correctness.index.with.null.property",
    "grafeo.spec.common.index.correctness.index.after.property.update",
    "grafeo.spec.common.index.correctness.index.old.value.gone.after.update",
    "grafeo.spec.common.index.correctness.index.after.delete",
    "grafeo.spec.common.index.correctness.index.remaining.after.delete",
    "grafeo.spec.common.index.correctness.index.reinsert.after.delete",
    "grafeo.spec.common.index.correctness.numeric.index.exact.lookup",
    "grafeo.spec.common.index.correctness.numeric.index.range.query",
    "grafeo.spec.common.index.correctness.bulk.insert.then.index",
    "grafeo.spec.common.index.correctness.index.count.all",
    "grafeo.spec.common.index.correctness.drop.index.query.still.works",
    // spec/lpg/cypher/admin.gtest: CREATE/DROP INDEX, SHOW INDEXES, PROFILE.
    "grafeo.spec.lpg.cypher.admin.create.index.on.label.property",
    "grafeo.spec.lpg.cypher.admin.create.index.and.query",
    "grafeo.spec.lpg.cypher.admin.drop.index",
    "grafeo.spec.lpg.cypher.admin.show.indexes.empty",
    "grafeo.spec.lpg.cypher.admin.show.indexes.after.create",
    "grafeo.spec.lpg.cypher.admin.profile.match",
    // spec/lpg/cypher/constraints.gtest: CREATE/DROP CONSTRAINT, SHOW
    // CONSTRAINTS.
    "grafeo.spec.lpg.cypher.constraints.create.unique.constraint",
    "grafeo.spec.lpg.cypher.constraints.create.not.null.constraint",
    "grafeo.spec.lpg.cypher.constraints.create.node.key.constraint",
    "grafeo.spec.lpg.cypher.constraints.drop.constraint",
    "grafeo.spec.lpg.cypher.constraints.drop.constraint.if.exists",
    "grafeo.spec.lpg.cypher.constraints.show.constraints.after.create",
    "grafeo.spec.lpg.cypher.constraints.show.constraints.empty",
];

#[derive(Debug, Error)]
pub enum GrafeoError {
    #[error("failed to read Grafeo directory {path}: {source}")]
    ReadDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read Grafeo manifest {path}: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse Grafeo manifest {path}: {source}")]
    Parse {
        path: String,
        source: serde_yaml::Error,
    },
    #[error("Grafeo scenario identity is invalid: {0}")]
    Identity(#[from] crate::identity::TestIdError),
    #[error("Grafeo scenario identity is duplicated: {0}")]
    DuplicateIdentity(TestId),
}

#[derive(Debug, Deserialize)]
struct Manifest {
    meta: Meta,
    #[serde(default)]
    tests: Vec<SourceTest>,
}

#[derive(Debug, Deserialize)]
struct Meta {
    language: Option<String>,
    dataset: Option<YamlValue>,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SourceTest {
    name: String,
    #[serde(default)]
    setup: Vec<String>,
    query: Option<String>,
    #[serde(default)]
    statements: Vec<String>,
    #[serde(default)]
    variants: BTreeMap<String, String>,
    expect: Option<SourceExpectation>,
    skip: Option<YamlValue>,
}

#[derive(Clone, Debug, Deserialize)]
struct SourceExpectation {
    rows: Option<Vec<Vec<YamlValue>>>,
    count: Option<u64>,
    error: Option<YamlValue>,
    ordered: Option<bool>,
    empty: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct GrafeoCase {
    pub id: TestId,
    pub source_path: String,
    pub name: String,
    pub query: String,
    pub statements: Vec<String>,
    pub setup: Vec<String>,
    pub dataset: Option<String>,
    pub title: Option<String>,
    pub expectation: Option<GrafeoExpectation>,
    pub skipped_upstream: bool,
    pub semantic_fingerprint: String,
    semantic_key: String,
}

#[derive(Clone, Debug)]
pub enum GrafeoExpectation {
    Rows {
        rows: Vec<Vec<String>>,
        ordered: bool,
    },
    Count(u64),
    Error,
    Empty,
}

#[derive(Debug)]
pub struct GrafeoCorpus {
    pub cases: Vec<GrafeoCase>,
    pub manifest_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GrafeoStats {
    pub manifests: usize,
    pub cypher_cases: usize,
    pub canonical: usize,
    pub duplicates: usize,
}

impl GrafeoCorpus {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, GrafeoError> {
        let root = root.as_ref();
        let mut paths = Vec::new();
        collect_manifests(root, &mut paths)?;
        paths.sort();
        let mut cases = Vec::new();
        let mut identities = HashSet::new();
        for path in &paths {
            let content = fs::read_to_string(path).map_err(|source| GrafeoError::ReadFile {
                path: path.display().to_string(),
                source,
            })?;
            if !content.lines().any(|line| {
                line.trim() == "language: cypher" || line.trim_start().starts_with("cypher:")
            }) {
                continue;
            }
            let sanitized = sanitize_plain_queries(&content);
            let manifest: Manifest =
                serde_yaml::from_str(&sanitized).map_err(|source| GrafeoError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let relative = path
                .strip_prefix(root)
                .expect("collected paths are rooted in the corpus");
            for test in manifest.tests {
                if manifest.meta.language.as_deref() == Some("cypher") {
                    if let Some(query) = &test.query {
                        push_case(
                            relative,
                            &manifest.meta,
                            &test,
                            query,
                            "native",
                            &mut identities,
                            &mut cases,
                        )?;
                    } else if !test.statements.is_empty() {
                        push_case(
                            relative,
                            &manifest.meta,
                            &test,
                            &test.statements.join(";\n"),
                            "native",
                            &mut identities,
                            &mut cases,
                        )?;
                    }
                }
                if let Some(query) = test.variants.get("cypher") {
                    push_case(
                        relative,
                        &manifest.meta,
                        &test,
                        query,
                        "variant",
                        &mut identities,
                        &mut cases,
                    )?;
                }
            }
        }
        cases.retain(|case| !NON_GQL_CASES.contains(&case.id.as_str()));
        Ok(Self {
            cases,
            manifest_count: paths.len(),
        })
    }

    pub fn stats(&self) -> GrafeoStats {
        let canonical = self
            .cases
            .iter()
            .map(|case| case.semantic_key.as_str())
            .collect::<HashSet<_>>()
            .len();
        GrafeoStats {
            manifests: self.manifest_count,
            cypher_cases: self.cases.len(),
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

fn sanitize_plain_queries(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    for line in content.lines() {
        let indentation = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        let replacement = ["query: ", "cypher: "].into_iter().find_map(|prefix| {
            let value = trimmed.strip_prefix(prefix)?;
            (!value.starts_with(['\'', '"', '|', '>'])).then_some((prefix.trim(), value))
        });
        if let Some((key, value)) = replacement {
            output.push_str(&" ".repeat(indentation));
            output.push_str(key);
            output.push_str(" |-\n");
            output.push_str(&" ".repeat(indentation + 2));
            output.push_str(value);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

fn push_case(
    relative: &Path,
    meta: &Meta,
    test: &SourceTest,
    query: &str,
    variant: &str,
    identities: &mut HashSet<TestId>,
    cases: &mut Vec<GrafeoCase>,
) -> Result<(), GrafeoError> {
    let source_path = relative.to_string_lossy().replace('\\', "/");
    let suffix = if variant == "variant" {
        ".cypher-variant"
    } else {
        ""
    };
    let id = TestId::parse(format!(
        "grafeo.{}.{}{}",
        normalize_identifier(relative.with_extension("").to_string_lossy().as_ref()),
        normalize_identifier(&test.name),
        suffix
    ))?;
    if !identities.insert(id.clone()) {
        return Err(GrafeoError::DuplicateIdentity(id));
    }
    let expectation = normalize_expectation(test.expect.as_ref());
    let dataset = meta.dataset.as_ref().map(yaml_text);
    let semantic_key = semantic_key(dataset.as_deref(), &test.setup, query, expectation.as_ref());
    cases.push(GrafeoCase {
        id,
        source_path,
        name: test.name.clone(),
        query: query.to_owned(),
        statements: test.statements.clone(),
        setup: test.setup.clone(),
        dataset,
        title: meta.title.clone(),
        expectation,
        skipped_upstream: test.skip.is_some(),
        semantic_fingerprint: fingerprint(&semantic_key),
        semantic_key,
    });
    Ok(())
}

fn normalize_expectation(source: Option<&SourceExpectation>) -> Option<GrafeoExpectation> {
    let source = source?;
    if source.error.is_some() {
        return Some(GrafeoExpectation::Error);
    }
    if source.empty == Some(true) {
        return Some(GrafeoExpectation::Empty);
    }
    if let Some(count) = source.count {
        return Some(GrafeoExpectation::Count(count));
    }
    source.rows.as_ref().map(|rows| GrafeoExpectation::Rows {
        rows: rows
            .iter()
            .map(|row| row.iter().map(yaml_text).collect())
            .collect(),
        ordered: source.ordered.unwrap_or(false),
    })
}

fn semantic_key(
    dataset: Option<&str>,
    setup: &[String],
    query: &str,
    expectation: Option<&GrafeoExpectation>,
) -> String {
    format!(
        "dataset={}\nsetup={}\nquery={}\nexpectation={expectation:?}",
        dataset.unwrap_or("empty"),
        setup
            .iter()
            .map(|value| normalize_whitespace(value))
            .collect::<Vec<_>>()
            .join("\u{1e}"),
        normalize_whitespace(query),
    )
}

fn run_canonical(
    case: &GrafeoCase,
    environment: RunEnvironment,
    run_id: &str,
    parse_cache: &mut QueryParseCache,
) -> ResultRecord {
    let started = Instant::now();
    let expectation = match case.expectation {
        Some(GrafeoExpectation::Error) => Expectation::Error,
        _ => Expectation::Rows,
    };
    let parse_result = if case.statements.is_empty() {
        parse_cache.parse(&case.query)
    } else {
        case.statements
            .iter()
            .try_for_each(|statement| parse_cache.parse(statement))
    };
    let (outcome, rows, message, execution) = match parse_result {
        Err(error) if expectation == Expectation::Error => {
            (Outcome::Passed, None, Some(error), "parser")
        }
        Err(error) => (Outcome::Failed, None, Some(error), "parser"),
        Ok(_) => execute_case(case, expectation),
    };
    base_record(
        case,
        environment,
        run_id,
        expectation,
        outcome,
        started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
        rows,
        message,
        execution,
    )
}

fn execute_case(
    case: &GrafeoCase,
    expectation: Expectation,
) -> (
    Outcome,
    Option<Vec<Vec<String>>>,
    Option<String>,
    &'static str,
) {
    let fixture = match empty_fixture(case.id.as_str()) {
        Ok(fixture) => fixture,
        Err(error) => {
            return (Outcome::Failed, None, Some(error.to_string()), "execution");
        }
    };
    if let Some(dataset) = case
        .dataset
        .as_deref()
        .filter(|dataset| *dataset != "empty")
    {
        let statements = match dataset_statements(dataset) {
            Ok(statements) => statements,
            Err(error) => return (Outcome::Failed, None, Some(error), "dataset-execution"),
        };
        for statement in statements {
            if let Err(error) = execute_statement(&fixture.session, &statement) {
                return (
                    Outcome::Failed,
                    None,
                    Some(format!("Grafeo dataset `{dataset}` setup failed: {error}")),
                    "dataset-execution",
                );
            }
        }
    }
    for setup in &case.setup {
        if let Err(error) = execute_statement(&fixture.session, setup) {
            return (
                Outcome::Failed,
                None,
                Some(format!(
                    "Grafeo setup query failed: {error}; query: {setup}"
                )),
                "setup-execution",
            );
        }
    }
    let statements = if case.statements.is_empty() {
        vec![case.query.as_str()]
    } else {
        case.statements.iter().map(String::as_str).collect()
    };
    let mut rows = Vec::new();
    for statement in statements {
        match execute_statement(&fixture.session, statement) {
            Ok(result) => rows = result,
            Err(error) if expectation == Expectation::Error => {
                return (Outcome::Passed, None, Some(error), "execution");
            }
            Err(error) => return (Outcome::Failed, None, Some(error), "execution"),
        }
    }
    match rows {
        rows if expectation == Expectation::Error => (
            Outcome::Failed,
            Some(stringify_rows(rows)),
            Some("expected an error but execution succeeded".to_owned()),
            "execution",
        ),
        rows => {
            let types = fixture
                .session
                .prepare(&case.query, &Parameters::new())
                .ok()
                .map(|statement| statement.result_types().to_vec());
            compare_rows(case, stringify_rows_with_types(rows, types.as_deref()))
        }
    }
}

fn dataset_statements(dataset: &str) -> Result<Vec<String>, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../testdata/donors/grafeo/tests/spec/datasets")
        .join(format!("{dataset}.setup"));
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read Grafeo dataset {}: {error}", path.display()))?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.strip_prefix("INSERT ")
                .map(|suffix| format!("CREATE {suffix}"))
                .unwrap_or_else(|| line.replace(" INSERT ", " CREATE "))
        })
        .collect())
}

fn execute_statement(
    session: &GraphConnection,
    statement: &str,
) -> Result<Vec<Vec<Value>>, String> {
    let parameters = Parameters::new();
    match session.query(statement, &parameters) {
        Ok(rows) => Ok(rows),
        Err(query_error) => session
            .execute(statement, &parameters)
            .map(|summary| summary.rows)
            .map_err(|mutation_error| {
                format!(
                    "query execution failed: {query_error}; mutation execution failed: {mutation_error}; query: {statement}"
                )
            }),
    }
}

fn compare_rows(
    case: &GrafeoCase,
    mut rows: Vec<Vec<String>>,
) -> (
    Outcome,
    Option<Vec<Vec<String>>>,
    Option<String>,
    &'static str,
) {
    let (matches, expected) = match case.expectation.as_ref() {
        Some(GrafeoExpectation::Rows {
            rows: expected,
            ordered,
        }) => {
            let mut expected = expected.clone();
            if !ordered {
                rows.sort();
                expected.sort();
            }
            (rows == expected, format!("{expected:?}"))
        }
        Some(GrafeoExpectation::Count(count)) => {
            (rows.len() as u64 == *count, format!("{count} rows"))
        }
        Some(GrafeoExpectation::Empty) => (rows.is_empty(), "no rows".to_owned()),
        Some(GrafeoExpectation::Error) => (false, "an error".to_owned()),
        None => return (Outcome::Passed, Some(rows), None, "execution"),
    };
    if matches {
        (Outcome::Passed, Some(rows), None, "execution")
    } else {
        (
            Outcome::Failed,
            Some(rows.clone()),
            Some(format!("expected {expected}, observed {rows:?}")),
            "execution",
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "result records require explicit outcome evidence"
)]
fn base_record(
    case: &GrafeoCase,
    environment: RunEnvironment,
    run_id: &str,
    expectation: Expectation,
    outcome: Outcome,
    duration_ns: u64,
    rows: Option<Vec<Vec<String>>>,
    message: Option<String>,
    execution: &str,
) -> ResultRecord {
    let row_count = rows.as_ref().map(|rows| rows.len() as u64);
    let digest = rows.as_ref().map(|rows| result_digest(rows));
    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: recorded_at(),
        environment,
        suite: "grafeo-deep".to_owned(),
        test_id: case.id.clone(),
        kind: TestKind::Conformance,
        area: case
            .source_path
            .split('/')
            .nth(1)
            .unwrap_or("grafeo")
            .to_owned(),
        fixture: case.dataset.clone().unwrap_or_else(|| "empty".to_owned()),
        expectation,
        outcome,
        duration_ns,
        source: SourceIdentity {
            name: "Grafeo".to_owned(),
            repository: "https://github.com/GrafeoDB/grafeo".to_owned(),
            revision: GRAFEO_REVISION.to_owned(),
            path: format!("tests/{}", case.source_path),
            case: case.name.clone(),
            license: "Apache-2.0".to_owned(),
            adaptation: "verbatim-gtest-case".to_owned(),
            issue: None,
            fixed_commit: None,
        },
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
        dimensions: BTreeMap::from([
            (
                "semantic_fingerprint".to_owned(),
                case.semantic_fingerprint.clone(),
            ),
            ("execution".to_owned(), execution.to_owned()),
            (
                "upstream_skipped".to_owned(),
                case.skipped_upstream.to_string(),
            ),
            (
                "manifest_title".to_owned(),
                case.title.clone().unwrap_or_default(),
            ),
        ]),
    }
}

fn collect_manifests(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), GrafeoError> {
    let entries = fs::read_dir(directory).map_err(|source| GrafeoError::ReadDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| GrafeoError::ReadDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("gtest") {
            paths.push(path);
        }
    }
    Ok(())
}

fn normalize_identifier(value: &str) -> String {
    let mut normalized = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            separator = false;
        } else if !separator && !normalized.is_empty() {
            normalized.push('-');
            separator = true;
        }
    }
    normalized.trim_matches('-').replace('-', ".")
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn yaml_text(value: &YamlValue) -> String {
    match value {
        YamlValue::Null => "<null>".to_owned(),
        YamlValue::Bool(value) => value.to_string(),
        YamlValue::Number(value) => value.to_string(),
        YamlValue::String(value) => value.clone(),
        value => serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}")),
    }
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
                        Value::Numeric(Numeric::Float(value)) => value.to_string(),
                        Value::Text(value) => value.to_string(),
                        Value::Blob(value) => format!("{value:?}"),
                    }
                })
                .collect()
        })
        .collect()
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
    fn named_dataset_statements_are_loaded_as_cypher_mutations() {
        let statements = dataset_statements("social_network").unwrap();
        assert!(!statements.is_empty());
        assert!(statements
            .iter()
            .all(|statement| !statement.contains(" INSERT ")));
        assert!(statements[0].starts_with("CREATE "));
    }

    #[test]
    fn imports_every_cypher_native_and_variant_case() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/grafeo/tests");
        let corpus = GrafeoCorpus::load(root).unwrap();
        assert_eq!(corpus.stats().manifests, 157);
        assert_eq!(corpus.stats().cypher_cases, 372);
    }
}
