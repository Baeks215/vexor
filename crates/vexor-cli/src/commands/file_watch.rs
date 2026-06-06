use crate::commands::{bench, compile};
use notify_debouncer_mini::new_debouncer;
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    sync::mpsc,
    time::Duration,
};
use vexor_compiler::SvgExport;

const WATCH_DEBOUNCE_MS: u64 = 200;

/// Watch `path` for content changes, compiling it on startup and on every
/// change, and hand the result to `on_compile`.
/// Blocks indefinitely
pub fn watch_file(
    path: &Path,
    stats: bool,
    mut on_compile: impl FnMut(Result<SvgExport, String>, Option<&bench::BenchReport>),
) -> notify::Result<()> {
    let path = path.canonicalize()?;

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(WATCH_DEBOUNCE_MS), tx)?;

    // Watches the parent directory rather than the file itself: editors like Vim
    // save atomically (write a temp file, rename it over the original)
    let watch_dir = path.parent().unwrap_or(&path);
    debouncer
        .watcher()
        .watch(watch_dir, notify::RecursiveMode::NonRecursive)?;

    // Hash file contents to prevent redundant compilation
    let mut last_hash = get_file_hash(&path);
    let (res, report) = compile::compile_file(&path, stats);
    on_compile(res, report.as_ref());
    println!("--- Watching {} for changes ---", path.display());

    for res in rx {
        match res {
            // Only react to events for the target file; the parent dir also
            // reports changes to unrelated siblings.
            Ok(events) => {
                if events.iter().any(|e| e.path == path) {
                    let current_hash = get_file_hash(&path);
                    if current_hash != last_hash {
                        last_hash = current_hash;
                        println!("\n--- Input changed, Re-compiling ---");
                        let (res, report) = compile::compile_file(&path, stats);
                        on_compile(res, report.as_ref());
                    }
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

fn get_file_hash(path: &Path) -> Option<u64> {
    let contents = fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Some(hasher.finish())
}
