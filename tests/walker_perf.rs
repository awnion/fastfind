mod common;

use std::process::Command;

use tempfile::TempDir;

fn chain() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("child/grandchild/great-grandchild")).unwrap();
    dir
}

fn output(dir: &TempDir, args: &[&str]) -> Vec<String> {
    let result = Command::new(common::fastfind_bin()).arg(dir.path()).args(args).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(result.stderr.is_empty());
    String::from_utf8(result.stdout).unwrap().lines().map(String::from).collect()
}

#[test]
fn depth_maxdepth_zero_still_evaluates_root() {
    let dir = chain();
    assert_eq!(output(&dir, &["-depth", "-maxdepth", "0"]), [dir.path().display().to_string()]);
}

#[test]
fn depth_maxdepth_evaluates_boundary_before_parent() {
    let dir = chain();
    assert_eq!(
        output(&dir, &["-depth", "-mindepth", "1", "-maxdepth", "2"]),
        [
            dir.path().join("child/grandchild").display().to_string(),
            dir.path().join("child").display().to_string(),
        ]
    );
}

#[test]
fn prune_walker_maxdepth_evaluates_boundary_after_parent() {
    let dir = chain();
    // A nonmatching prune selects the sequential preorder walker without pruning the chain.
    assert_eq!(
        output(&dir, &["-maxdepth", "2", "-name", "absent", "-prune", "-o", "-print"]),
        [
            dir.path().display().to_string(),
            dir.path().join("child").display().to_string(),
            dir.path().join("child/grandchild").display().to_string(),
        ]
    );
}
