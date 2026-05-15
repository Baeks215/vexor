use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(version, about = "CLI tool for vexor language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .vx source file
    Compile(commands::compile::CompileArgs),
    /// Compile a .vx source file and watch for changes
    Watch(commands::compile::CompileArgs),
    /// Render compiled exports in a live GUI window
    Gui(commands::gui::GuiArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile(args) => commands::compile::run(args),
        Commands::Watch(args) => commands::watch::run(args),
        Commands::Gui(args) => commands::gui::run(args),
    }
}
