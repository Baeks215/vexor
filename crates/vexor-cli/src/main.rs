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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile(args) => commands::compile::run(args),
    }
}
