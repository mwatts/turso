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
    /// AGE's expected output for this invocation is an ERROR, so erroring is
    /// the passing outcome and successful execution is the failure.
    pub expects_error: bool,
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

/// Donor files testing postgres/AGE-specific surfaces rather than Cypher:
/// jsonb operator syntax, pgvector, postgres extensions, and jsonb casts.
/// These stay out of the conformance corpus entirely.
const AGE_SPECIFIC_FILES: [&str; 2] = ["pg_trgm.sql", "fuzzystrmatch.sql"];

impl AgeCorpus {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, AgeError> {
        let root = root.as_ref();
        let mut paths = Vec::new();
        collect_sql_files(root, &mut paths)?;
        paths.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_none_or(|name| !AGE_SPECIFIC_FILES.contains(&name))
        });
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
            let expected = expected_outcomes(root, relative, &invocation);
            let mut expected_cursor = 0;
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
                let expects_error =
                    aligned_expectation(&expected, &mut expected_cursor, &semantic_key);
                cases.push(AgeCase {
                    id,
                    source_path: source_path.clone(),
                    source_line: content[..whole.start()].lines().count() + 1,
                    query,
                    graph_argument,
                    semantic_fingerprint: fingerprint(&semantic_key),
                    expects_error,
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
    // Rows whose expected AGE output is an ERROR pass by erroring: rejecting
    // the query anywhere in the pipeline matches AGE's observable behavior,
    // while executing it successfully accepts a query AGE rejects.
    let succeeded = |error: Option<String>| match (case.expects_error, error) {
        (false, None) => (Outcome::Passed, None),
        (false, Some(message)) => (Outcome::Failed, Some(message)),
        (true, Some(_)) => (Outcome::Passed, None),
        (true, None) => (
            Outcome::Failed,
            Some("query succeeded but AGE expects an error".to_owned()),
        ),
    };
    let (outcome, message, execution) = match parse_cache.parse(&case.query) {
        Ok(()) => match empty_fixture(case.id.as_str()) {
            Ok(fixture) => match fixture
                .session
                .query(&case.query, &MutationParameters::new())
            {
                Ok(_) => {
                    let (outcome, message) = succeeded(None);
                    (outcome, message, "execution")
                }
                // Mirror the TCK statement router: statements the read
                // pipeline rejects may still be executable mutations.
                Err(query_error) => match fixture
                    .session
                    .mutate(&case.query, &MutationParameters::new())
                {
                    Ok(_) => {
                        let (outcome, message) = succeeded(None);
                        (outcome, message, "execution")
                    }
                    Err(mutation_error) => {
                        let (outcome, message) = succeeded(Some(format!(
                            "query execution failed: {query_error}; mutation execution failed: {mutation_error}"
                        )));
                        (outcome, message, "execution")
                    }
                },
            },
            Err(error) => (
                Outcome::Failed,
                Some(error.to_string()),
                "fixture-execution",
            ),
        },
        Err(error) => {
            let (outcome, message) = succeeded(Some(error));
            (outcome, message, "parser")
        }
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
        expectation: if case.expects_error {
            Expectation::Error
        } else {
            Expectation::Rows
        },
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

/// Parse the psql expected output alongside a regression file into the
/// ordered (normalized query, expects-error) sequence of its cypher
/// invocations. Missing expected files yield an empty sequence, which
/// defaults every query to a row expectation.
fn expected_outcomes(root: &Path, relative: &Path, invocation: &Regex) -> Vec<(String, bool)> {
    let Some(parent) = root.parent() else {
        return Vec::new();
    };
    let path = parent.join("expected").join(relative.with_extension("out"));
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    invocation
        .captures_iter(&content)
        .map(|captures| {
            let query = captures
                .get(2)
                .expect("query capture exists")
                .as_str()
                .trim();
            let rest = &content[captures.get(0).expect("whole capture exists").end()..];
            // The statement's own output follows the terminating `);` of the
            // echoed invocation: either a result-set header or an ERROR line.
            let output = rest
                .split_once(';')
                .map_or("", |(_, output)| output)
                .trim_start();
            let file_stem = relative
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default();
            let expects_error = output.strip_prefix("ERROR:").is_some_and(|message| {
                !is_infrastructure_error(message) && !is_age_restriction_error(message, file_stem)
            });
            (normalize_query(query), expects_error)
        })
        .collect()
}

/// Expected errors raised by Postgres infrastructure (roles, row-level
/// security, unique-constraint indexes, missing graphs) or AGE internals
/// rather than Cypher semantics: the Cypher query itself is valid, so those
/// rows keep row expectations.
fn is_infrastructure_error(message: &str) -> bool {
    let first_line = message.trim_start().lines().next().unwrap_or_default();
    first_line.starts_with("permission denied")
        || first_line.contains("row-level security")
        || (first_line.starts_with("graph") && first_line.contains("does not exist"))
        || first_line.starts_with("duplicate key value violates unique constraint")
        || first_line.starts_with("could not find rte")
        || first_line.contains("is only for internal use")
}

/// Expected errors where AGE rejects a query that is valid openCypher/GQL,
/// or where the error belongs to the surrounding SQL invocation rather than
/// the Cypher text. Executing these successfully is correct for a Cypher
/// engine, so the rows keep row expectations:
///
/// - re-binding a variable with an additional label/type is a conjunctive
///   predicate in openCypher/GQL, not an error
/// - fresh variables inside EXISTS patterns are valid pattern predicates
/// - backtick-quoted names may contain arbitrary characters
/// - repeated ON CREATE SET actions are allowed by the openCypher grammar
/// - labels and relationship types are separate namespaces
/// - shortest-path errors come from AGE's SQL table functions, and cast or
///   invocation errors from the outer SELECT's column definition list
fn is_age_restriction_error(message: &str, file_stem: &str) -> bool {
    let first_line = message.trim_start().lines().next().unwrap_or_default();
    // Adding a label to a re-bound variable is a conjunctive predicate in a
    // MATCH, but inside a MERGE pattern it is a genuine creation conflict.
    let rebind_label_predicate =
        first_line.starts_with("multiple labels for variable") && file_stem != "cypher_merge";
    rebind_label_predicate
        || first_line.starts_with("label name is invalid")
        || first_line.starts_with("ON CREATE SET specified more than once")
        || first_line.contains("is for vertices, not edges")
        || first_line.contains("is for edges, not vertices")
        || first_line.starts_with("age_shortest_path:")
        || first_line.starts_with("age_all_shortest_paths:")
        || first_line.contains("WITH ORDINALITY")
        || first_line.contains("ROWS FROM")
        || first_line.starts_with("column definition list")
        || first_line.starts_with("return row and column definition list")
        || first_line.contains("cannot be rescanned")
        || first_line.contains("for column")
        || first_line.starts_with("cannot cast agtype object")
        || first_line.starts_with("cannot cast agtype array")
        // pgvector distance functions exist in this engine even though AGE
        // lacks them; generic unknown-function errors stay error-expected.
        || first_line.contains("cosine_distance")
        || first_line.contains("l2_distance")
        || first_line.contains("inner_product")
        || first_line.starts_with("unsupported Unicode escape")
}

/// The expected output echoes invocations in order but can interleave extra
/// matches (error context lines, non-regression statements), so align by
/// scanning forward for the next entry with the same normalized query text.
fn aligned_expectation(
    expected: &[(String, bool)],
    cursor: &mut usize,
    semantic_key: &str,
) -> bool {
    let found = expected[*cursor..]
        .iter()
        .position(|(query, _)| query == semantic_key);
    match found {
        Some(offset) => {
            let expects_error = expected[*cursor + offset].1;
            *cursor += offset + 1;
            expects_error
        }
        None => false,
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
    fn error_expectations_come_from_the_expected_output_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/age/sql");
        let corpus = AgeCorpus::load(root).unwrap();
        let case = |id: &str| {
            corpus
                .cases
                .iter()
                .find(|case| case.id.as_str() == id)
                .unwrap()
        };
        // AGE rejects properties on an already-bound CREATE node; erroring is
        // the expected outcome, not a failure.
        assert!(case("age.cypher.create.query-38").expects_error);
        // Plain CREATE of a fresh node returns rows.
        assert!(!case("age.cypher.create.query-1").expects_error);
        // Files without an expected output default to row expectations.
        assert!(!case("age.issue.369.query-1").expects_error);
        // Postgres-infrastructure errors (roles, row-level security) are not
        // Cypher semantics; those rows keep row expectations.
        assert!(corpus
            .cases
            .iter()
            .filter(|case| case.id.as_str().starts_with("age.security."))
            .all(|case| !case.expects_error));
        // AGE-specific restrictions on valid openCypher/GQL keep row
        // expectations: re-binding with an extra type is a conjunctive
        // predicate, and shortest-path errors come from AGE's SQL functions.
        assert!(!case("age.cypher.match.query-100").expects_error);
        assert!(!case("age.age.shortest.path.query-117").expects_error);
        // Genuinely invalid Cypher stays error-expected: re-declaring a
        // bound variable in CREATE is VariableAlreadyBound in the TCK too.
        assert!(case("age.cypher.create.query-77").expects_error);
    }

    #[test]
    fn imports_every_dollar_quoted_cypher_invocation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/age/sql");
        let corpus = AgeCorpus::load(root).unwrap();
        // Postgres/AGE-specific files stay excluded; EXPLAIN queries run
        // through core's EXPLAIN QUERY PLAN.
        assert_eq!(corpus.stats().files, 45);
        assert_eq!(corpus.stats().queries, corpus.cases.len());
    }
}
