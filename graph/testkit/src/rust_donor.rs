use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use syn::{
    visit::{self, Visit},
    Expr, ExprMethodCall, ItemFn, Lit, Local, Pat,
};
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

#[derive(Clone, Copy, Debug)]
pub struct RustDonorSource {
    pub suite: &'static str,
    pub name: &'static str,
    pub repository: &'static str,
    pub revision: &'static str,
    pub upstream_path: &'static str,
    pub license: &'static str,
}

pub const SPARROWDB: RustDonorSource = RustDonorSource {
    suite: "sparrowdb-deep",
    name: "SparrowDB",
    repository: "https://github.com/ryaker/SparrowDB",
    revision: "82d85b7a861dfb2e127452ed89eebbcee74bfef0",
    upstream_path: "crates/sparrowdb/tests",
    license: "MIT",
};

pub const CQLITE: RustDonorSource = RustDonorSource {
    suite: "cqlite-deep",
    name: "CQLite",
    repository: "https://github.com/cqlite/cqlite",
    revision: "e2b677e8429a4cb0ead087ffbd9195f4f3999819",
    upstream_path: "tests",
    license: "MIT",
};

/// SparrowDB and CQLite cases that exercise surfaces outside openCypher and
/// GQL rather than genuine parser gaps: storage-admin commands
/// (`CHECKPOINT`/`OPTIMIZE`), legacy Neo4j 3.x index/constraint DDL
/// (`CREATE INDEX ON :Label(prop)`, `CREATE CONSTRAINT ... ASSERT`),
/// deliberately malformed syntax used to test error messages, and CQLite's
/// own dialect relaxations:
///
/// - CQLite writes single-dash relationship shorthand (`(a) -> (b)`,
///   `(a) <- (b)`, `(a) - (b)`) where openCypher/GQL require a doubled dash
///   (`-->`, `<--`, `--`); the grammar's `<arrow line>` appears twice around
///   the optional bracket, so the minimum directed form is three characters.
/// - CQLite also accepts a bare `WHERE ...` query with no preceding
///   `MATCH`/`WITH`; the grammar only ever attaches `<where clause>` to
///   `WITH`, `YIELD`, or a pattern, never as a standalone top-level clause.
const NON_GQL_CASES: &[&str] = &[
    // acceptance.rs, match_after_create.rs, readtx_query.rs, regression_379.rs,
    // spa_189_checkpoint_optimize.rs, spa_200_batch_hop_perf.rs: CHECKPOINT
    // and OPTIMIZE are storage-admin commands, not Cypher.
    "sparrowdb.acceptance.check-4-checkpoint-optimize-no-error.query-3",
    "sparrowdb.acceptance.check-4-checkpoint-optimize-no-error.query-4",
    "sparrowdb.match-after-create.match-finds-all-nodes-after-wal-only-creates.query-1",
    "sparrowdb.readtx-query.readtx-query-rejects-checkpoint.query-1",
    "sparrowdb.regression-379.detach-delete-after-checkpoint.query-4",
    "sparrowdb.spa-189-checkpoint-optimize.checkpoint-command-runs-without-error.query-1",
    "sparrowdb.spa-189-checkpoint-optimize.checkpoint-command-runs-after-writes.query-3",
    "sparrowdb.spa-189-checkpoint-optimize.optimize-command-runs-without-error.query-1",
    "sparrowdb.spa-189-checkpoint-optimize.optimize-command-runs-after-writes.query-3",
    "sparrowdb.spa-200-batch-hop-perf.two-hop-returns-valid-names.query-2",
    // spa_151_kms_query_validation.rs, spa_235_234_create_index_constraint.rs,
    // spa_306_constraint_persistence.rs, vector_index.rs: legacy Neo4j 3.x
    // `CREATE INDEX ON :Label(prop)` / `CREATE CONSTRAINT ... ASSERT` and
    // vector index DDL, none of which are openCypher or GQL.
    "sparrowdb.spa-151-kms-query-validation.kms-q33-create-unique-constraint.query-1",
    "sparrowdb.spa-151-kms-query-validation.kms-q34-create-property-index.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.create-index-supports-equality-lookup.query-3",
    "sparrowdb.spa-235-234-create-index-constraint.create-index-on-missing-label-is-noop.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.unique-constraint-allows-first-insert.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.unique-constraint-rejects-duplicate.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.unique-constraint-is-label-scoped.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.unique-constraint-rejects-duplicate-within-same-statement.query-1",
    "sparrowdb.spa-235-234-create-index-constraint.unique-constraint-allows-different-values.query-1",
    "sparrowdb.spa-306-constraint-persistence.unique-constraint-persists-across-reopen.query-1",
    "sparrowdb.spa-306-constraint-persistence.multiple-constraints-persist.query-1",
    "sparrowdb.spa-306-constraint-persistence.multiple-constraints-persist.query-2",
    "sparrowdb.vector-index.create-vector-index-ddl.query-1",
    "sparrowdb.vector-index.create-vector-index-ddl.query-2",
    // spa_243_create_entity.rs, spa_244_mcp_errors.rs,
    // spa_265_backtick_escaping.rs: deliberately malformed syntax exercised
    // to test error-message quality, not valid under any Cypher/GQL grammar.
    "sparrowdb.spa-243-create-entity.spa243-empty-class-name-returns-descriptive-error.query-1",
    "sparrowdb.spa-244-mcp-errors.spa244-empty-query-returns-meaningful-error.query-1",
    "sparrowdb.spa-244-mcp-errors.spa244-syntax-error-returns-meaningful-error.query-1",
    "sparrowdb.spa-265-backtick-escaping.unterminated-backtick-is-error.query-1",
    // CQLite: single-dash relationship shorthand instead of `-->`/`<--`/`--`.
    "cqlite.basic-queries.match-multiple-edges.query-2",
    "cqlite.delete-queries.delete-edge.query-2",
    "cqlite.delete-queries.delete-edge.query-4",
    "cqlite.match-queries.match-single-path.query-1",
    "cqlite.match-queries.match-path-with-multiple-clauses.query-1",
    "cqlite.match-queries.match-long-path.query-1",
    "cqlite.match-queries-where.match-long-path-with-id-constraint.query-1",
    "cqlite.match-queries-where.match-long-path-with-id-constraint.query-2",
    "cqlite.match-queries-where.match-short-path-with-id-constraint.query-1",
    // CQLite: bare `WHERE ...` query with no preceding MATCH/WITH.
    "cqlite.where-conditions.where-a-and-b.query-1",
    "cqlite.where-conditions.where-a-or-b.query-1",
    "cqlite.where-conditions.where-a.query-1",
    "cqlite.where-conditions.where-not-a.query-1",
];

