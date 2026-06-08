use stats_alloc::{Region, Stats};
use std::fmt;
use std::time::{Duration, Instant};

/// Time + memory captured around a single compile.
pub struct BenchReport {
    pub duration: Duration,
    pub stats: Stats,
}

/// measure wrapper around a closure. Only call this when stats are requested;
/// otherwise run the closure directly so no timing or allocation tracking runs.
///
/// Note: `stats_alloc` measures the process-global allocator across all
/// threads. So concurrent threads can inflate the reported memory. Time is unaffected.
pub fn measure<T>(f: impl FnOnce() -> T) -> (T, BenchReport) {
    let reg = Region::new(crate::GLOBAL);
    let start = Instant::now();
    let out = f();
    let duration = start.elapsed();
    (
        out,
        BenchReport {
            duration,
            stats: reg.change(),
        },
    )
}

/// Renders the captured time and memory usage for a successful compile.
impl fmt::Display for BenchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  time:             {:.3?}", self.duration)?;
        writeln!(f, "  allocations:      {}", self.stats.allocations)?;
        write!(
            f,
            "  bytes allocated:  {}",
            format_bytes(self.stats.bytes_allocated)
        )
    }
}

/// Format a byte count as B / KiB / MiB / GiB.
fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    let b = bytes as f64;
    if b < KIB {
        format!("{bytes} B")
    } else if b < KIB * KIB {
        format!("{:.1} KiB", b / KIB)
    } else if b < KIB * KIB * KIB {
        format!("{:.1} MiB", b / (KIB * KIB))
    } else {
        format!("{:.1} GiB", b / (KIB * KIB * KIB))
    }
}
