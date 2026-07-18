use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use thiserror::Error;

use crate::{
    history::recorded_at,
    identity::TestId,
    model::{
        Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
        HISTORY_SCHEMA_VERSION,
    },
    query_cache::QueryParseCache,
};

const LADYBUG_REVISION: &str = "7eab431c6becf64f58f7c2ff4c0fb1f160acb492";

#[derive(Debug, Error)]
pub enum LadybugError {
    #[error("failed to read Ladybug directory {path}: {source}")]
    ReadDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read Ladybug test {path}: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("Ladybug statement in {path}:{line} has no expected-output marker")]
    MissingExpectation { path: String, line: usize },
    #[error("Ladybug scenario identity is invalid: {0}")]
    Identity(#[from] crate::identity::TestIdError),
    #[error("Ladybug scenario identity is duplicated: {0}")]
    DuplicateIdentity(TestId),
}

#[derive(Clone, Debug)]
pub struct LadybugCase {
    pub id: TestId,
    pub source_path: String,
    pub source_line: usize,
    pub dataset: String,
    pub case_name: String,
    pub log_name: String,
    pub query: String,
    pub expected_contract: String,
    pub expected_lines: Vec<String>,
    pub semantic_fingerprint: String,
    semantic_key: String,
}

#[derive(Debug)]
pub struct LadybugCorpus {
    pub cases: Vec<LadybugCase>,
    pub file_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LadybugStats {
    pub files: usize,
    pub statements: usize,
    pub canonical: usize,
    pub duplicates: usize,
}

impl LadybugCorpus {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, LadybugError> {
        let root = root.as_ref();
        let mut paths = Vec::new();
        collect_test_files(root, &mut paths)?;
        paths.sort();
        let mut identities = HashSet::new();
        let mut cases = Vec::new();
        for path in &paths {
            parse_file(root, path, &mut identities, &mut cases)?;
        }
        Ok(Self {
            cases,
            file_count: paths.len(),
        })
    }

