use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::Command,
};

use chrono::{SecondsFormat, Utc};
use thiserror::Error;

use crate::model::{ResultRecord, RunEnvironment, HISTORY_SCHEMA_VERSION};

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("history I/O failed for {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("history line {line} is invalid JSON: {source}")]
    Json {
        line: usize,
        source: serde_json::Error,
    },
    #[error("history line {line} uses unsupported schema version {version}")]
    Version { line: usize, version: u32 },
    #[error(
        "duplicate history record for run `{run_id}`, test `{test_id}`, operation {operation:?}"
    )]
    Duplicate {
        run_id: String,
        test_id: String,
        operation: Option<String>,
    },
    #[error("git command `{command}` failed")]
    Git { command: String },
}

/// The cargo profile this binary was built with.
///
/// Corpus and benchmark runs are pinned to `--release` because their timings
/// are only comparable against history when optimized. A call site that names
/// its own profile can disagree with the build; deriving it means it cannot.
/// `debug_assertions` is the proxy: no workspace profile overrides it.
pub fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    }
}

pub fn discover_environment(profile: impl Into<String>) -> Result<RunEnvironment, HistoryError> {
    let git_commit = git_output(&["rev-parse", "HEAD"])?;
    let git_dirty = !git_output(&["status", "--porcelain"])?.is_empty();
    Ok(RunEnvironment {
        git_commit,
        git_dirty,
        package_version: env!("CARGO_PKG_VERSION").to_owned(),
        profile: profile.into(),
        os: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
    })
}

pub fn new_run_id(environment: &RunEnvironment, suite: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S%.6fZ");
    let short_commit = environment
        .git_commit
        .get(..12)
        .unwrap_or(&environment.git_commit);
    format!("{timestamp}-{short_commit}-{suite}")
}

pub fn recorded_at() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)
}

pub fn append(path: impl AsRef<Path>, records: &[ResultRecord]) -> Result<(), HistoryError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_error(path, source))?;
    }
    let mut existing = read(path)?;
    existing.extend_from_slice(records);
    validate_unique(&existing)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    for record in records {
        serde_json::to_writer(&mut file, record)
            .map_err(|source| HistoryError::Json { line: 0, source })?;
        file.write_all(b"\n")
            .map_err(|source| io_error(path, source))?;
    }
    file.sync_all().map_err(|source| io_error(path, source))
}

pub fn read(path: impl AsRef<Path>) -> Result<Vec<ResultRecord>, HistoryError> {
    let path = path.as_ref();
    let file = match OpenOptions::new().read(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(io_error(path, source)),
    };
    let mut records = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|source| io_error(path, source))?;
        if line.trim().is_empty() {
            continue;
        }
        let record: ResultRecord =
            serde_json::from_str(&line).map_err(|source| HistoryError::Json {
                line: line_number,
                source,
            })?;
        if record.schema_version > HISTORY_SCHEMA_VERSION {
            return Err(HistoryError::Version {
                line: line_number,
                version: record.schema_version,
            });
        }
        records.push(record);
    }
    validate_unique(&records)?;
    Ok(records)
}

/// The report renders the newest run of each suite and diffs it against the
/// one before, so retention below this keeps a file that cannot answer the
/// question it exists to answer.
pub const MINIMUM_RETAINED_RUNS: usize = 2;

/// What `prune` did, for the caller to print before anything is swapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PruneOutcome {
    pub records_read: usize,
    pub records_written: usize,
    pub runs_kept: usize,
    pub runs_dropped: usize,
}

