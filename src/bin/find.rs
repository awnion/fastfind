use std::io;
use std::io::Write;
use std::process::ExitCode;

use fastfind::parser;
use fastfind::walker;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_SHA"), ")");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let config = match parser::parse(&args) {
        Ok(c) => c,
        Err(e) if e == parser::VERSION_SENTINEL => {
            println!("find (fastfind) {VERSION}");
            println!("https://crates.io/crates/fastfind");
            return ExitCode::SUCCESS;
        }
        Err(e) if e == parser::HELP_SENTINEL => {
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
    println!("Fast parallel find -- drop-in GNU find replacement");
    println!();
    println!("Usage: find [-H] [-L] [-P] [path...] [expression]");
    println!();
    println!("Options:");
    println!("  -H/-L/-P         symlink following mode");
    println!("  -maxdepth N      descend at most N levels");
    println!("  -mindepth N      skip entries at depth less than N");
    println!("  -depth/-d        process directory contents before directory");
    println!("  -xdev/-mount     don't descend into other filesystems");
    println!("  -noleaf          don't optimize directory link count");
    println!();
    println!("Tests:");
    println!("  -name PATTERN    match filename against glob");
    println!("  -iname PATTERN   case-insensitive -name");
    println!("  -path PATTERN    match full path against glob");
    println!("  -ipath PATTERN   case-insensitive -path");
    println!("  -regex PATTERN   match full path against regex");
    println!("  -iregex PATTERN  case-insensitive -regex");
    println!("  -type TYPE       f/d/l/b/c/p/s (comma-separated)");
    println!("  -xtype TYPE      like -type but checks symlink target");
    println!("  -size N[ckMG]    file size");
    println!("  -empty            empty file or directory");
    println!("  -perm MODE       permission bits");
    println!("  -user NAME       file owner");
    println!("  -group NAME      file group");
    println!("  -uid/-gid N      numeric owner/group");
    println!("  -nouser/-nogroup  no matching user/group");
    println!("  -readable/-writable/-executable");
    println!("  -links N         hard link count");
    println!("  -inum N          inode number");
    println!("  -samefile FILE   same inode");
    println!("  -mtime/-mmin N   modification time");
    println!("  -atime/-amin N   access time");
    println!("  -ctime/-cmin N   change time");
    println!("  -newer FILE      newer than file");
    println!("  -newerXY FILE    compare timestamps");
    println!("  -used N          days since last access after status change");
    println!("  -lname PATTERN   symlink target matches glob");
    println!("  -true/-false     always true/false");
    println!();
    println!("Actions:");
    println!("  -print           print path + newline (default)");
    println!("  -print0          print path + NUL");
    println!("  -printf FORMAT   formatted output");
    println!("  -ls              ls -dils format");
    println!("  -exec CMD ;      execute per file");
    println!("  -exec CMD {{}} +   batch execution");
    println!("  -execdir CMD ;   execute from file's directory");
    println!("  -ok CMD ;        execute with confirmation");
    println!("  -delete          delete matched files");
    println!("  -prune           don't descend into directory");
    println!("  -quit            exit immediately");
    println!("  -fprint FILE     print to file");
    println!("  -fprint0 FILE    print0 to file");
    println!("  -fprintf FILE FMT  printf to file");
    println!("  -fls FILE        ls to file");
    println!();
    println!("Operators:");
    println!("  ( EXPR )         grouping");
    println!("  ! EXPR / -not    negation");
    println!("  EXPR -a EXPR     AND (implicit)");
    println!("  EXPR -o EXPR     OR");
    println!("  EXPR , EXPR      list");
    println!();
    println!("  --version        show version");
    println!("  --help           show this help");
}
