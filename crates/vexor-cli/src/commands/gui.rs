use clap::Args;
use std::{path::PathBuf, sync::mpsc};
use vexor_gui::NamedSvg;

use crate::commands::file_watch;

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
    let result = file_watch::watch_file(&path, |compiled| match compiled {
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
        Err(e) => eprintln!("{e}"),
    });
    if let Err(e) = result {
        eprintln!("{e}");
    }
}
