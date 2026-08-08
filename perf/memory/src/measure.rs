use serde::Serialize;
use std::path::Path;
use std::time::Instant;

const KB: f64 = 1024.0;
const MB: f64 = 1024.0 * KB;
const GB: f64 = 1024.0 * MB;

/// Told to the user whenever heap numbers are missing, so an empty column is
/// never mistaken for a measured zero.
pub const DHAT_DISABLED_HINT: &str =
    "built without the dhat allocator; rebuild with RUSTFLAGS=\"--cfg dhat_heap\"";

/// Renders an optional byte count as megabytes, or an empty CSV cell when the
/// number was never measured.
fn opt_mb(bytes: Option<u64>) -> String {
    bytes.map_or(String::new(), |b| format!("{:.2}", b as f64 / MB))
}

/// Renders an optional count, or an empty CSV cell when it was never measured.
fn opt_count(count: Option<u64>) -> String {
    count.map_or(String::new(), |c| c.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshot {
    pub rss_bytes: usize,
    pub phase: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct MemoryReport {
    /// Journal mode used (wal or mvcc)
    pub mode: String,
    /// Name of the workload profile that was executed
    pub workload: String,
    /// Number of batch iterations executed per connection
    pub iterations: usize,
    /// Number of SQL statements per transaction batch
    pub batch_size: usize,
    /// Number of concurrent connections used during the run phase
    pub connections: usize,
    /// Process RSS before any database work (includes runtime overhead)
    pub baseline_bytes: usize,
    /// Highest RSS observed across all periodic snapshots
    pub peak_bytes: usize,
    /// Process RSS at the end of the benchmark
    pub final_bytes: usize,
    /// final_bytes - baseline_bytes; net RSS growth attributable to the workload
    pub net_growth_bytes: usize,
    /// Heap bytes still allocated at measurement time (via dhat).
    /// None when the binary was built without the dhat allocator.
    pub heap_current_bytes: Option<usize>,
    /// Highest simultaneous heap allocation observed during the entire run (via dhat).
    /// None when the binary was built without the dhat allocator.
    pub heap_peak_bytes: Option<usize>,
    /// Total number of individual allocations made during the entire run (via dhat).
    /// None when the binary was built without the dhat allocator.
    pub total_allocs: Option<u64>,
    /// Cumulative bytes allocated (including already-freed); measures allocation pressure.
    /// None when the binary was built without the dhat allocator.
    pub total_bytes_allocated: Option<u64>,
    /// Time-series of RSS snapshots taken at phase transitions and periodically
    pub snapshots: Vec<MemorySnapshot>,
    /// Size of the .db file on disk after the benchmark
    pub db_file_bytes: u64,
    /// Size of the .db-wal file (WAL mode); None if absent or empty
    pub wal_file_bytes: Option<u64>,
    /// Size of the .db-log file (MVCC logical log); None if absent or empty
    pub log_file_bytes: Option<u64>,
}

pub fn take_snapshot(start: Instant, phase: &str) -> MemorySnapshot {
    let stats = memory_stats::memory_stats().expect("failed to get memory stats");
    MemorySnapshot {
        rss_bytes: stats.physical_mem,
        phase: phase.to_string(),
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

pub fn file_size(path: &str) -> u64 {
    Path::new(path).metadata().map(|m| m.len()).unwrap_or(0)
}

impl MemoryReport {
    pub fn print_human(&self) {
        println!(
            "=== MEMORY BENCHMARK ({}, {}) ===",
            self.mode, self.workload
        );
        println!(
            "Iterations:  {} x {} rows",
            self.iterations, self.batch_size
        );
        println!("Connections: {}", self.connections);

        println!();
        println!("--- RSS (process-level) ---");
        println!("Baseline:    {}", format_bytes(self.baseline_bytes));
        if self.snapshots.len() > 2 {
            for snap in &self.snapshots[1..self.snapshots.len() - 1] {
                println!(
                    "{:<12} {}  (at {}ms)",
                    format!("{}:", snap.phase),
                    format_bytes(snap.rss_bytes),
                    snap.elapsed_ms
                );
            }
        }
        println!("Peak:        {}", format_bytes(self.peak_bytes));
        println!("Final:       {}", format_bytes(self.final_bytes));
        println!("Net growth:  {}", format_bytes(self.net_growth_bytes));

        println!();
        println!("--- Heap (dhat) ---");
        match (
            self.heap_current_bytes,
            self.heap_peak_bytes,
            self.total_allocs,
            self.total_bytes_allocated,
        ) {
            (Some(current), Some(peak), Some(allocs), Some(total_bytes)) => {
                println!("Current:     {}", format_bytes(current));
                println!("Peak:        {}", format_bytes(peak));
                println!("Total allocs:  {allocs}");
                println!("Total bytes:   {}", format_bytes(total_bytes as usize));
            }
            _ => println!("unavailable: {DHAT_DISABLED_HINT}"),
        }

        println!();
        println!("--- Disk ---");
        println!("DB file:     {}", format_bytes(self.db_file_bytes as usize));
        if let Some(wal) = self.wal_file_bytes {
            println!("WAL file:    {}", format_bytes(wal as usize));
        }
        if let Some(log) = self.log_file_bytes {
            println!("Log file:    {}", format_bytes(log as usize));
        }
    }

    pub fn print_json(&self) {
        println!(
            "{}",
            serde_json::to_string_pretty(self).expect("failed to serialize report")
        );
    }

    pub fn print_csv_header() {
        println!(
            "mode,workload,iterations,batch_size,connections,baseline_mb,rss_peak_mb,rss_final_mb,rss_growth_mb,heap_current_mb,heap_peak_mb,total_allocs,total_bytes_mb,db_mb,wal_mb,log_mb"
        );
    }

    pub fn print_csv(&self) {
        println!("{}", self.csv_row());
    }

    /// Builds the CSV row. Split out from printing so a run with no heap
    /// numbers can be checked directly.
    fn csv_row(&self) -> String {
        format!(
            "{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{},{},{},{},{:.2},{:.2},{:.2}",
            self.mode,
            self.workload,
            self.iterations,
            self.batch_size,
            self.connections,
            self.baseline_bytes as f64 / MB,
            self.peak_bytes as f64 / MB,
            self.final_bytes as f64 / MB,
            self.net_growth_bytes as f64 / MB,
            opt_mb(self.heap_current_bytes.map(|b| b as u64)),
            opt_mb(self.heap_peak_bytes.map(|b| b as u64)),
            opt_count(self.total_allocs),
            opt_mb(self.total_bytes_allocated),
            self.db_file_bytes as f64 / MB,
            self.wal_file_bytes.unwrap_or(0) as f64 / MB,
            self.log_file_bytes.unwrap_or(0) as f64 / MB,
        )
    }
}

fn format_bytes(bytes: usize) -> String {
    let bytes_f = bytes as f64;
    if bytes_f >= GB {
        format!("{:.2} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.2} KB", bytes_f / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(heap: Option<u64>) -> MemoryReport {
        MemoryReport {
            mode: "wal".to_string(),
            workload: "insert-heavy".to_string(),
            iterations: 1,
            batch_size: 1,
            connections: 1,
            baseline_bytes: 0,
            peak_bytes: 0,
            final_bytes: 0,
            net_growth_bytes: 0,
            heap_current_bytes: heap.map(|h| h as usize),
            heap_peak_bytes: heap.map(|h| h as usize),
            total_allocs: heap,
            total_bytes_allocated: heap,
            snapshots: Vec::new(),
            db_file_bytes: 0,
            wal_file_bytes: None,
            log_file_bytes: None,
        }
    }

    /// A build without the dhat allocator never measured the heap at all. If
    /// those columns came out as 0.00 they would read as a run that allocated
    /// nothing, so they have to stay empty instead.
    #[test]
    fn missing_heap_numbers_are_blank_not_zero() {
        let row = report(None).csv_row();
        let columns: Vec<&str> = row.split(',').collect();

        // heap_current_mb, heap_peak_mb, total_allocs, total_bytes_mb
        assert_eq!(&columns[9..13], &["", "", "", ""], "row was: {row}");
    }

    /// A real measurement of zero is a different fact from no measurement, and
    /// has to survive as a number.
    #[test]
    fn a_measured_zero_heap_still_prints_as_zero() {
        let row = report(Some(0)).csv_row();
        let columns: Vec<&str> = row.split(',').collect();

        assert_eq!(
            &columns[9..13],
            &["0.00", "0.00", "0", "0.00"],
            "row was: {row}"
        );
    }
}
