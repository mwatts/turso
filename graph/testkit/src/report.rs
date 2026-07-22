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

    append_latest_corpus_histogram(&mut report, records);

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

        if latest.len() > 500 {
            append_large_suite_summary(&mut report, latest);
        } else if suite.starts_with("performance-") {
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

fn append_latest_corpus_histogram(report: &mut String, records: &[ResultRecord]) {
    let Some(run_id) = records
        .iter()
        .filter(|record| record.run_id.ends_with("corpus-deep"))
        .map(|record| record.run_id.as_str())
        .max()
    else {
        return;
    };
    let run = records
        .iter()
        .filter(|record| record.run_id == run_id)
        .collect::<Vec<_>>();
    let passed = run
        .iter()
        .filter(|record| record.outcome == Outcome::Passed)
        .count();
    let unsupported = run
        .iter()
        .filter(|record| record.outcome == Outcome::Unsupported)
        .count();
    let failures = run
        .iter()
        .filter(|record| record.outcome == Outcome::Failed)
        .collect::<Vec<_>>();
    let mut histogram = BTreeMap::<String, usize>::new();
    for record in &failures {
        *histogram.entry(failure_family(record)).or_default() += 1;
    }
    let mut histogram = histogram.into_iter().collect::<Vec<_>>();
    histogram.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    report.push_str(&format!(
        "## Latest complete corpus run\n\n- Run: `{run_id}`\n- Records: {}\n- Passed: {passed}\n- Unsupported: {unsupported}\n- Failed: {}\n\n### Failure-reason histogram\n\n| Failure family | Count |\n|---|---:|\n",
        run.len(),
        failures.len()
    ));
    for (family, count) in histogram {
        report.push_str(&format!("| {family} | {count} |\n"));
    }
    report.push('\n');
}

fn failure_family(record: &ResultRecord) -> String {
    let boundary = record
        .dimensions
        .get("execution")
        .map(String::as_str)
        .unwrap_or("unknown");
    let message = record.message.as_deref().unwrap_or_default();
    let reason = if record.expectation == crate::model::Expectation::Unsupported
        && record
            .dimensions
            .contains_key("vendor_unsupported_function")
    {
        "expected vendor-unsupported function"
    } else if boundary == "parser" {
        if message.starts_with("expected clause at byte 0") {
            "unsupported starting clause"
        } else if message.contains("comparison_op")
            || message.contains("additive_op")
            || message.contains("multiplicative_op")
            || message.contains("property_suffix")
        {
            "expression/operator continuation grammar"
        } else if message.contains("projection_items")
            || message.contains("DISTINCT or primary_expression")
            || message.contains("AS,")
        {
            "projection/expression item grammar"
        } else if message.contains("node_pattern")
            || message.contains("relationship_body")
            || message.contains("relationship_pattern")
        {
            "graph-pattern grammar"
        } else if message.contains("map_literal") {
            "map-literal grammar"
        } else {
            "other grammar"
        }
    } else if message.contains("unknown label") {
        "fixture schema missing label"
    } else if message.contains("unknown relationship type") {
        "fixture schema missing relationship type"
    } else if message.contains("unknown property") {
        "fixture schema missing property"
    } else if message.contains("unknown parameter")
        || (message.contains("parameter `") && message.contains("unsupported"))
    {
        "parameter binding/declaration"
    } else if message.contains("no such function") {
        "runtime scalar function missing"
    } else if message.contains("query produced no plan") {
        "standalone projection has no input plan"
    } else if message.contains("projection clauses in mutation queries is not supported") {
        "mutation projection unsupported"
    } else if message.contains("mutation clauses in read queries is not supported") {
        "mutation operation unsupported"
    } else if message.contains("no such column") {
        "generated SQL references missing column"
    } else if message.contains("side-effect comparison is not implemented") {
        "side-effect oracle missing"
    } else if message.contains("result expectation is not representable") {
        "result oracle missing"
    } else if message.contains("expected an error but execution succeeded") {
        "expected-error mismatch"
    } else {
        "other"
    };
    format!("`{boundary}`: {reason}")
}

fn append_large_suite_summary(report: &mut String, latest: &[&ResultRecord]) {
    let mut by_area = BTreeMap::<(&str, &'static str), usize>::new();
    let mut by_execution = BTreeMap::<(&str, &'static str), usize>::new();
    for record in latest {
        let outcome = outcome_name(record.outcome);
        *by_area.entry((&record.area, outcome)).or_default() += 1;
        let execution = record
            .dimensions
            .get("execution")
            .map(String::as_str)
            .unwrap_or("unknown");
        *by_execution.entry((execution, outcome)).or_default() += 1;
    }
    report.push_str("### Results by source area\n\n| Area | Outcome | Count |\n|---|---|---:|\n");
    for ((area, outcome), count) in by_area {
        report.push_str(&format!("| {area} | `{outcome}` | {count} |\n"));
    }
    report.push_str(
        "\n### Results by execution boundary\n\n| Boundary | Outcome | Count |\n|---|---|---:|\n",
    );
    for ((execution, outcome), count) in by_execution {
        report.push_str(&format!("| `{execution}` | `{outcome}` | {count} |\n"));
    }
    let failures = latest
        .iter()
        .filter(|record| {
            matches!(
                record.outcome,
                Outcome::Failed | Outcome::UnexpectedlySupported
            )
        })
        .collect::<Vec<_>>();
    report.push_str(&format!("\n### Failures ({})\n\n", failures.len()));
    if failures.is_empty() {
        report.push_str("- None.\n");
    } else {
        for record in failures {
            let diagnostic = record
                .message
                .as_deref()
                .unwrap_or("no diagnostic")
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n");
            report.push_str(&format!("- `{}`: {}\n", record.test_id, diagnostic));
        }
    }
}

fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Passed => "passed",
        Outcome::Failed => "failed",
        Outcome::Unsupported => "unsupported",
        Outcome::UnexpectedlySupported => "unexpectedly-supported",
        Outcome::ResourceExhausted => "resource-exhausted",
    }
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

    #[test]
    fn report_includes_complete_corpus_failure_histogram() {
        let mut record = test_record("run-corpus-deep");
        record.outcome = Outcome::Failed;
        record.message = Some("query produced no plan".to_owned());
        record
            .dimensions
            .insert("execution".to_owned(), "execution".to_owned());

        let report = render(&[record]);

        assert!(report.contains("Latest complete corpus run"));
        assert!(report.contains("`execution`: standalone projection has no input plan | 1"));
    }

    #[test]
    fn report_separates_vendor_unsupported_functions_from_stdlib_gaps() {
        let mut record = test_record("run-corpus-deep");
        record.outcome = Outcome::Unsupported;
        record.expectation = crate::model::Expectation::Unsupported;
        record.message = Some("Parse error: no such function: vertex_stats".to_owned());
        record
            .dimensions
            .insert("execution".to_owned(), "execution".to_owned());
        record.dimensions.insert(
            "vendor_unsupported_function".to_owned(),
            "vertex_stats".to_owned(),
        );

        let report = render(&[record]);

        assert!(report.contains("Unsupported: 1"));
        assert!(report.contains("Failed: 0"));
        assert!(!report.contains("expected vendor-unsupported function | 1"));
        assert!(!report.contains("runtime scalar function missing | 1"));
    }
}
