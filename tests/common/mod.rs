#![allow(dead_code)]

use std::process::Command;

use tempfile::TempDir;

pub const GNU_FIND: &str = "gfind";

pub fn fastfind_bin() -> std::path::PathBuf {
    let mut path =
        std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("find");
    path
}

/// Create a test directory tree:
/// tmp/
///   file1.txt
///   file2.rs
///   .hidden
///   sub/
///     file3.txt
///     deep/
///       file4.log
///   empty/
pub fn generate_test_dir() -> TempDir {
    let dir = TempDir::new().unwrap();

    std::fs::write(dir.path().join("file1.txt"), "hello\n").unwrap();
    std::fs::write(dir.path().join("file2.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join(".hidden"), "secret\n").unwrap();

    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file3.txt"), "world\n").unwrap();

    let deep = sub.join("deep");
    std::fs::create_dir(&deep).unwrap();
    std::fs::write(deep.join("file4.log"), "log line\n").unwrap();

    let empty = dir.path().join("empty");
    std::fs::create_dir(&empty).unwrap();

    dir
}

/// Run GNU find and fastfind with the same args, return sorted stdout lines + exit codes.
pub fn run_both(dir: &str, args: &[&str]) -> (Vec<String>, Vec<String>, i32, i32) {
    let gnu = Command::new(GNU_FIND).arg(dir).args(args).output().expect("failed to run GNU find");

    let fast =
        Command::new(fastfind_bin()).arg(dir).args(args).output().expect("failed to run fastfind");

    let mut gnu_lines: Vec<String> =
        String::from_utf8_lossy(&gnu.stdout).lines().map(String::from).collect();
    let mut fast_lines: Vec<String> =
        String::from_utf8_lossy(&fast.stdout).lines().map(String::from).collect();

    gnu_lines.sort();
    fast_lines.sort();

    let gnu_exit = gnu.status.code().unwrap_or(-1);
    let fast_exit = fast.status.code().unwrap_or(-1);

    (gnu_lines, fast_lines, gnu_exit, fast_exit)
}

/// Assert that fastfind produces the same sorted output and exit code as GNU find.
pub fn assert_same_output(dir: &str, args: &[&str]) {
    let (gnu, fast, gnu_exit, fast_exit) = run_both(dir, args);
    assert_eq!(
        gnu_exit, fast_exit,
        "exit codes differ for args {args:?}: gnu={gnu_exit}, fast={fast_exit}\ngnu: {gnu:?}\nfast: {fast:?}"
    );
    assert_eq!(gnu, fast, "output differs for args {args:?}\ngnu: {gnu:?}\nfast: {fast:?}");
}

/// Assert same output when flags appear before the path (e.g. -L).
pub fn assert_same_output_with_leading(dir: &str, leading: &[&str], args: &[&str]) {
    let mut gnu_args = Vec::new();
    gnu_args.extend_from_slice(leading);
    gnu_args.push(dir);
    gnu_args.extend_from_slice(args);

    let mut fast_args = Vec::new();
    fast_args.extend_from_slice(leading);
    fast_args.push(dir);
    fast_args.extend_from_slice(args);

    let gnu = Command::new(GNU_FIND).args(&gnu_args).output().expect("failed to run GNU find");
    let fast =
        Command::new(fastfind_bin()).args(&fast_args).output().expect("failed to run fastfind");

    let mut gnu_lines: Vec<String> =
        String::from_utf8_lossy(&gnu.stdout).lines().map(String::from).collect();
    let mut fast_lines: Vec<String> =
        String::from_utf8_lossy(&fast.stdout).lines().map(String::from).collect();

    gnu_lines.sort();
    fast_lines.sort();

    let gnu_exit = gnu.status.code().unwrap_or(-1);
    let fast_exit = fast.status.code().unwrap_or(-1);

    assert_eq!(
        gnu_exit, fast_exit,
        "exit codes differ: gnu={gnu_exit}, fast={fast_exit}\ngnu: {gnu_lines:?}\nfast: {fast_lines:?}"
    );
    assert_eq!(gnu_lines, fast_lines, "output differs\ngnu: {gnu_lines:?}\nfast: {fast_lines:?}");
}

/// Run fastfind and return sorted stdout lines.
pub fn run_fast(dir: &str, args: &[&str]) -> (Vec<String>, i32) {
    let out =
        Command::new(fastfind_bin()).arg(dir).args(args).output().expect("failed to run fastfind");

    let mut lines: Vec<String> =
        String::from_utf8_lossy(&out.stdout).lines().map(String::from).collect();
    lines.sort();

    (lines, out.status.code().unwrap_or(-1))
}
