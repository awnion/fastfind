use std::io;
use std::io::Write;
use std::process::ExitCode;

use fastfind::cli::Config;
use fastfind::walker;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match Config::parse(&args) {
        Ok(c) => c,
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

    let mut stdout = io::BufWriter::new(io::stdout().lock());
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
