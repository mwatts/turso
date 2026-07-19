use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use regex::Regex;
use thiserror::Error;
use turso_graph_frontend::MutationParameters;

use crate::{
    history::recorded_at,
    identity::TestId,
    model::{
        Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
        HISTORY_SCHEMA_VERSION,
    },
    query_cache::QueryParseCache,
    runner::empty_fixture,
};

const AGE_REVISION: &str = "6876abcab0a3281eb65a7e2a91238e0b5abfdea7";

#[derive(Debug, Error)]
pub enum AgeError {
    #[error("failed to read AGE directory {path}: {source}")]
    ReadDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read AGE regression {path}: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("AGE query identity is invalid: {0}")]
    Identity(#[from] crate::identity::TestIdError),
    #[error("AGE query identity is duplicated: {0}")]
    DuplicateIdentity(TestId),
}

#[derive(Clone, Debug)]
pub struct AgeCase {
    pub id: TestId,
    pub source_path: String,
    pub source_line: usize,
    pub query: String,
    pub graph_argument: String,
    pub semantic_fingerprint: String,
    semantic_key: String,
}

#[derive(Debug)]
pub struct AgeCorpus {
    pub cases: Vec<AgeCase>,
    pub file_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgeStats {
    pub files: usize,
    pub queries: usize,
    pub canonical: usize,
    pub duplicates: usize,
}

impl AgeCorpus {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, AgeError> {
        let root = root.as_ref();
        let mut paths = Vec::new();
        collect_sql_files(root, &mut paths)?;
        paths.sort();
        let invocation = Regex::new(r"(?is)cypher\s*\(\s*([^,]+?)\s*,\s*\$\$(.*?)\$\$")
            .expect("static AGE extraction regex is valid");
        let mut identities = HashSet::new();
        let mut cases = Vec::new();
        for path in &paths {
            let content = fs::read_to_string(path).map_err(|source| AgeError::ReadFile {
                path: path.display().to_string(),
                source,
            })?;
            let relative = path
                .strip_prefix(root)
                .expect("collected paths are rooted in the corpus");
            let source_path = relative.to_string_lossy().replace('\\', "/");
            for (index, captures) in invocation.captures_iter(&content).enumerate() {
                let whole = captures.get(0).expect("whole regex capture exists");
                let graph_argument = captures
                    .get(1)
                    .expect("graph argument capture exists")
                    .as_str()
                    .trim()
                    .to_owned();
                let query = captures
                    .get(2)
                    .expect("query capture exists")
                    .as_str()
                    .trim()
                    .to_owned();
                let id = TestId::parse(format!(
                    "age.{}.query-{}",
                    normalize_identifier(relative.with_extension("").to_string_lossy().as_ref()),
                    index + 1
                ))?;
                if !identities.insert(id.clone()) {
                    return Err(AgeError::DuplicateIdentity(id));
                }
                let semantic_key = normalize_query(&query);
                cases.push(AgeCase {
                    id,
                    source_path: source_path.clone(),
                    source_line: content[..whole.start()].lines().count() + 1,
                    query,
                    graph_argument,
                    semantic_fingerprint: fingerprint(&semantic_key),
                    semantic_key,
                });
            }
        }
        Ok(Self {
            cases,
            file_count: paths.len(),
        })
    }

    pub fn stats(&self) -> AgeStats {
        let canonical = self
            .cases
            .iter()
            .map(|case| case.semantic_key.as_str())
            .collect::<HashSet<_>>()
            .len();
        AgeStats {
            files: self.file_count,
            queries: self.cases.len(),
            canonical,
            duplicates: self.cases.len() - canonical,
        }
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

fn run_canonical(
    case: &AgeCase,
    environment: RunEnvironment,
    run_id: &str,
    parse_cache: &mut QueryParseCache,
) -> ResultRecord {
    let started = Instant::now();
    let (outcome, message, execution) = match parse_cache.parse(&case.query) {
        Ok(()) => match empty_fixture(case.id.as_str()) {
            Ok(fixture) => match fixture
                .session
                .query(&case.query, &MutationParameters::new())
            {
                Ok(_) => (Outcome::Passed, None, "execution"),
                // Mirror the TCK statement router: statements the read
                // pipeline rejects may still be executable mutations.
                Err(query_error) => match fixture
                    .session
                    .mutate(&case.query, &MutationParameters::new())
                {
                    Ok(_) => (Outcome::Passed, None, "execution"),
                    Err(mutation_error) => (
                        Outcome::Failed,
                        Some(format!(
                            "query execution failed: {query_error}; mutation execution failed: {mutation_error}"
                        )),
                        "execution",
                    ),
                },
            },
            Err(error) => (
                Outcome::Failed,
                Some(error.to_string()),
                "fixture-execution",
            ),
        },
        Err(error) => (Outcome::Failed, Some(error), "parser"),
    };
    base_record(
        case,
        environment,
        run_id,
        started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
        outcome,
        message,
        execution,
    )
}

fn base_record(
    case: &AgeCase,
    environment: RunEnvironment,
    run_id: &str,
    duration_ns: u64,
    outcome: Outcome,
    message: Option<String>,
    execution: &str,
) -> ResultRecord {
    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: recorded_at(),
        environment,
        suite: "age-deep".to_owned(),
        test_id: case.id.clone(),
        kind: TestKind::Conformance,
        area: case.source_path.trim_end_matches(".sql").to_owned(),
        fixture: case.graph_argument.clone(),
        expectation: Expectation::Rows,
        outcome,
        duration_ns,
        source: SourceIdentity {
            name: "Apache AGE".to_owned(),
            repository: "https://github.com/apache/age".to_owned(),
            revision: AGE_REVISION.to_owned(),
            path: format!("regress/sql/{}", case.source_path),
            case: format!("cypher invocation at line {}", case.source_line),
            license: "Apache-2.0".to_owned(),
            adaptation: "verbatim-dollar-quoted-query".to_owned(),
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
        ]),
    }
}

fn collect_sql_files(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), AgeError> {
    let entries = fs::read_dir(directory).map_err(|source| AgeError::ReadDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| AgeError::ReadDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_sql_files(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("sql") {
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

fn normalize_query(query: &str) -> String {
    query
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
    fn imports_every_dollar_quoted_cypher_invocation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/age/sql");
        let corpus = AgeCorpus::load(root).unwrap();
        assert_eq!(corpus.stats().files, 47);
        assert_eq!(corpus.stats().queries, 3_677);
    }
}
