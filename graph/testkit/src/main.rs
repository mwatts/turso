use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use turso_graph_testkit::{
    age::AgeCorpus,
    grafeo::GrafeoCorpus,
    history::{append, discover_environment, new_run_id, read},
    manifest::ScenarioManifest,
    model::Outcome,
    performance::PerformanceManifest,
    query_cache::QueryParseCache,
    report,
    runner::ScenarioRunner,
    rust_donor::{RustDonorCorpus, CQLITE, SPARROWDB},
    tck::TckCorpus,
};

#[derive(Parser)]
#[command(about = "Run and report Turso graph conformance, regression, and performance history")]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        #[arg(value_enum, default_value_t = Suite::Smoke)]
        suite: Suite,
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    Performance {
        #[arg(value_enum, default_value_t = Suite::Smoke)]
        profile: Suite,
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    Tck {
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    TckStats,
    Grafeo {
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    GrafeoStats,
    Corpus {
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    CorpusStats,
    Age {
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        no_record: bool,
    },
    AgeStats,
    SparrowdbStats,
    CqliteStats,
    Report {
        #[arg(long)]
        history: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    VerifyHistory {
        #[arg(long)]
        history: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Suite {
    Smoke,
    Deep,
}

impl Suite {
    fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Deep => "deep",
        }
    }
}

fn main() -> ExitCode {
    match run(Arguments::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Arguments) -> Result<bool> {
    let root = repository_root();
    match arguments.command {
        Command::Run {
            suite,
            history,
            no_record,
        } => run_suite(&root, suite, history, no_record),
        Command::Performance {
            profile,
            history,
            no_record,
        } => run_performance(&root, profile, history, no_record),
        Command::Tck { history, no_record } => run_tck(&root, history, no_record),
        Command::TckStats => tck_stats(&root),
        Command::Grafeo { history, no_record } => run_grafeo(&root, history, no_record),
        Command::GrafeoStats => grafeo_stats(&root),
        Command::Corpus { history, no_record } => run_corpus(&root, history, no_record),
        Command::CorpusStats => corpus_stats(&root),
        Command::Age { history, no_record } => run_age(&root, history, no_record),
        Command::AgeStats => age_stats(&root),
        Command::SparrowdbStats => rust_donor_stats(&root, SPARROWDB),
        Command::CqliteStats => rust_donor_stats(&root, CQLITE),
        Command::Report { history, output } => {
            let history = history.unwrap_or_else(|| default_history(&root));
            let output = output.unwrap_or_else(|| root.join("graph/test-results/REPORT.md"));
            write_report(&history, &output)?;
            Ok(true)
        }
        Command::VerifyHistory { history } => {
            let records = read(history.unwrap_or_else(|| default_history(&root)))?;
            println!("verified {} history records", records.len());
            Ok(true)
        }
    }
}

fn corpus_stats(root: &Path) -> Result<bool> {
    let tck = tck_corpus(root)?.stats();
    let grafeo = grafeo_corpus(root)?.stats();
    let age = age_corpus(root)?.stats();
    let sparrowdb = sparrowdb_corpus(root)?.stats();
    let cqlite = cqlite_corpus(root)?.stats();
    println!(
        "source_identities={} canonical_contracts={} exact_duplicates={} tck={} grafeo={} age={} sparrowdb={} cqlite={}",
        tck.expanded + grafeo.cypher_cases + age.queries + sparrowdb.queries + cqlite.queries,
        tck.canonical + grafeo.canonical + age.canonical + sparrowdb.canonical + cqlite.canonical,
        tck.duplicates
            + grafeo.duplicates
            + age.duplicates
            + sparrowdb.duplicates
            + cqlite.duplicates,
        tck.expanded,
        grafeo.cypher_cases,
        age.queries,
        sparrowdb.queries,
        cqlite.queries
    );
    Ok(true)
}

fn age_corpus(root: &Path) -> Result<AgeCorpus> {
    AgeCorpus::load(root.join("graph/testdata/donors/age/sql")).map_err(Into::into)
}

fn sparrowdb_corpus(root: &Path) -> Result<RustDonorCorpus> {
    RustDonorCorpus::load(
        root.join("graph/testdata/donors/sparrowdb/tests"),
        SPARROWDB,
    )
    .map_err(Into::into)
}

fn cqlite_corpus(root: &Path) -> Result<RustDonorCorpus> {
    RustDonorCorpus::load(root.join("graph/testdata/donors/cqlite/tests"), CQLITE)
        .map_err(Into::into)
}

fn rust_donor_stats(
    root: &Path,
    source: turso_graph_testkit::rust_donor::RustDonorSource,
) -> Result<bool> {
    let corpus = if source.suite == SPARROWDB.suite {
        sparrowdb_corpus(root)?
    } else {
        cqlite_corpus(root)?
    };
    let stats = corpus.stats();
    println!(
        "files={} queries={} canonical={} duplicates={}",
        stats.files, stats.queries, stats.canonical, stats.duplicates
    );
    Ok(true)
}

fn age_stats(root: &Path) -> Result<bool> {
    let stats = age_corpus(root)?.stats();
    println!(
        "files={} queries={} canonical={} duplicates={}",
        stats.files, stats.queries, stats.canonical, stats.duplicates
    );
    Ok(true)
}

fn run_age(root: &Path, history: Option<PathBuf>, no_record: bool) -> Result<bool> {
    let corpus = age_corpus(root)?;
    let stats = corpus.stats();
    let environment = discover_environment("dev")?;
    let run_id = new_run_id(&environment, "age-deep");
    let mut parse_cache = QueryParseCache::default();
    let records = corpus.run_with_cache(environment, &run_id, &mut parse_cache);
    let passed = records
        .iter()
        .filter(|record| record.outcome == Outcome::Passed)
        .count();
    let failed = records.len() - passed;
    println!(
        "files={} queries={} canonical={} deduplicated={} passed={} failed={}",
        stats.files, stats.queries, stats.canonical, stats.duplicates, passed, failed
    );
    print_classifications(&records);
    print_failures(&records);
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = outcomes_are_clean(&records);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn run_corpus(root: &Path, history: Option<PathBuf>, no_record: bool) -> Result<bool> {
    let tck = tck_corpus(root)?;
    let grafeo = grafeo_corpus(root)?;
    let age = age_corpus(root)?;
    let sparrowdb = sparrowdb_corpus(root)?;
    let cqlite = cqlite_corpus(root)?;
    let environment = discover_environment("dev")?;
    let run_id = new_run_id(&environment, "corpus-deep");
    let mut parse_cache = QueryParseCache::default();
    let mut records = tck.run_with_cache(environment.clone(), &run_id, &mut parse_cache);
    records.extend(grafeo.run_with_cache(environment.clone(), &run_id, &mut parse_cache));
    records.extend(age.run_with_cache(environment.clone(), &run_id, &mut parse_cache));
    records.extend(sparrowdb.run_with_cache(environment.clone(), &run_id, &mut parse_cache));
    records.extend(cqlite.run_with_cache(environment, &run_id, &mut parse_cache));
    validate_binary_outcomes(&records)?;
    let passed = records
        .iter()
        .filter(|record| record.outcome == Outcome::Passed)
        .count();
    let unsupported = records
        .iter()
        .filter(|record| record.outcome == Outcome::Unsupported)
        .count();
    let failed = records.len() - passed - unsupported;
    let cache = parse_cache.stats();
    println!(
        "source_identities={} passed={} unsupported={} failed={} parse_requests={} unique_queries={} cross_source_intersections={}",
        records.len(),
        passed,
        unsupported,
        failed,
        cache.requests,
        cache.unique_queries,
        cache.intersections
    );
    print_classifications(&records);
    print_failures(&records);
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = outcomes_are_clean(&records);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn grafeo_corpus(root: &Path) -> Result<GrafeoCorpus> {
    GrafeoCorpus::load(root.join("graph/testdata/donors/grafeo/tests")).map_err(Into::into)
}

fn grafeo_stats(root: &Path) -> Result<bool> {
    let stats = grafeo_corpus(root)?.stats();
    println!(
        "manifests={} cypher_cases={} canonical={} duplicates={}",
        stats.manifests, stats.cypher_cases, stats.canonical, stats.duplicates
    );
    Ok(true)
}

fn run_grafeo(root: &Path, history: Option<PathBuf>, no_record: bool) -> Result<bool> {
    let corpus = grafeo_corpus(root)?;
    let stats = corpus.stats();
    let environment = discover_environment("dev")?;
    let run_id = new_run_id(&environment, "grafeo-deep");
    let records = corpus.run(environment, &run_id);
    let passed = records
        .iter()
        .filter(|record| record.outcome == Outcome::Passed)
        .count();
    let unsupported = records
        .iter()
        .filter(|record| record.outcome == Outcome::Unsupported)
        .count();
    let failed = records.len() - passed - unsupported;
    println!(
        "manifests={} cases={} canonical={} deduplicated={} passed={} unsupported={} failed={}",
        stats.manifests,
        stats.cypher_cases,
        stats.canonical,
        stats.duplicates,
        passed,
        unsupported,
        failed
    );
    print_classifications(&records);
    print_failures(&records);
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = outcomes_are_clean(&records);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn tck_corpus(root: &Path) -> Result<TckCorpus> {
    TckCorpus::load(root.join("graph/testdata/tck/opencypher/features")).map_err(Into::into)
}

fn tck_stats(root: &Path) -> Result<bool> {
    let stats = tck_corpus(root)?.stats();
    println!(
        "features={} expanded={} canonical={} duplicates={}",
        stats.features, stats.expanded, stats.canonical, stats.duplicates
    );
    Ok(true)
}

fn run_tck(root: &Path, history: Option<PathBuf>, no_record: bool) -> Result<bool> {
    let corpus = tck_corpus(root)?;
    let stats = corpus.stats();
    let environment = discover_environment("dev")?;
    let run_id = new_run_id(&environment, "tck-deep");
    let records = corpus.run(environment, &run_id);
    let passed = records
        .iter()
        .filter(|record| record.outcome == Outcome::Passed)
        .count();
    let unsupported = records
        .iter()
        .filter(|record| record.outcome == Outcome::Unsupported)
        .count();
    let failed = records.len() - passed - unsupported;
    println!(
        "features={} expanded={} canonical={} deduplicated={} passed={} unsupported={} failed={}",
        stats.features,
        stats.expanded,
        stats.canonical,
        stats.duplicates,
        passed,
        unsupported,
        failed
    );
    print_classifications(&records);
    print_failures(&records);
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = outcomes_are_clean(&records);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn print_classifications(records: &[turso_graph_testkit::model::ResultRecord]) {
    let mut classifications = std::collections::BTreeMap::<String, usize>::new();
    for record in records {
        let execution = record
            .dimensions
            .get("execution")
            .map(String::as_str)
            .unwrap_or("unknown");
        *classifications
            .entry(format!("{}:{execution}", outcome_name(record.outcome)))
            .or_default() += 1;
    }
    for (classification, count) in classifications {
        println!("{classification}={count}");
    }
}

fn print_failures(records: &[turso_graph_testkit::model::ResultRecord]) {
    for record in records.iter().filter(|record| {
        matches!(
            record.outcome,
            Outcome::Failed
                | Outcome::Unsupported
                | Outcome::UnexpectedlySupported
                | Outcome::ResourceExhausted
        )
    }) {
        println!(
            "{:?}\t{}\t{}",
            record.outcome,
            record.test_id,
            record.message.as_deref().unwrap_or_default()
        );
    }
}

fn outcomes_are_clean(records: &[turso_graph_testkit::model::ResultRecord]) -> bool {
    records
        .iter()
        .all(|record| record.outcome == Outcome::Passed)
}

fn validate_binary_outcomes(records: &[turso_graph_testkit::model::ResultRecord]) -> Result<()> {
    for record in records {
        anyhow::ensure!(
            record.dimensions.get("execution").map(String::as_str) != Some("deduplicated"),
            "{} bypassed execution through a canonical result alias",
            record.test_id
        );
        anyhow::ensure!(
            matches!(record.outcome, Outcome::Passed | Outcome::Failed),
            "{} emitted non-binary outcome {:?}",
            record.test_id,
            record.outcome
        );
        if record.outcome == Outcome::Failed {
            anyhow::ensure!(
                record
                    .message
                    .as_ref()
                    .is_some_and(|message| !message.is_empty()),
                "{} failed without a reason",
                record.test_id
            );
        }
    }
    Ok(())
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

fn run_performance(
    root: &Path,
    profile: Suite,
    history: Option<PathBuf>,
    no_record: bool,
) -> Result<bool> {
    let manifest = PerformanceManifest::load(root.join("graph/testdata/suites/performance.toml"))?;
    let environment = discover_environment("dev")?;
    let suite = format!("performance-{}", profile.as_str());
    let run_id = new_run_id(&environment, &suite);
    let records = manifest.run(profile.as_str(), environment, &run_id)?;
    for record in &records {
        println!(
            "{:?}\t{}\t{}\t{:.2}",
            record.outcome,
            record.test_id,
            record.duration_ns,
            record.throughput_per_second.unwrap_or_default()
        );
        if let Some(message) = &record.message {
            eprintln!("{message}");
        }
    }
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = records
        .iter()
        .all(|record| record.outcome == Outcome::Passed);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn run_suite(root: &Path, suite: Suite, history: Option<PathBuf>, no_record: bool) -> Result<bool> {
    let manifests = [
        root.join("graph/testdata/suites/conformance.toml"),
        root.join("graph/testdata/suites/portable.toml"),
        root.join("graph/testdata/suites/regressions.toml"),
    ];
    let mut scenarios = Vec::new();
    for path in manifests {
        let manifest =
            ScenarioManifest::load(&path).with_context(|| format!("loading {}", path.display()))?;
        scenarios.extend(
            manifest
                .scenario
                .into_iter()
                .filter(|scenario| scenario.tiers.iter().any(|tier| tier == suite.as_str())),
        );
    }
    anyhow::ensure!(
        !scenarios.is_empty(),
        "suite `{}` discovered zero scenarios",
        suite.as_str()
    );
    let environment = discover_environment("dev")?;
    let run_id = new_run_id(&environment, suite.as_str());
    let runner = ScenarioRunner::new(environment, &run_id, suite.as_str());
    let mut records = Vec::with_capacity(scenarios.len());
    for scenario in &scenarios {
        let record = runner.run(scenario)?;
        println!(
            "{:?}\t{}\t{}{}",
            record.outcome,
            record.test_id,
            record.duration_ns,
            record
                .message
                .as_deref()
                .map(|message| format!("\t{message}"))
                .unwrap_or_default()
        );
        records.push(record);
    }
    if !no_record {
        let history = history.unwrap_or_else(|| default_history(root));
        append(&history, &records)?;
        write_report(&history, &root.join("graph/test-results/REPORT.md"))?;
    }
    let clean = outcomes_are_clean(&records);
    println!("run {run_id}: {} records, clean={clean}", records.len());
    Ok(clean)
}

fn write_report(history: &Path, output: &Path) -> Result<()> {
    let records = read(history)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, report::render(&records))
        .with_context(|| format!("writing {}", output.display()))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("testkit lives under graph/testkit")
}

fn default_history(root: &Path) -> PathBuf {
    root.join("graph/test-results/history.jsonl")
}

#[cfg(test)]
mod tests {
    use super::*;
    use turso_graph_testkit::{
        identity::TestId,
        model::{
            Expectation, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
            HISTORY_SCHEMA_VERSION,
        },
    };

    fn record(outcome: Outcome) -> ResultRecord {
        ResultRecord {
            schema_version: HISTORY_SCHEMA_VERSION,
            run_id: "run".to_owned(),
            recorded_at: "2026-07-18T00:00:00Z".to_owned(),
            environment: RunEnvironment {
                git_commit: "0".repeat(40),
                git_dirty: false,
                package_version: "test".to_owned(),
                profile: "test".to_owned(),
                os: "test".to_owned(),
                architecture: "test".to_owned(),
            },
            suite: "test".to_owned(),
            test_id: TestId::parse("test.outcome").unwrap(),
            kind: TestKind::Conformance,
            area: "test".to_owned(),
            fixture: "test".to_owned(),
            expectation: Expectation::Rows,
            outcome,
            duration_ns: 0,
            source: SourceIdentity {
                name: "test".to_owned(),
                repository: "test".to_owned(),
                revision: "test".to_owned(),
                path: "test".to_owned(),
                case: "test".to_owned(),
                license: "test".to_owned(),
                adaptation: "test".to_owned(),
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
            message: None,
            dimensions: Default::default(),
        }
    }

    #[test]
    fn unsupported_outcome_keeps_conformance_run_red() {
        assert!(!outcomes_are_clean(&[record(Outcome::Unsupported)]));
    }

    #[test]
    fn only_passed_outcomes_make_conformance_run_clean() {
        assert!(outcomes_are_clean(&[record(Outcome::Passed)]));
        assert!(!outcomes_are_clean(&[
            record(Outcome::Passed),
            record(Outcome::Failed),
        ]));
    }

    #[test]
    fn binary_outcome_validation_rejects_unsupported_and_reasonless_failures() {
        assert!(validate_binary_outcomes(&[record(Outcome::Passed)]).is_ok());
        assert!(validate_binary_outcomes(&[record(Outcome::Unsupported)]).is_err());
        assert!(validate_binary_outcomes(&[record(Outcome::Failed)]).is_err());

        let mut failed = record(Outcome::Failed);
        failed.message = Some("parser rejected the query".to_owned());
        assert!(validate_binary_outcomes(&[failed]).is_ok());

        let mut aliased = record(Outcome::Passed);
        aliased
            .dimensions
            .insert("execution".to_owned(), "deduplicated".to_owned());
        assert!(validate_binary_outcomes(&[aliased]).is_err());
    }
}
