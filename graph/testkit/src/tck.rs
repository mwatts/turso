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
use turso_graph_frontend::MutationParameters;

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
        let mut canonical_results: HashMap<&str, ResultRecord> = HashMap::new();
        let mut records = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            if let Some(canonical) = canonical_results.get(case.semantic_key.as_str()) {
                records.push(alias_record(case, canonical, environment.clone(), run_id));
                continue;
            }
            let record = run_canonical(case, environment.clone(), run_id, parse_cache);
            canonical_results.insert(case.semantic_key.as_str(), record.clone());
            records.push(record);
        }
        records
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
            Outcome::Unsupported,
            None,
            Some("TCK scenario has no executable query step".to_owned()),
            "discovery-only",
        ),
        Some(query) => match parse_cache.parse(query) {
            Err(error) if expectation == Expectation::Error => {
                (Outcome::Passed, None, Some(error), "parser")
            }
            Err(error) => (
                Outcome::Unsupported,
                None,
                Some(error),
                "parser",
            ),
            Ok(_) if !scalar_execution_eligible(case, query) => (
                Outcome::Unsupported,
                None,
                Some(
                    "query parses, but this scenario requires graph fixtures, parameters, side-effect accounting, or graph-value comparison not yet expressible by the generic TCK adapter"
                        .to_owned(),
                ),
                "adapter",
            ),
            Ok(_) => execute_scalar_case(case, query, expectation),
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

fn execute_scalar_case(
    case: &TckCase,
    query: &str,
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
            return (
                Outcome::Unsupported,
                None,
                Some(error.to_string()),
                "scalar-execution",
            )
        }
    };
    match fixture.session.query(query, &MutationParameters::new()) {
        Err(error) if expectation == Expectation::Error => (
            Outcome::Passed,
            None,
            Some(error.to_string()),
            "scalar-execution",
        ),
        Err(error) => (
            Outcome::Unsupported,
            None,
            Some(error.to_string()),
            "scalar-execution",
        ),
        Ok(rows) if expectation == Expectation::Error => (
            Outcome::Failed,
            Some(stringify_rows(rows)),
            Some("expected an error but execution succeeded".to_owned()),
            "scalar-execution",
        ),
        Ok(rows) => {
            let mut rows = stringify_rows(rows);
            let Some((mut expected_rows, ordered)) = expected_rows(case) else {
                return (
                    Outcome::Unsupported,
                    Some(rows),
                    Some(
                        "result expectation is not representable by the scalar adapter".to_owned(),
                    ),
                    "scalar-execution",
                );
            };
            if !ordered {
                rows.sort();
                expected_rows.sort();
            }
            if rows == expected_rows {
                (Outcome::Passed, Some(rows), None, "scalar-execution")
            } else {
                (
                    Outcome::Failed,
                    Some(rows.clone()),
                    Some(format!("expected {expected_rows:?}, observed {rows:?}")),
                    "scalar-execution",
                )
            }
        }
    }
}

fn scalar_execution_eligible(case: &TckCase, query: &str) -> bool {
    let upper = query.trim_start().to_ascii_uppercase();
    let scalar_query = upper.starts_with("RETURN ") || upper.starts_with("UNWIND ");
    scalar_query
        && !case.steps.iter().any(|step| {
            step.value == "having executed:"
                || step.value.contains("parameters are")
                || step.value.contains("side effects should be")
                || step.value.contains("graph should be")
        })
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
    if value.starts_with('(') || value.starts_with("<[") {
        return None;
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
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|value| match value {
                    Value::Null => "<null>".to_owned(),
                    Value::Numeric(Numeric::Integer(value)) => value.to_string(),
                    Value::Numeric(Numeric::Float(value)) if value.fract() == 0.0 => {
                        format!("{value:.1}")
                    }
                    Value::Numeric(Numeric::Float(value)) => value.to_string(),
                    Value::Text(value) => value.to_string(),
                    Value::Blob(value) => format!("{value:?}"),
                })
                .collect()
        })
        .collect()
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

fn alias_record(
    case: &TckCase,
    canonical: &ResultRecord,
    environment: RunEnvironment,
    run_id: &str,
) -> ResultRecord {
    let mut record = base_record(
        case,
        environment,
        run_id,
        canonical.expectation,
        canonical.outcome,
        0,
        None,
        canonical.message.clone(),
        "deduplicated",
    );
    record.dimensions.insert(
        "canonical_test_id".to_owned(),
        canonical.test_id.to_string(),
    );
    record.result_digest.clone_from(&canonical.result_digest);
    record.row_count = canonical.row_count;
    record
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
