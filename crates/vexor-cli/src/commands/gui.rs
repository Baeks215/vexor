use clap::Args;
use notify_debouncer_mini::new_debouncer;
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};
use vexor_gui::NamedSvg;

const WATCH_DEBOUNCE_MS: u64 = 200;

#[derive(Args, Clone)]
pub struct GuiArgs {
    /// Path to the .vx source file
    pub input: PathBuf,
}

pub fn run(args: GuiArgs) {
    let (tx, rx) = mpsc::channel::<Vec<NamedSvg>>();
    let path = args.input.clone();
    let title = format!("vexor — {}", path.display());

    let result = vexor_gui::run(title, rx, move |ctx| {
        std::thread::spawn(move || watch_loop(path, tx, ctx));
    });
    if let Err(e) = result {
        eprintln!("gui error: {e}");
        std::process::exit(1);
    }
}

fn watch_loop(path: PathBuf, tx: mpsc::Sender<Vec<NamedSvg>>, ctx: eframe::egui::Context) {
    let (fs_tx, fs_rx) = mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(WATCH_DEBOUNCE_MS), fs_tx).unwrap();
    if let Err(e) = debouncer
        .watcher()
        .watch(Path::new(&path), notify::RecursiveMode::NonRecursive)
    {
        eprintln!("{e}");
        return;
    }

    let mut last_hash = get_file_hash(&path);
    compile_and_send(&path, &tx, &ctx);
    println!("--- Watching {} for changes ---", path.display());

    for res in fs_rx {
        match res {
            Ok(_events) => {
                let current_hash = get_file_hash(&path);
                if current_hash != last_hash {
                    last_hash = current_hash;
                    println!("\n--- Input changed, Re-compiling ---");
                    compile_and_send(&path, &tx, &ctx);
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}

fn compile_and_send(path: &PathBuf, tx: &mpsc::Sender<Vec<NamedSvg>>, ctx: &eframe::egui::Context) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not read '{}': {e}", path.display());
            return;
        }
    };
    match vexor_compiler::compile_to_svg(&source) {
        Ok(exports) => {
            let exports: Vec<NamedSvg> = exports
                .into_iter()
                .map(|e| NamedSvg {
                    name: e.name,
                    svg: e.data,
                })
                .collect();
            if tx.send(exports).is_ok() {
                ctx.request_repaint();
            }
        }
        Err(e) => eprintln!(
            "compilation failed for '{}':\n\n{}",
            path.display(),
            e.format_colored()
        ),
    }
}

fn get_file_hash(path: &PathBuf) -> Option<u64> {
    let contents = fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Some(hasher.finish())
}