/// The newest `keep` run ids of every suite, floored at [`MINIMUM_RETAINED_RUNS`].
///
/// Run ids carry a timestamp prefix, so lexicographic order is recency order;
/// `report::render` already depends on that, and retention reuses it rather
/// than inventing a second rule. Counting per suite stops a suite that runs
/// hourly from evicting one that runs weekly.
pub fn retained_run_ids(records: &[ResultRecord], keep: usize) -> BTreeSet<String> {
    let keep = keep.max(MINIMUM_RETAINED_RUNS);
    let mut by_suite: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for record in records {
        by_suite
            .entry(&record.suite)
            .or_default()
            .insert(&record.run_id);
    }
    by_suite
        .into_values()
        .flat_map(|runs| {
            runs.into_iter()
                .rev()
                .take(keep)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Copy the retained runs of `source` into `target`, leaving `source` intact.
///
/// history.jsonl is gitignored and cannot be regenerated, so pruning never
/// destroys: it writes beside the source and the caller decides whether to
/// archive and swap.
pub fn prune(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    keep: usize,
) -> Result<PruneOutcome, HistoryError> {
    let records = read(source)?;
    let retained = retained_run_ids(&records, keep);
    let runs_total = records
        .iter()
        .map(|record| record.run_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let kept = records
        .iter()
        .filter(|record| retained.contains(&record.run_id))
        .cloned()
        .collect::<Vec<_>>();
    let outcome = PruneOutcome {
        records_read: records.len(),
        records_written: kept.len(),
        runs_kept: retained.len(),
        runs_dropped: runs_total - retained.len(),
    };
    let target = target.as_ref();
    if target.exists() {
        fs::remove_file(target).map_err(|source| io_error(target, source))?;
    }
    append(target, &kept)?;
    Ok(outcome)
}

pub fn result_digest(rows: &[Vec<String>]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for row in rows {
        for value in row {
            for byte in value.as_bytes().iter().copied().chain([0xff]) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash ^= 0xfe;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn validate_unique(records: &[ResultRecord]) -> Result<(), HistoryError> {
    let mut keys = HashSet::new();
    for record in records {
        let key = (
            record.run_id.clone(),
            record.test_id.clone(),
            record.operation.clone(),
        );
        if !keys.insert(key.clone()) {
            return Err(HistoryError::Duplicate {
                run_id: key.0,
                test_id: key.1.to_string(),
                operation: key.2,
            });
        }
    }
    Ok(())
}

fn git_output(arguments: &[&str]) -> Result<String, HistoryError> {
    let output = Command::new("git")
        .args(arguments)
        .output()
        .map_err(|_| HistoryError::Git {
            command: arguments.join(" "),
        })?;
    if !output.status.success() {
        return Err(HistoryError::Git {
            command: arguments.join(" "),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn io_error(path: &Path, source: std::io::Error) -> HistoryError {
    HistoryError::Io {
        path: path.display().to_string(),
        source,
    }
}

#[cfg(test)]
pub(crate) fn test_record(run_id: &str) -> ResultRecord {
    use std::collections::BTreeMap;

    use crate::{
        identity::TestId,
        model::{Expectation, Outcome, SourceIdentity, TestKind},
    };

    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: run_id.to_owned(),
        recorded_at: "2026-07-17T00:00:00Z".to_owned(),
        environment: RunEnvironment {
            git_commit: "a".repeat(40),
            git_dirty: false,
            package_version: "0.7.0".to_owned(),
            profile: "test".to_owned(),
            os: "test".to_owned(),
            architecture: "test".to_owned(),
        },
        suite: "smoke".to_owned(),
        test_id: TestId::parse("tck.return.literal").unwrap(),
        kind: TestKind::Smoke,
        area: "expression".to_owned(),
        fixture: "empty".to_owned(),
        expectation: Expectation::Rows,
        outcome: Outcome::Passed,
        duration_ns: 1,
        source: SourceIdentity {
            name: "TCK".to_owned(),
            repository: "https://github.com/opencypher/openCypher".to_owned(),
            revision: "b".repeat(40),
            path: "x.feature".to_owned(),
            case: "literal".to_owned(),
            license: "Apache-2.0".to_owned(),
            adaptation: "fixture-adaptation".to_owned(),
            issue: None,
            fixed_commit: None,
        },
        operation: None,
        graph_shape: None,
        scale: None,
        iterations: None,
        throughput_per_second: None,
        row_count: Some(1),
        node_count: None,
        relationship_count: None,
        result_digest: Some(result_digest(&[vec!["1".to_owned()]])),
        message: None,
        dimensions: BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_round_trip_is_append_only_and_rejects_duplicate_keys() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.jsonl");
        append(&path, &[test_record("run-1")]).unwrap();
        append(&path, &[test_record("run-2")]).unwrap();
        assert_eq!(read(&path).unwrap().len(), 2);
        assert!(append(&path, &[test_record("run-2")]).is_err());
        assert_eq!(read(&path).unwrap().len(), 2);
    }

    #[test]
    fn legacy_schema_version_one_rows_read_without_a_semantics_version() {
        // history.jsonl is append-only and ~1.25 GB of schema-version-1 rows.
        // Reading must never require rewriting them.
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.jsonl");
        let mut legacy = serde_json::to_value(test_record("legacy")).unwrap();
        legacy["schema_version"] = serde_json::json!(1);
        legacy
            .as_object_mut()
            .unwrap()
            .remove("semantics_version")
            .expect("current records carry the field");
        std::fs::write(&path, format!("{legacy}\n")).unwrap();

        let records = read(&path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].schema_version, 1);
        assert_eq!(
            records[0].semantics_version, 0,
            "legacy rows report 0, meaning the semantics used are unknown"
        );
    }

    #[test]
    fn current_rows_carry_the_semantic_profile_version() {
        let record = test_record("current");
        assert_eq!(record.schema_version, HISTORY_SCHEMA_VERSION);
        assert_eq!(
            record.semantics_version,
            turso_graph_ir::SEMANTIC_PROFILE_VERSION,
            "a run must record which semantic rules produced its verdicts"
        );
    }

    #[test]
    fn build_profile_comes_from_the_build_not_the_caller() {
        // A --release corpus or cypherbench run that records `profile: "dev"`
        // makes its timings silently uncomparable against history. The string
        // has to be derived, so no call site can get it wrong.
        let expected = if cfg!(debug_assertions) {
            "dev"
        } else {
            "release"
        };
        assert_eq!(build_profile(), expected);
        assert_eq!(
            discover_environment(build_profile()).unwrap().profile,
            expected
        );
    }

    fn record_in(run_id: &str, suite: &str, test: &str) -> ResultRecord {
        let mut record = test_record(run_id);
        record.suite = suite.to_owned();
        record.test_id = crate::identity::TestId::parse(test).unwrap();
        record
    }

    #[test]
    fn retention_keeps_the_newest_runs_of_every_suite_independently() {
        // Run ids are timestamp-prefixed, so lexicographic order is recency
        // order. `report::render` already relies on that; retention must not
        // invent a second rule. A rarely-run suite must not be evicted by a
        // frequently-run one, so the count is per suite.
        let records = vec![
            record_in("20260101T000000Z-aaa-corpus", "corpus", "tck.a.b.c"),
            record_in("20260102T000000Z-bbb-corpus", "corpus", "tck.a.b.c"),
            record_in("20260103T000000Z-ccc-corpus", "corpus", "tck.a.b.c"),
            record_in("20260101T000000Z-aaa-smoke", "smoke", "tck.a.b.c"),
        ];

        let retained = retained_run_ids(&records, 2);

        assert_eq!(
            retained,
            BTreeSet::from([
                "20260102T000000Z-bbb-corpus".to_owned(),
                "20260103T000000Z-ccc-corpus".to_owned(),
                "20260101T000000Z-aaa-smoke".to_owned(),
            ])
        );
    }

    #[test]
    fn retention_never_drops_below_what_the_report_reads() {
        // report::render renders the latest run and diffs it against the one
        // before, so a retention of 1 would silently empty the change table.
        let records = vec![
            record_in("20260101T000000Z-aaa-corpus", "corpus", "tck.a.b.c"),
            record_in("20260102T000000Z-bbb-corpus", "corpus", "tck.a.b.c"),
        ];

        assert_eq!(retained_run_ids(&records, 0).len(), 2);
        assert_eq!(retained_run_ids(&records, 1).len(), 2);
    }

    #[test]
    fn prune_writes_a_new_file_and_never_touches_the_source() {
        // history.jsonl is not in git and cannot be regenerated. Pruning writes
        // beside the source so the caller decides when to swap, and the caller
        // can archive the original first.
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("history.jsonl");
        let target = directory.path().join("history.pruned.jsonl");
        append(
            &source,
            &[
                record_in("20260101T000000Z-aaa-corpus", "corpus", "tck.a.b.c"),
                record_in("20260102T000000Z-bbb-corpus", "corpus", "tck.a.b.c"),
                record_in("20260103T000000Z-ccc-corpus", "corpus", "tck.a.b.c"),
            ],
        )
        .unwrap();

        let outcome = prune(&source, &target, 2).unwrap();

        assert_eq!(outcome.records_read, 3);
        assert_eq!(outcome.records_written, 2);
        assert_eq!(outcome.runs_dropped, 1);
        assert_eq!(read(&source).unwrap().len(), 3, "source is left intact");
        let pruned = read(&target).unwrap();
        assert_eq!(pruned.len(), 2);
        assert!(
            pruned
                .iter()
                .all(|record| record.run_id != "20260101T000000Z-aaa-corpus"),
            "the oldest run is the one dropped"
        );
    }

    #[test]
    fn result_digest_is_order_sensitive_and_reproducible() {
        let first = vec![vec!["a".to_owned()], vec!["b".to_owned()]];
        let second = vec![vec!["b".to_owned()], vec!["a".to_owned()]];
        assert_eq!(result_digest(&first), result_digest(&first));
        assert_ne!(result_digest(&first), result_digest(&second));
    }
}
