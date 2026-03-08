use std::io;
use std::io::Write;
use std::process::ExitCode;

use fastfind::cli;
use fastfind::parser;
use fastfind::walker;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match parser::parse(&args) {
        Ok(c) => c,
        Err(e) if e == parser::VERSION_SENTINEL => {
            println!("{}", cli::version_string());
            return ExitCode::SUCCESS;
        }
        Err(e) if e == parser::HELP_SENTINEL => {
            cli::build_cli().print_help().ok();
            println!();
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("find: {e}");
            return ExitCode::FAILURE;
        }
    };

    // validate that all starting paths exist
    for path in &config.paths {
        if !path.exists() {
            eprintln!("find: '{}': No such file or directory", path.display());
            return ExitCode::FAILURE;
        }
    }

    let mut stdout = io::BufWriter::with_capacity(64 * 1024, io::stdout().lock());
    match walker::walk(&config, &mut stdout) {
        Ok(_) => {
            let _ = stdout.flush();
            ExitCode::SUCCESS
        }
        Err(e) => {
            let _ = stdout.flush();
            eprintln!("find: {e}");
            ExitCode::FAILURE
        }
    }
}
