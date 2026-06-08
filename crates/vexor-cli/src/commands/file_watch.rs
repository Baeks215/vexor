use crate::commands::{bench, compile};
use notify::{Event, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    sync::mpsc,
};
use vexor_compiler::SvgExport;

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
    let mut watcher = recommended_watcher(tx)?;

    // Watches the parent directory rather than the file itself: editors like Vim
    // save atomically (write a temp file, rename it over the original)
    let watch_dir = path.parent().unwrap_or(&path);
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

    // Hash file contents to prevent redundant compilation
    let mut last_hash = None;
    match fs::read_to_string(&path) {
        Ok(source) => {
            last_hash = Some(hash_source(&source));
            let (res, report) = compile::compile_source(&path, &source, stats);
            on_compile(res, report.as_ref());
        }
        Err(e) => on_compile(
            Err(format!("could not read '{}': {e}", path.display())),
            None,
        ),
    }
    println!("--- Watching {} for changes ---", path.display());

    for res in rx {
        match res {
            // Only react to completed saves of the target file; the parent dir
            // also reports unrelated siblings and mid-write noise.
            Ok(event) if is_save_event(&event, &path) => {
                let Ok(source) = fs::read_to_string(&path) else {
                    continue;
                };
                let current_hash = Some(hash_source(&source));
                if current_hash != last_hash {
                    last_hash = current_hash;
                    println!("\n--- Input changed, Re-compiling ---");
                    let (res, report) = compile::compile_source(&path, &source, stats);
                    on_compile(res, report.as_ref());
                }
            }
            Ok(_) => {}
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

/// Determines if a file event is a save event for the target file.
fn is_save_event(event: &Event, target: &Path) -> bool {
    use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};

    if !event.paths.iter().any(|p| p == target) {
        return false;
    }
    match event.kind {
        // Streaming writers (VS Code, nano) on Linux/macOS: one event on close.
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => true,
        // Atomic savers (Vim, Neovim, Sublime): temp file renamed over target.
        EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Both)) => true,
        // Windows emits no Close(Write); it streams Modify(Data) while writing.
        // Read+write open to probe file-lock guard, succeeds if write is done.
        EventKind::Modify(ModifyKind::Data(_)) if cfg!(target_os = "windows") => {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(target)
                .is_ok()
        }
        _ => false,
    }
}

fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
