use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use regex::Regex;
use thiserror::Error;
use turso_graph_frontend::Parameters;

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
    /// AGE-only administrative or catalog function exercised by this query.
    /// These queries remain executable so newly added support is detected.
    pub vendor_unsupported_function: Option<&'static str>,
    semantic_key: String,
}

impl AgeCase {
    fn expectation(&self) -> Expectation {
        if self.vendor_unsupported_function.is_some() {
            Expectation::Unsupported
        } else if self.expects_error {
            Expectation::Error
        } else {
            Expectation::Rows
        }
    }
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
/// trigram similarity and fuzzy-string-match functions layered on top of
/// postgres, not Cypher parsing. These stay out of the conformance corpus
/// entirely.
const AGE_SPECIFIC_FILES: [&str; 2] = ["pg_trgm.sql", "fuzzystrmatch.sql"];

/// Individual cypher invocations that exercise AGE/postgres-specific syntax
/// rather than openCypher or GQL, or that are deliberately malformed to test
/// error handling. Excluded case by case (rather than by whole file) because
/// their donor files otherwise contain genuine openCypher coverage:
///
/// - `<label> ={...}` / `= $param` node and relationship property filters are
///   AGE's `age.enable_containment` extension (see `cypher_match.sql`'s
///   `test_enable_containment` graph), not part of the openCypher or GQL
///   property-specification grammar, which never places `=` before a map
///   literal or parameter.
/// - postgres-style `EXPLAIN (VERBOSE, COSTS OFF)`/`EXPLAIN (costs off)`
///   embed postgres `EXPLAIN` options inside the cypher() body; neither
///   openCypher nor GQL define an EXPLAIN clause at all, let alone one with
///   parenthesized planner options.
/// - `reduce(s, x IN [1, 2] | s + x)` omits the required
///   `<binding variable> = <value expression>` accumulator initialization;
///   AGE itself rejects it with a syntax error ("missing \", var IN list\"").
/// - `toIntegerList(32[])` and the unterminated string literal in `scan.sql`
///   are deliberately malformed syntax used to test error messages.
const NON_GQL_CASES: &[&str] = &[
    // cypher_match.sql: AGE containment `={...}`/`= $param` property filters.
    "age.cypher.match.query-354",
    "age.cypher.match.query-355",
    "age.cypher.match.query-356",
    "age.cypher.match.query-357",
    "age.cypher.match.query-358",
    "age.cypher.match.query-359",
    "age.cypher.match.query-360",
    "age.cypher.match.query-361",
    "age.cypher.match.query-362",
    "age.cypher.match.query-363",
    "age.cypher.match.query-364",
    "age.cypher.match.query-365",
    "age.cypher.match.query-366",
    "age.cypher.match.query-367",
    "age.cypher.match.query-369",
    "age.cypher.match.query-370",
    "age.cypher.match.query-371",
    "age.cypher.match.query-372",
    "age.cypher.match.query-373",
    "age.cypher.match.query-374",
    "age.cypher.match.query-375",
    "age.cypher.match.query-376",
    "age.cypher.match.query-377",
    "age.cypher.match.query-378",
    "age.cypher.match.query-379",
    "age.cypher.match.query-380",
    "age.cypher.match.query-381",
    "age.cypher.match.query-382",
    "age.cypher.match.query-383",
    "age.cypher.match.query-384",
    "age.cypher.match.query-386",
    "age.cypher.match.query-387",
    "age.cypher.match.query-399",
    "age.cypher.match.query-400",
    "age.cypher.match.query-401",
    // expr.sql: postgres EXPLAIN options and a deliberately malformed literal.
    "age.expr.query-460",
    "age.expr.query-1053",
    // list_comprehension.sql: AGE containment `={...}` property filters.
    "age.list.comprehension.query-101",
    "age.list.comprehension.query-102",
    "age.list.comprehension.query-104",
    "age.list.comprehension.query-105",
    // scan.sql: deliberately unterminated string literal.
    "age.scan.query-46",
    // age_reduce.sql: reduce() missing its required accumulator init.
    "age.age.reduce.query-70",
    // pgvector.sql: postgres `OPERATOR(schema.op)` explicit-operator-
    // invocation syntax around pgvector distance operators (`<->`, `<#>`,
    // `<=>`, `<+>`); `OPERATOR(...)` is a postgres SQL construct with no
    // counterpart in openCypher or GQL expression grammar. The file's other
    // `::vector` cast queries stay in the corpus (see the `::` note above).
    "age.pgvector.query-32",
    "age.pgvector.query-35",
    "age.pgvector.query-36",
    "age.pgvector.query-37",
    "age.pgvector.query-38",
    "age.pgvector.query-39",
    "age.pgvector.query-40",
    "age.pgvector.query-41",
    "age.pgvector.query-42",
    "age.pgvector.query-43",
    "age.pgvector.query-44",
    "age.pgvector.query-45",
    "age.pgvector.query-46",
    "age.pgvector.query-47",
    "age.pgvector.query-48",
    "age.pgvector.query-49",
    "age.pgvector.query-50",
    "age.pgvector.query-51",
    "age.pgvector.query-52",
    "age.pgvector.query-62",
    "age.pgvector.query-64",
    "age.pgvector.query-69",
];

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
        let vendor_function = Regex::new(
            r"(?i)\b(vertex_stats|delete_global_graphs|graph_stats|is_valid_label_name)\s*\(",
        )
        .expect("static AGE vendor-function regex is valid");
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
                let vendor_unsupported_function = vendor_function
                    .captures(&query)
                    .and_then(|captures| captures.get(1))
                    .and_then(|name| canonical_vendor_function(name.as_str()));
                cases.push(AgeCase {
                    id,
                    source_path: source_path.clone(),
                    source_line: content[..whole.start()].lines().count() + 1,
                    query,
                    graph_argument,
                    semantic_fingerprint: fingerprint(&semantic_key),
                    expects_error,
                    vendor_unsupported_function,
                    semantic_key,
                });
            }
        }
        cases.retain(|case| !NON_GQL_CASES.contains(&case.id.as_str()));
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
    let expectation = case.expectation();
    // Rows whose expected AGE output is an ERROR pass by erroring: rejecting
    // the query anywhere in the pipeline matches AGE's observable behavior.
    // Vendor-unsupported rows remain red as Unsupported; successful execution
    // is a failure because the policy then requires reclassification.
    let (outcome, message, execution) = match parse_cache.parse(&case.query) {
        Ok(()) => match empty_fixture(case.id.as_str()) {
            Ok(fixture) => match fixture.session.query(&case.query, &Parameters::new()) {
                Ok(_) => {
                    let (outcome, message) = classify_execution(expectation, None);
                    (outcome, message, "execution")
                }
                // Mirror the TCK statement router: statements the read
                // pipeline rejects may still be executable mutations.
                Err(query_error) => {
                    match fixture.session.execute(&case.query, &Parameters::new()) {
                        Ok(_) => {
                            let (outcome, message) = classify_execution(expectation, None);
                            (outcome, message, "execution")
                        }
                        Err(mutation_error) => {
                            let (outcome, message) = classify_execution(expectation, Some(format!(
                            "query execution failed: {query_error}; mutation execution failed: {mutation_error}"
                        )));
                            (outcome, message, "execution")
                        }
                    }
                }
            },
            Err(error) => (
                Outcome::Failed,
                Some(error.to_string()),
                "fixture-execution",
            ),
        },
        Err(error) => {
            let (outcome, message) = classify_execution(expectation, Some(error));
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
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: recorded_at(),
        environment,
        suite: "age-deep".to_owned(),
        test_id: case.id.clone(),
        kind: TestKind::Conformance,
        area: case.source_path.trim_end_matches(".sql").to_owned(),
        fixture: case.graph_argument.clone(),
        expectation: case.expectation(),
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
        ])
        .into_iter()
        .chain(case.vendor_unsupported_function.map(|function| {
            (
                "vendor_unsupported_function".to_owned(),
                function.to_owned(),
            )
        }))
        .collect(),
    }
}

