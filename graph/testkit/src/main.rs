use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use turso_graph_testkit::{
    history::{append, discover_environment, new_run_id, read},
    manifest::ScenarioManifest,
    model::Outcome,
    performance::PerformanceManifest,
    report,
    runner::ScenarioRunner,
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
    let clean = records
        .iter()
        .all(|record| matches!(record.outcome, Outcome::Passed | Outcome::Unsupported));
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
