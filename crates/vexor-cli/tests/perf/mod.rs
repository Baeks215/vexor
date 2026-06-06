//! Manual benchmark harness for the compiler pipeline.
//!
//! Ignored by default. Runs every `.vx` file in `tests/perf/cases`, measuring the full
//! `compile_to_svg` pipeline (parse -> eval -> scene -> SVG), and prints a table of
//! time + allocation stats. Run with:
//!
//! cargo test -p vexor-cli --test mod --release -- --ignored --nocapture bench_cases

use stats_alloc::{Region, Stats};
use std::fs;
use std::hint::black_box;
use std::time::{Duration, Instant};

/// Number of timed iterations per case (after one warmup).
const ITERATIONS: u32 = 100;

/// Outcome of benchmarking a single case.
struct Row {
    name: String,
    /// `None` if the case failed to compile.
    result: Option<Measured>,
}

struct Measured {
    min: Duration,
    mean: Duration,
    stats: Stats,
}

#[test]
#[ignore = "manual benchmark; run with --ignored --nocapture"]
fn bench_cases() {
    let dir = "tests/perf/cases";
    let mut paths: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("could not read '{dir}': {e}"))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "vx"))
        .collect();
    paths.sort();

    let mut rows: Vec<Row> = Vec::new();
    for path in paths {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("could not read '{}': {e}", path.display()));
        rows.push(Row {
            name,
            result: bench_one(&source),
        });
    }

    print_table(&rows);

    // Surface failures loudly so a broken case can't masquerade as "benchmarked".
    let failed: Vec<&str> = rows
        .iter()
        .filter(|r| r.result.is_none())
        .map(|r| r.name.as_str())
        .collect();
    assert!(failed.is_empty(), "cases failed to compile: {failed:?}");
}

/// Benchmarks a single source string, or returns `None` if it does not compile.
fn bench_one(source: &str) -> Option<Measured> {
    // Bail out early (and mark as failed) if the case does not compile.
    if vexor_compiler::compile_to_svg(source).is_err() {
        return None;
    }

    // Warmup.
    let _ = black_box(vexor_compiler::compile_to_svg(source));

    // Timing: keep the best (min) and accumulate for the mean.
    let mut min = Duration::MAX;
    let mut total = Duration::ZERO;
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let out = black_box(vexor_compiler::compile_to_svg(source));
        let elapsed = start.elapsed();
        black_box(out).ok();
        min = min.min(elapsed);
        total += elapsed;
    }
    let mean = total / ITERATIONS;

    // Memory: a single instrumented compile is deterministic.
    let reg = Region::new(crate::GLOBAL);
    let out = vexor_compiler::compile_to_svg(source);
    let stats = reg.change();
    black_box(out).ok();

    Some(Measured { min, mean, stats })
}

fn print_table(rows: &[Row]) {
    println!();
    println!(
        "{:<18} {:>12} {:>12} {:>10} {:>12}",
        "case", "time/min", "time/mean", "allocs", "bytes"
    );
    println!("{}", "-".repeat(68));
    for row in rows {
        match &row.result {
            Some(m) => println!(
                "{:<18} {:>12} {:>12} {:>10} {:>12}",
                row.name,
                format!("{:.3?}", m.min),
                format!("{:.3?}", m.mean),
                m.stats.allocations,
                format_bytes(m.stats.bytes_allocated),
            ),
            None => println!("{:<18} {:>12}", row.name, "FAILED"),
        }
    }
    println!();
}

/// Format a byte count as B / KiB / MiB / GiB. Mirrors `vexor-cli` bench output.
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