#[derive(Debug, Error)]
pub enum RustDonorError {
    #[error("failed to read donor directory {path}: {source}")]
    ReadDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read donor source {path}: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse donor Rust source {path}: {source}")]
    ParseFile { path: String, source: syn::Error },
    #[error("donor query identity is invalid: {0}")]
    Identity(#[from] crate::identity::TestIdError),
    #[error("donor query identity is duplicated: {0}")]
    DuplicateIdentity(TestId),
}

#[derive(Clone, Debug)]
pub struct RustDonorCase {
    pub id: TestId,
    pub source_path: String,
    pub function: String,
    pub method: String,
    pub query: String,
    pub semantic_fingerprint: String,
    semantic_key: String,
}

#[derive(Debug)]
pub struct RustDonorCorpus {
    pub source: RustDonorSource,
    pub cases: Vec<RustDonorCase>,
    pub file_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustDonorStats {
    pub files: usize,
    pub queries: usize,
    pub canonical: usize,
    pub duplicates: usize,
}

impl RustDonorCorpus {
    pub fn load(root: impl AsRef<Path>, source: RustDonorSource) -> Result<Self, RustDonorError> {
        let root = root.as_ref();
        let mut paths = Vec::new();
        collect_rust_files(root, &mut paths)?;
        paths.sort();
        let mut identities = HashSet::new();
        let mut cases = Vec::new();
        for path in &paths {
            let content = fs::read_to_string(path).map_err(|source| RustDonorError::ReadFile {
                path: path.display().to_string(),
                source,
            })?;
            let syntax = syn::parse_file(&content).map_err(|source| RustDonorError::ParseFile {
                path: path.display().to_string(),
                source,
            })?;
            let relative = path
                .strip_prefix(root)
                .expect("collected paths are rooted in the corpus");
            let source_path = relative.to_string_lossy().replace('\\', "/");
            for item in &syntax.items {
                let syn::Item::Fn(function) = item else {
                    continue;
                };
                let extracted = extract_function(function);
                for (index, query) in extracted.into_iter().enumerate() {
                    let id = TestId::parse(format!(
                        "{}.{}.{}.query-{}",
                        source.suite.trim_end_matches("-deep"),
                        normalize_identifier(
                            relative.with_extension("").to_string_lossy().as_ref()
                        ),
                        normalize_identifier(&function.sig.ident.to_string()),
                        index + 1
                    ))?;
                    if !identities.insert(id.clone()) {
                        return Err(RustDonorError::DuplicateIdentity(id));
                    }
                    let semantic_key = normalize_query(&query.query);
                    cases.push(RustDonorCase {
                        id,
                        source_path: source_path.clone(),
                        function: function.sig.ident.to_string(),
                        method: query.method,
                        query: query.query,
                        semantic_fingerprint: fingerprint(&semantic_key),
                        semantic_key,
                    });
                }
            }
        }
        cases.retain(|case| !NON_GQL_CASES.contains(&case.id.as_str()));

        Ok(Self {
            source,
            cases,
            file_count: paths.len(),
        })
    }

