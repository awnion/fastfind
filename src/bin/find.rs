use std::io;
use std::io::Write;
use std::process::ExitCode;

use fastfind::cli::Config;
use fastfind::walker;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_SHA"), ")");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match Config::parse(&args) {
        Ok(c) => c,
        Err(e) if e == "__version__" => {
            println!("find (fastfind) {VERSION}");
            println!("https://crates.io/crates/fastfind");
            return ExitCode::SUCCESS;
        }
        Err(e) if e == "__help__" => {
            print_help();
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

fn print_help() {
    println!("find (fastfind) {VERSION}");
    println!("Fast parallel find — drop-in GNU find replacement for AI agents");
    println!();
    println!("Usage: find [path...] [expression]");
    println!();
    println!("Options:");
    println!("  -name PATTERN    match filename against glob pattern");
    println!("  -type f|d|l      filter by type: file, directory, symlink");
    println!("  -maxdepth N      descend at most N levels");
    println!("  -mindepth N      skip entries at depth less than N");
    println!("  -print           print matching entries (default)");
    println!("  --version        show version");
    println!("  --help           show this help");
}
