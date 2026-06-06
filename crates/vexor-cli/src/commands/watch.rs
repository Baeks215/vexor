use crate::commands::{compile, file_watch};

pub fn run(args: compile::CompileArgs) {
    let input = args.input.clone();
    let result = file_watch::watch_file(&input, args.stats, |compiled, report| match compiled {
        Ok(exports) => match compile::write_exports(&args.output, &exports) {
            Ok(()) => {
                println!("--- Compiled successfully ---");
                if let Some(report) = report {
                    println!("{report}");
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        Err(e) => eprintln!("Error: {e}"),
    });
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
