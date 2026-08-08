use anyhow::Result;
use clap::{Parser, ValueEnum};
use memory_benchmark::measure::{
    DHAT_DISABLED_HINT, MemoryReport, MemorySnapshot, file_size, take_snapshot,
};
use memory_benchmark::profile::Phase;
use memory_benchmark::workload::{
    JournalMode, WorkloadConfig, WorkloadObserver, WorkloadProfile, clean_db_files, run_workload,
};
use std::time::{Duration, Instant};

// A binary gets one global allocator. `turso` installs a mimalloc-backed one
// whenever its `mimalloc` feature is on, and any workspace-wide build turns
// that feature on for every crate at once -- so a dhat allocator that is
// always compiled in makes `cargo build --workspace` fail to link.
//
// This is deliberately a `cfg` flag rather than a Cargo feature: `--all-features`
// would switch a feature back on and reintroduce the clash. Profiling runs opt
// in with RUSTFLAGS="--cfg dhat_heap", which nothing else turns on by accident.
#[cfg(dhat_heap)]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Csv,
}

#[derive(Parser)]
#[command(name = "memory-benchmark")]
#[command(about = "Memory usage benchmark for Turso SQL workloads")]
struct Args {
    /// Journal mode
    #[arg(short = 'm', long = "mode", default_value = "wal")]
    mode: JournalMode,

    /// Built-in workload profile
    #[arg(short = 'w', long = "workload", default_value = "insert-heavy")]
    workload: WorkloadProfile,

    /// Number of iterations for the workload
    #[arg(short = 'i', long = "iterations", default_value = "1000")]
    iterations: usize,

    /// Batch size (rows per transaction)
    #[arg(short = 'b', long = "batch-size", default_value = "100")]
    batch_size: usize,

    /// SQLite page cache size (in pages, negative = KiB)
    #[arg(long = "cache-size")]
    cache_size: Option<i64>,

    /// Number of concurrent connections
    #[arg(long = "connections", default_value = "1")]
    connections: usize,

    /// Busy timeout in milliseconds
    #[arg(long = "timeout", default_value = "30000")]
    timeout: u64,

    /// Output format
    #[arg(long = "format", default_value = "human")]
    format: OutputFormat,

    /// Run a final checkpoint after the workload completes
    #[arg(long)]
    checkpoint: bool,

    /// MVCC only: set `PRAGMA mvcc_checkpoint_threshold` (bytes; -1 disables
    /// auto-checkpoint). Use -1 to isolate the inline-GC effect.
    #[arg(long = "mvcc-checkpoint-threshold")]
    mvcc_checkpoint_threshold: Option<i64>,

    /// MVCC only: set `PRAGMA mvcc_gc_threshold` (live-version growth per
    /// inline GC pass; -1 disables inline GC). Toggle this for A/B runs.
    #[arg(long = "mvcc-gc-threshold")]
    mvcc_gc_threshold: Option<i64>,
}

/// The dhat heap numbers for a finished run.
struct HeapTotals {
    current_bytes: usize,
    peak_bytes: usize,
    allocs: u64,
    bytes_allocated: u64,
}

/// Reads the dhat heap totals, or None when the dhat allocator was compiled
/// out. Asking dhat for stats without its allocator installed would only ever
/// report zeros, which reads exactly like a run that allocated nothing.
fn heap_totals() -> Option<HeapTotals> {
    #[cfg(dhat_heap)]
    {
        let stats = dhat::HeapStats::get();
        Some(HeapTotals {
            current_bytes: stats.curr_bytes,
            peak_bytes: stats.max_bytes,
            allocs: stats.total_blocks,
            bytes_allocated: stats.total_bytes,
        })
    }
    #[cfg(not(dhat_heap))]
    {
        None
    }
}

/// Takes RSS snapshots at phase transitions and tracks the RSS peak after
/// every batch.
struct SnapshotObserver {
    start: Instant,
    snapshots: Vec<MemorySnapshot>,
    peak_bytes: usize,
}

impl WorkloadObserver for SnapshotObserver {
    fn on_phase(&mut self, phase: Phase) {
        let label = match phase {
            Phase::Setup => "setup",
            Phase::Run => "run-start",
            Phase::Checkpoint => "checkpoint",
            Phase::Done => unreachable!(),
        };
        self.snapshots.push(take_snapshot(self.start, label));
    }

    fn after_batch(&mut self) {
        let current = take_snapshot(self.start, "periodic");
        if current.rss_bytes > self.peak_bytes {
            self.peak_bytes = current.rss_bytes;
        }
    }
}

fn main() -> Result<()> {
    #[cfg(dhat_heap)]
    let _profiler = dhat::Profiler::new_heap();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.connections.max(1))
        .build()?;

    rt.block_on(async_main(args))
}

async fn async_main(args: Args) -> Result<()> {
    let db_path = "memory_benchmark.db";
    clean_db_files(db_path);

    let cfg = WorkloadConfig {
        mode: args.mode,
        workload: args.workload,
        iterations: args.iterations,
        batch_size: args.batch_size,
        connections: args.connections,
        timeout: Duration::from_millis(args.timeout),
        cache_size: args.cache_size,
        checkpoint: args.checkpoint,
        mvcc_checkpoint_threshold: args.mvcc_checkpoint_threshold,
        mvcc_gc_threshold: args.mvcc_gc_threshold,
    };

    let start = Instant::now();

    // Baseline snapshot before any DB work
    let baseline_snapshot = take_snapshot(start, "baseline");
    let baseline = baseline_snapshot.rss_bytes;
    let mut observer = SnapshotObserver {
        start,
        snapshots: vec![baseline_snapshot],
        peak_bytes: baseline,
    };

    let workload_name = run_workload(db_path, &cfg, &mut observer).await?;

    // Final snapshot
    let final_snap = take_snapshot(start, "final");
    let peak_bytes = observer.peak_bytes.max(final_snap.rss_bytes);
    let mut snapshots = observer.snapshots;
    snapshots.push(final_snap.clone());

    let heap = heap_totals();
    if heap.is_none() {
        eprintln!("warning: heap numbers omitted -- {DHAT_DISABLED_HINT}");
    }
    let report = MemoryReport {
        mode: args.mode.to_string(),
        workload: workload_name,
        iterations: args.iterations,
        batch_size: args.batch_size,
        connections: args.connections,
        baseline_bytes: baseline,
        peak_bytes,
        final_bytes: final_snap.rss_bytes,
        net_growth_bytes: final_snap.rss_bytes.saturating_sub(baseline),
        heap_current_bytes: heap.as_ref().map(|h| h.current_bytes),
        heap_peak_bytes: heap.as_ref().map(|h| h.peak_bytes),
        total_allocs: heap.as_ref().map(|h| h.allocs),
        total_bytes_allocated: heap.as_ref().map(|h| h.bytes_allocated),
        snapshots,
        db_file_bytes: file_size(db_path),
        wal_file_bytes: {
            let wal_path = format!("{db_path}-wal");
            let size = file_size(&wal_path);
            if size > 0 { Some(size) } else { None }
        },
        log_file_bytes: {
            let log_path = format!("{db_path}-log");
            let size = file_size(&log_path);
            if size > 0 { Some(size) } else { None }
        },
    };

    match args.format {
        OutputFormat::Human => report.print_human(),
        OutputFormat::Json => report.print_json(),
        OutputFormat::Csv => {
            MemoryReport::print_csv_header();
            report.print_csv();
        }
    }

    Ok(())
}
