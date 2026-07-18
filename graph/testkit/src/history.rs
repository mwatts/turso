use std::{
    collections::HashSet,
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
    fn result_digest_is_order_sensitive_and_reproducible() {
        let first = vec![vec!["a".to_owned()], vec!["b".to_owned()]];
        let second = vec![vec!["b".to_owned()], vec!["a".to_owned()]];
        assert_eq!(result_digest(&first), result_digest(&first));
        assert_ne!(result_digest(&first), result_digest(&second));
    }
}