    pub fn stats(&self) -> RustDonorStats {
        let canonical = self
            .cases
            .iter()
            .map(|case| case.semantic_key.as_str())
            .collect::<HashSet<_>>()
            .len();
        RustDonorStats {
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
        let mut records = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            let started = Instant::now();
            let (outcome, message, execution) = match parse_cache.parse(&case.query) {
                Ok(()) => match empty_fixture(case.id.as_str()) {
                    Ok(fixture) => match fixture
                        .session
                        .query(&case.query, &MutationParameters::new())
                    {
                        Ok(_) => (Outcome::Passed, None, "execution"),
                        // Mirror the TCK statement router: statements the
                        // read pipeline rejects may still be executable
                        // mutations.
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
            let record = self.record(
                case,
                environment.clone(),
                run_id,
                started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                outcome,
                message,
                execution,
            );
            records.push(record);
        }
        records
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "record construction keeps source fields explicit"
    )]
    fn record(
        &self,
        case: &RustDonorCase,
        environment: RunEnvironment,
        run_id: &str,
        duration_ns: u64,
        outcome: Outcome,
        message: Option<String>,
        execution: &str,
    ) -> ResultRecord {
        let dimensions = BTreeMap::from([
            (
                "semantic_fingerprint".to_owned(),
                case.semantic_fingerprint.clone(),
            ),
            ("execution".to_owned(), execution.to_owned()),
            ("method".to_owned(), case.method.clone()),
        ]);
        ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
            run_id: run_id.to_owned(),
            recorded_at: recorded_at(),
            environment,
            suite: self.source.suite.to_owned(),
            test_id: case.id.clone(),
            kind: TestKind::Conformance,
            area: case.source_path.trim_end_matches(".rs").to_owned(),
            fixture: case.function.clone(),
            expectation: Expectation::Rows,
            outcome,
            duration_ns,
            source: SourceIdentity {
                name: self.source.name.to_owned(),
                repository: self.source.repository.to_owned(),
                revision: self.source.revision.to_owned(),
                path: format!("{}/{}", self.source.upstream_path, case.source_path),
                case: format!("{} call in {}", case.method, case.function),
                license: self.source.license.to_owned(),
                adaptation: "rust-ast-literal-query-extraction".to_owned(),
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
            dimensions,
        }
    }
}

#[derive(Debug)]
struct ExtractedQuery {
    method: String,
    query: String,
}

#[derive(Default)]
struct QueryVisitor {
    bindings: HashMap<String, String>,
    queries: Vec<ExtractedQuery>,
}

impl<'ast> Visit<'ast> for QueryVisitor {
    fn visit_local(&mut self, local: &'ast Local) {
        if let (Pat::Ident(pattern), Some(initializer)) = (&local.pat, &local.init) {
            if let Some(value) = resolve_string(&initializer.expr, &self.bindings) {
                self.bindings.insert(pattern.ident.to_string(), value);
            }
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if matches!(
            method.as_str(),
            "execute" | "query" | "prepare" | "execute_query"
        ) {
            if let Some(query) = call
                .args
                .first()
                .and_then(|argument| resolve_string(argument, &self.bindings))
            {
                self.queries.push(ExtractedQuery { method, query });
            }
        }
        visit::visit_expr_method_call(self, call);
    }
}

fn extract_function(function: &ItemFn) -> Vec<ExtractedQuery> {
    let mut visitor = QueryVisitor::default();
    visitor.visit_block(&function.block);
    visitor.queries
}

fn resolve_string(expression: &Expr, bindings: &HashMap<String, String>) -> Option<String> {
    match expression {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) => Some(value.value()),
            _ => None,
        },
        Expr::Reference(reference) => resolve_string(&reference.expr, bindings),
        Expr::Paren(paren) => resolve_string(&paren.expr, bindings),
        Expr::Path(path) if path.path.segments.len() == 1 => bindings
            .get(&path.path.segments[0].ident.to_string())
            .cloned(),
        _ => None,
    }
}

fn collect_rust_files(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), RustDonorError> {
    let entries = fs::read_dir(directory).map_err(|source| RustDonorError::ReadDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| RustDonorError::ReadDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
    Ok(())
}

fn normalize_query(query: &str) -> String {
    query
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn fingerprint(value: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_direct_and_locally_bound_query_literals_only() {
        let function: ItemFn = syn::parse_quote! {
            fn donor_test() {
                let query = "MATCH (n) RETURN n";
                let expected = "not a query argument";
                db.execute("CREATE (:N)");
                db.query(&query);
                assert_eq!(actual, expected);
            }
        };
        let queries = extract_function(&function);
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].query, "CREATE (:N)");
        assert_eq!(queries[1].query, "MATCH (n) RETURN n");
    }

    #[test]
    fn vendored_rust_donor_inventory_is_complete() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/donors");
        let sparrowdb = RustDonorCorpus::load(root.join("sparrowdb/tests"), SPARROWDB).unwrap();
        let cqlite = RustDonorCorpus::load(root.join("cqlite/tests"), CQLITE).unwrap();
        assert_eq!(sparrowdb.stats().files, 161);
        assert_eq!(sparrowdb.stats().queries, 2_225);
        assert_eq!(cqlite.stats().files, 12);
        assert_eq!(cqlite.stats().queries, 124);
    }
}
