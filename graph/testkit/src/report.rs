use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Outcome, ResultRecord};

pub fn render(records: &[ResultRecord]) -> String {
    let mut by_suite: BTreeMap<&str, BTreeMap<&str, Vec<&ResultRecord>>> = BTreeMap::new();
    for record in records {
        by_suite
            .entry(&record.suite)
            .or_default()
            .entry(&record.run_id)
            .or_default()
            .push(record);
    }
    let mut report = String::from("# Graph test history\n\n");
    report.push_str("Generated from `graph/test-results/history.jsonl`. Results are grouped by stable test identity; performance comparisons are meaningful only for matching environment and workload dimensions.\n\n");
    if by_suite.is_empty() {
        report.push_str("No recorded runs.\n");
        return report;
    }

    for (suite, runs) in &by_suite {
        let (latest_id, latest) = runs.last_key_value().expect("suite has a run");
        let passed = latest
            .iter()
            .filter(|record| record.outcome == Outcome::Passed)
            .count();
        let unsupported = latest
            .iter()
            .filter(|record| record.outcome == Outcome::Unsupported)
            .count();
        let failed = latest.len() - passed - unsupported;
        let environment = &latest[0].environment;
        report.push_str(&format!(
            "## Latest `{suite}` run\n\n- Run: `{latest_id}`\n- Commit: `{}`{}\n- Package: `{}`\n- Environment: `{}/{}` (`{}`)\n- Records: {}\n- Passed: {passed}\n- Unsupported: {unsupported}\n- Failed or changed: {failed}\n\n",
            environment.git_commit,
            if environment.git_dirty { " (dirty)" } else { "" },
            environment.package_version,
            environment.os,
            environment.architecture,
            environment.profile,
            latest.len()
        ));

        if let Some((previous_id, previous)) = runs.iter().rev().nth(1) {
            append_changes(&mut report, previous_id, previous, latest);
            if suite.starts_with("performance-") {
                append_performance_deltas(&mut report, previous, latest);
            }
        }

        if suite.starts_with("performance-") {
            report.push_str("| Test | Operation | Scale | Outcome | Duration | Throughput/s |\n|---|---|---:|---|---:|---:|\n");
            for record in latest {
                report.push_str(&format!(
                    "| `{}` | {} | {} | `{:?}` | {:.3} ms | {:.2} |\n",
                    record.test_id,
                    record.operation.as_deref().unwrap_or("-"),
                    record.scale.unwrap_or_default(),
                    record.outcome,
                    record.duration_ns as f64 / 1_000_000.0,
                    record.throughput_per_second.unwrap_or_default()
                ));
            }
        } else {
            report
                .push_str("| Test | Kind | Area | Outcome | Duration |\n|---|---|---|---|---:|\n");
            for record in latest {
                report.push_str(&format!(
                    "| `{}` | `{:?}` | {} | `{:?}` | {:.3} ms |\n",
                    record.test_id,
                    record.kind,
                    record.area,
                    record.outcome,
                    record.duration_ns as f64 / 1_000_000.0
                ));
            }
        }
        report.push('\n');
    }

    let run_count = by_suite.values().map(BTreeMap::len).sum::<usize>();
    report.push_str(&format!(
        "## Longitudinal inventory\n\n- Runs: {run_count}\n- Result records: {}\n- Unique test identities: {}\n",
        records.len(),
        records
            .iter()
            .map(|record| record.test_id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
    ));
    report
}

fn append_changes(
    report: &mut String,
    previous_id: &str,
    previous: &[&ResultRecord],
    latest: &[&ResultRecord],
) {
    let previous_outcomes = previous
        .iter()
        .map(|record| (record.test_id.as_str(), record.outcome))
        .collect::<BTreeMap<_, _>>();
    let changes = latest
        .iter()
        .filter(|record| {
            previous_outcomes
                .get(record.test_id.as_str())
                .is_some_and(|outcome| *outcome != record.outcome)
        })
        .collect::<Vec<_>>();
    report.push_str(&format!("### Outcome changes from `{previous_id}`\n\n"));
    if changes.is_empty() {
        report.push_str("- No outcome changes.\n\n");
    } else {
        for record in changes {
            report.push_str(&format!("- `{}`: {:?}\n", record.test_id, record.outcome));
        }
        report.push('\n');
    }
}

fn append_performance_deltas(
    report: &mut String,
    previous: &[&ResultRecord],
    latest: &[&ResultRecord],
) {
    let previous_durations = previous
        .iter()
        .map(|record| (record.test_id.as_str(), record.duration_ns))
        .collect::<BTreeMap<_, _>>();
    report.push_str("### Duration changes\n\n");
    for record in latest {
        let Some(previous_ns) = previous_durations.get(record.test_id.as_str()) else {
            continue;
        };
        if *previous_ns == 0 {
            continue;
        }
        let delta = (record.duration_ns as f64 / *previous_ns as f64 - 1.0) * 100.0;
        report.push_str(&format!("- `{}`: {delta:+.1}%\n", record.test_id));
    }
    report.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::test_record;

    #[test]
    fn report_covers_latest_run_and_history() {
        let report = render(&[test_record("run-1"), test_record("run-2")]);
        assert!(report.contains("Latest `smoke` run"));
        assert!(report.contains("Outcome changes from `run-1`"));
    }
}
