use notify_debouncer_mini::new_debouncer;
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use crate::commands::compile;

const WATCH_DEBOUNCE_MS: u64 = 200;

pub fn run(args: compile::CompileArgs) {
    let path = &args.input;

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(WATCH_DEBOUNCE_MS), tx).unwrap();

    debouncer
        .watcher()
        .watch(Path::new(path), notify::RecursiveMode::NonRecursive)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

    let mut last_hash = get_file_hash(path);

    pipeline(args.clone());
    println!("--- Watching {} for changes ---", path.display());

    for res in rx {
        match res {
            Ok(_events) => {
                let current_hash = get_file_hash(path);
                if current_hash != last_hash {
                    last_hash = current_hash;
                    println!("\n--- Input changed, Re-compiling ---");
                    pipeline(args.clone());
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}

fn pipeline(args: compile::CompileArgs) {
    if let Err(e) = compile::compile_to_svg(args) {
        eprintln!("Error: {}", e)
    }
}

fn get_file_hash(path: &PathBuf) -> Option<u64> {
    let contents = fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Some(hasher.finish())
}
