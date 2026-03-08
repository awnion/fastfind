mod common;

use common::*;

#[test]
fn type_l() {
    let dir = generate_test_dir();
    let target = dir.path().join("file1.txt");
    let link = dir.path().join("link_to_file");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "l"]);
}

#[test]
fn follow_symlinks_l() {
    let dir = generate_test_dir();
    let target = dir.path().join("file1.txt");
    let link = dir.path().join("link_to_file");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-L", "-type", "f"]);
}

#[test]
fn follow_symlinks_l_flag() {
    let dir = generate_test_dir();
    let target = dir.path().join("file1.txt");
    let link = dir.path().join("link_to_file");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert_same_output_with_leading(dir.path().to_str().unwrap(), &["-L"], &["-type", "f"]);
}

#[test]
fn xtype_follows_symlinks() {
    let dir = generate_test_dir();
    let target = dir.path().join("file1.txt");
    let link = dir.path().join("link_to_file");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-xtype", "f"]);
}

#[test]
fn lname_pattern() {
    let dir = generate_test_dir();
    let target = dir.path().join("file1.txt");
    let link = dir.path().join("link_to_file");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert_same_output(dir.path().to_str().unwrap(), &["-lname", "*.txt"]);
}

#[test]
fn type_l_no_symlinks() {
    let dir = generate_test_dir();
    assert_same_output(dir.path().to_str().unwrap(), &["-type", "l"]);
}