fn classify_execution(
    expectation: Expectation,
    error: Option<String>,
) -> (Outcome, Option<String>) {
    match (expectation, error) {
        (Expectation::Rows, None) => (Outcome::Passed, None),
        (Expectation::Rows, Some(message)) => (Outcome::Failed, Some(message)),
        (Expectation::Unsupported, Some(message)) => (Outcome::Unsupported, Some(message)),
        (Expectation::Error, Some(_)) => (Outcome::Passed, None),
        (Expectation::Error, None) => (
            Outcome::Failed,
            Some("query succeeded but AGE expects an error".to_owned()),
        ),
        (Expectation::Unsupported, None) => (
            Outcome::Failed,
            Some(
                "known vendor-unsupported query succeeded and requires reclassification".to_owned(),
            ),
        ),
    }
}

fn canonical_vendor_function(name: &str) -> Option<&'static str> {
    [
        "vertex_stats",
        "delete_global_graphs",
        "graph_stats",
        "is_valid_label_name",
    ]
    .into_iter()
    .find(|candidate| name.eq_ignore_ascii_case(candidate))
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
        // Nested reduce() is valid openCypher; AGE's planner rejects it.
        || first_line.contains("not supported in a reduce")
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

    #[test]
    fn classifies_only_age_administrative_and_catalog_functions_as_vendor_unsupported() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors/age/sql");
        let corpus = AgeCorpus::load(root).unwrap();
        let case = |id: &str| {
            corpus
                .cases
                .iter()
                .find(|case| case.id.as_str() == id)
                .unwrap()
        };

        assert_eq!(
            case("age.age.global.graph.query-4").vendor_unsupported_function,
            Some("vertex_stats")
        );
        assert_eq!(
            case("age.age.global.graph.query-8").vendor_unsupported_function,
            Some("delete_global_graphs")
        );
        assert_eq!(
            case("age.age.global.graph.query-28").vendor_unsupported_function,
            Some("graph_stats")
        );
        assert_eq!(
            case("age.name.validation.query-5").vendor_unsupported_function,
            Some("is_valid_label_name")
        );

        let counts = corpus
            .cases
            .iter()
            .filter_map(|case| case.vendor_unsupported_function)
            .fold(BTreeMap::new(), |mut counts, function| {
                *counts.entry(function).or_insert(0) += 1;
                counts
            });
        assert_eq!(
            counts,
            BTreeMap::from([
                ("delete_global_graphs", 16),
                ("graph_stats", 7),
                ("is_valid_label_name", 9),
                ("vertex_stats", 21),
            ])
        );

        // Entity accessors remain portable graph work rather than AGE policy.
        assert_eq!(
            case("age.direct.field.access.query-30").vendor_unsupported_function,
            None
        );
        assert_eq!(
            case("age.direct.field.access.query-31").vendor_unsupported_function,
            None
        );

        let unsupported = case("age.age.global.graph.query-4");
        assert_eq!(unsupported.expectation(), Expectation::Unsupported);
        assert_eq!(
            classify_execution(
                unsupported.expectation(),
                Some("no such function: vertex_stats".to_owned())
            ),
            (
                Outcome::Unsupported,
                Some("no such function: vertex_stats".to_owned())
            )
        );
        assert_eq!(
            classify_execution(unsupported.expectation(), None).0,
            Outcome::Failed
        );
    }
}