    pub fn stats(&self) -> LadybugStats {
        let canonical = self
            .cases
            .iter()
            .map(|case| case.semantic_key.as_str())
            .collect::<HashSet<_>>()
            .len();
        LadybugStats {
            files: self.file_count,
            statements: self.cases.len(),
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

fn parse_file(
    root: &Path,
    path: &Path,
    identities: &mut HashSet<TestId>,
    cases: &mut Vec<LadybugCase>,
) -> Result<(), LadybugError> {
    let content = fs::read_to_string(path).map_err(|source| LadybugError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;
    let lines = content.lines().collect::<Vec<_>>();
    let relative = path
        .strip_prefix(root)
        .expect("collected paths are rooted in the corpus");
    let source_path = relative.to_string_lossy().replace('\\', "/");
    let mut dataset = "unspecified".to_owned();
    let mut case_name = "uncased".to_owned();
    let mut log_name = String::new();
    let mut statement_index = 0_usize;
    let mut index = 0_usize;
    while index < lines.len() {
        let line = lines[index];
        if let Some(value) = line.strip_prefix("-DATASET ") {
            value.trim().clone_into(&mut dataset);
            index += 1;
            continue;
        }
        if let Some(value) = line.strip_prefix("-CASE ") {
            value.trim().clone_into(&mut case_name);
            log_name.clear();
            index += 1;
            continue;
        }
        if let Some(value) = line.strip_prefix("-LOG ") {
            value.trim().clone_into(&mut log_name);
            index += 1;
            continue;
        }
        let Some(first_query_line) = line.strip_prefix("-STATEMENT") else {
            index += 1;
            continue;
        };
        let source_line = index + 1;
        statement_index += 1;
        let mut query_lines = Vec::new();
        if !first_query_line.trim().is_empty() {
            query_lines.push(first_query_line.trim().to_owned());
        }
        index += 1;
        while index < lines.len()
            && !lines[index].starts_with("----")
            && !is_directive(lines[index])
        {
            if !lines[index].trim().is_empty() {
                query_lines.push(lines[index].trim().to_owned());
            }
            index += 1;
        }
        let mut expected_lines = Vec::new();
        let expected_contract = if index < lines.len() && lines[index].starts_with("----") {
            let contract = lines[index]
                .strip_prefix("----")
                .expect("checked expectation prefix")
                .trim()
                .to_owned();
            index += 1;
            while index < lines.len() && !is_directive(lines[index]) {
                if !lines[index].is_empty() {
                    expected_lines.push(lines[index].to_owned());
                }
                index += 1;
            }
            contract
        } else {
            "missing".to_owned()
        };
        let query = query_lines.join(" ");
        let id = TestId::parse(format!(
            "ladybug.{}.{}.statement-{statement_index}",
            normalize_identifier(relative.with_extension("").to_string_lossy().as_ref()),
            normalize_identifier(&case_name)
        ))?;
        if !identities.insert(id.clone()) {
            return Err(LadybugError::DuplicateIdentity(id));
        }
        let semantic_key = format!(
            "dataset={}\nquery={}\ncontract={}\nexpected={}",
            dataset,
            normalize_whitespace(&query),
            expected_contract,
            expected_lines.join("\u{1e}")
        );
        cases.push(LadybugCase {
            id,
            source_path: source_path.clone(),
            source_line,
            dataset: dataset.clone(),
            case_name: case_name.clone(),
            log_name: log_name.clone(),
            query,
            expected_contract,
            expected_lines,
            semantic_fingerprint: fingerprint(&semantic_key),
            semantic_key,
        });
    }
    Ok(())
}

fn is_directive(line: &str) -> bool {
    line.starts_with("-STATEMENT")
        || line.starts_with("-CASE ")
        || line.starts_with("-LOG ")
        || line.starts_with("-DATASET ")
        || line == "--"
}

fn run_canonical(
    case: &LadybugCase,
    environment: RunEnvironment,
    run_id: &str,
    parse_cache: &mut QueryParseCache,
) -> ResultRecord {
    let started = Instant::now();
    let expects_error = case.expected_contract.starts_with("error");
    let expectation = if expects_error {
        Expectation::Error
    } else {
        Expectation::Rows
    };
    let (outcome, message, execution) = match parse_cache.parse(&case.query) {
        Err(error) if expects_error => (Outcome::Passed, Some(error), "parser"),
        Err(error) => (Outcome::Unsupported, Some(error), "parser"),
        Ok(_) => (
            Outcome::Unsupported,
            Some(
                "query parses, but the Ladybug dataset and expected graph-value format are not yet executable by the generic adapter"
                    .to_owned(),
            ),
            "adapter",
        ),
    };
    base_record(
        case,
        environment,
        run_id,
        expectation,
        outcome,
        started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
        message,
        execution,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "result records require explicit outcome evidence"
)]
fn base_record(
    case: &LadybugCase,
    environment: RunEnvironment,
    run_id: &str,
    expectation: Expectation,
    outcome: Outcome,
    duration_ns: u64,
    message: Option<String>,
    execution: &str,
) -> ResultRecord {
    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: recorded_at(),
        environment,
        suite: "ladybug-deep".to_owned(),
        test_id: case.id.clone(),
        kind: TestKind::Conformance,
        area: case
            .source_path
            .split('/')
            .next()
            .unwrap_or("ladybug")
            .to_owned(),
        fixture: case.dataset.clone(),
        expectation,
        outcome,
        duration_ns,
        source: SourceIdentity {
            name: "Ladybug".to_owned(),
            repository: "https://github.com/mwatts/ladybug".to_owned(),
            revision: LADYBUG_REVISION.to_owned(),
            path: format!("test/test_files/{}", case.source_path),
            case: if case.log_name.is_empty() {
                case.case_name.clone()
            } else {
                format!("{} / {}", case.case_name, case.log_name)
            },
            license: "MIT".to_owned(),
            adaptation: "verbatim-statement-assertion".to_owned(),
            issue: None,
            fixed_commit: None,
        },
        operation: None,
        graph_shape: None,
        scale: None,
        iterations: None,
        throughput_per_second: None,
        row_count: None,
        node_count: None,
        relationship_count: None,
        result_digest: None,
        message,
        dimensions: BTreeMap::from([
            (
                "semantic_fingerprint".to_owned(),
                case.semantic_fingerprint.clone(),
            ),
            ("execution".to_owned(), execution.to_owned()),
            ("source_line".to_owned(), case.source_line.to_string()),
            (
                "expected_contract".to_owned(),
                case.expected_contract.clone(),
            ),
            (
                "expected_line_count".to_owned(),
                case.expected_lines.len().to_string(),
            ),
        ]),
    }
}

fn alias_record(
    case: &LadybugCase,
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
        canonical.message.clone(),
        "deduplicated",
    );
    record.dimensions.insert(
        "canonical_test_id".to_owned(),
        canonical.test_id.to_string(),
    );
    record
}

fn collect_test_files(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), LadybugError> {
    let entries = fs::read_dir(directory).map_err(|source| LadybugError::ReadDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| LadybugError::ReadDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("test") {
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
            normalized.push('.');
            separator = true;
        }
    }
    normalized.trim_matches('.').to_owned()
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
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
    fn imports_every_statement_assertion() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/ladybug/test_files");
        let corpus = LadybugCorpus::load(&root).unwrap();
        let mut source_counts = HashMap::<String, usize>::new();
        for case in &corpus.cases {
            *source_counts.entry(case.source_path.clone()).or_default() += 1;
        }
        let mut paths = Vec::new();
        collect_test_files(&root, &mut paths).unwrap();
        for path in paths {
            let relative = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(path).unwrap();
            let markers = source
                .lines()
                .filter(|line| line.starts_with("-STATEMENT"))
                .count();
            let imported = source_counts.get(&relative).copied().unwrap_or_default();
            assert_eq!(markers, imported, "statement discovery drift in {relative}");
        }
        assert_eq!(corpus.stats().files, 477);
        assert_eq!(corpus.stats().statements, 15_940);
    }
}
